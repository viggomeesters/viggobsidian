// Obsidian Vault API glue: no rules, disk paths, writes, subprocesses or downloads.
const MAX_FILES = 10_000;
const MAX_NOTE_BYTES = 1024 * 1024;
const MAX_TOTAL_BYTES = 16 * 1024 * 1024;
const MAX_REQUEST_BYTES = 24 * 1024 * 1024;
const incomplete = (reason) => ({ schema: "viggobsidian.link-style-result.v1", status: "incomplete", reason });
const visible = (file) => file.path.endsWith(".md") && file.path.split("/").every((part) => !part.startsWith("."));
const inventory = (vault) => vault.getMarkdownFiles().filter(visible).map((file) => ({
  file, path: file.path, mtime: file.stat.mtime, ctime: file.stat.ctime, size: file.stat.size,
})).sort((a, b) => a.path < b.path ? -1 : a.path > b.path ? 1 : 0);
const signature = (items) => JSON.stringify(items.map(({ path, mtime, ctime, size }) => [path, mtime, ctime, size]));

export async function validateVaultSnapshot(vault, wasmBytes, {
  targets = null,
  signal,
  timeoutMs = 10_000,
  createWorker = () => new Worker(new URL("./validator-worker.mjs", import.meta.url), { type: "module" }),
} = {}) {
  let changed = false;
  let worker;
  let timer;
  let abort;
  const deadline = performance.now() + timeoutMs;
  const events = ["modify", "create", "delete", "rename"].map((name) => vault.on(name, () => { changed = true; }));
  try {
    if (signal?.aborted) return incomplete("cancelled");
    const files = inventory(vault);
    if (files.length > MAX_FILES) return incomplete("file_limit");
    let total = 0;
    const notes = [];
    for (const { file, path, size } of files) {
      if (signal?.aborted) return incomplete("cancelled");
      if (size > MAX_NOTE_BYTES || total + size > MAX_TOTAL_BYTES) return incomplete("content_limit");
      let readTimer;
      let readAbort;
      const markdown = await Promise.race([
        vault.read(file),
        new Promise((_, reject) => {
          readTimer = setTimeout(() => reject(new Error("capture_timeout")), Math.max(0, deadline - performance.now()));
          readAbort = () => reject(new Error("cancelled"));
          signal?.addEventListener("abort", readAbort, { once: true });
          if (signal?.aborted) readAbort();
        }),
      ]).finally(() => {
        clearTimeout(readTimer);
        if (readAbort) signal?.removeEventListener("abort", readAbort);
      });
      const bytes = new TextEncoder().encode(markdown).length;
      total += bytes;
      if (bytes > MAX_NOTE_BYTES || total > MAX_TOTAL_BYTES) return incomplete("content_limit");
      notes.push({ path, markdown });
      if (changed) return incomplete("snapshot_changed");
    }
    if (changed || signature(files) !== signature(inventory(vault))) return incomplete("snapshot_changed");
    const snapshot = { schema: "viggobsidian.link-style-snapshot.v1", complete: true,
      inventory: files.map(({ path }) => path), notes, targets };
    if (new TextEncoder().encode(JSON.stringify(snapshot)).length > MAX_REQUEST_BYTES) return incomplete("request_limit");
    worker = createWorker();
    const result = await new Promise((resolve) => {
      worker.onmessage = ({ data }) => resolve(data);
      worker.onerror = () => resolve(incomplete("worker_failed"));
      timer = setTimeout(() => resolve(incomplete("timeout")), Math.max(0, deadline - performance.now()));
      abort = () => resolve(incomplete("cancelled"));
      signal?.addEventListener("abort", abort, { once: true });
      if (signal?.aborted) { abort(); return; }
      worker.postMessage({ wasmBytes, snapshot });
    });
    if (signal?.aborted) return incomplete("cancelled");
    if (changed || signature(files) !== signature(inventory(vault))) return incomplete("snapshot_changed");
    return result;
  } catch {
    return incomplete("capture_or_worker_failed");
  } finally {
    clearTimeout(timer);
    if (abort) signal?.removeEventListener("abort", abort);
    worker?.terminate();
    events.forEach((event) => vault.offref(event));
  }
}
