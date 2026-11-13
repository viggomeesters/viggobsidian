// Required JS platform boundary tests; no validation rules are implemented here.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { Worker as NodeWorker } from "node:worker_threads";
import { test } from "node:test";
import { validateVaultSnapshot } from "./vault-adapter.mjs";

const wasmBytes = readFileSync(process.env.LINK_STYLE_WASM);
const native = process.env.LINK_STYLE_NATIVE;
function createWorker() {
  const url = new URL("./validator-worker.mjs", import.meta.url).href;
  const node = new NodeWorker(`import { parentPort } from 'node:worker_threads';
    globalThis.self={postMessage:data=>parentPort.postMessage(data)};
    await import(${JSON.stringify(url)});
    parentPort.on('message',data=>self.onmessage({data}));`, { eval: true });
  const facade = { postMessage: (data) => node.postMessage(data), terminate: () => node.terminate() };
  node.on("message", (data) => facade.onmessage?.({ data }));
  node.on("error", () => facade.onerror?.());
  return facade;
}
class SyntheticVault {
  listeners = new Map();
  constructor(documents) {
    this.documents = documents;
    this.files = Object.entries(documents).map(([path, source]) => ({ path, stat: {
      mtime: 1, ctime: 1, size: new TextEncoder().encode(source).length,
    } }));
  }
  getMarkdownFiles() { return this.files; }
  async read(file) { return this.documents[file.path]; }
  on(name, callback) { const token = {}; this.listeners.set(token, { name, callback }); return token; }
  offref(token) { this.listeners.delete(token); }
  change() { for (const { callback } of this.listeners.values()) callback(); }
}
const documents = {
  "owner.md": "# Owner\n[[d/wiki/wiki]] [[missing/path]] [[wiki]]\n",
  "d/wiki/wiki.md": "---\r\nlayer: wiki\r\n---\r\n# Élève\r\n[[d/raw/source]]\r\n",
  "other/wiki/wiki.md": "# Other\n",
  "d/raw/source.md": "# Source\n\n## Activity\nHuman [[missing/path]]\n> Quote [[missing/path]]\n",
  "mask.md": "# Mask\n```md\n[[missing/path]]\n```\n> Quote [[missing/path]]\n`[[missing/path]]`\n",
};

test("real WASM worker equals native core for complete and invalid snapshots", async () => {
  const module = await WebAssembly.compile(wasmBytes);
  assert.deepEqual(WebAssembly.Module.imports(module), []);
  const base = { schema: "viggobsidian.link-style-snapshot.v1", complete: true,
    inventory: Object.keys(documents), notes: Object.entries(documents).map(([path, markdown]) => ({ path, markdown })) };
  for (const snapshot of [base, { ...base, targets: ["owner.md"] }, { ...base, complete: false },
    { ...base, notes: base.notes.slice(1) }, { ...base, inventory: [...base.inventory, base.inventory[0]] },
    { ...base, targets: ["missing.md"] }]) {
    const result = spawnSync(native, { input: JSON.stringify(snapshot), encoding: "utf8" });
    assert.notEqual(result.status, null, result.error?.message);
    const expected = JSON.parse(result.stdout);
    const worker = createWorker();
    try {
      const actual = await new Promise((resolve, reject) => {
        const timer = setTimeout(() => reject(new Error("worker timeout")), 10_000);
        worker.onmessage = ({ data }) => { clearTimeout(timer); resolve(data); };
        worker.onerror = () => { clearTimeout(timer); reject(new Error("worker error")); };
        worker.postMessage({ wasmBytes, snapshot });
      });
      assert.deepEqual(actual, expected);
    } finally { worker.terminate(); }
  }
});

test("synthetic Vault API capture runs the worker and cleans every subscription", async () => {
  const vault = new SyntheticVault(documents);
  const actual = await validateVaultSnapshot(vault, wasmBytes, { createWorker });
  assert.equal(actual.status, "complete");
  assert.ok(actual.report.issues.some((issue) => issue.kind === "source-link-label"));
  assert.ok(actual.report.issues.some((issue) => issue.kind === "wiki-source-style-class"));
  assert.equal(vault.listeners.size, 0);
});

test("a stalled Vault read is bounded and still releases event subscriptions", async () => {
  const vault = new SyntheticVault(documents);
  vault.read = () => new Promise(() => {});
  const actual = await validateVaultSnapshot(vault, wasmBytes, { timeoutMs: 5, createWorker });
  assert.equal(actual.status, "incomplete");
  assert.equal(vault.listeners.size, 0);
});

test("changes during capture or worker evaluation never return complete", async () => {
  for (const duringCapture of [true, false]) {
    const vault = new SyntheticVault(documents);
    if (duringCapture) vault.read = async (file) => { vault.change(); return documents[file.path]; };
    const factory = () => { const worker = createWorker(); vault.change(); return worker; };
    const actual = await validateVaultSnapshot(vault, wasmBytes, { createWorker: factory });
    assert.equal(actual.status, "incomplete");
    assert.equal(actual.reason, "snapshot_changed");
    assert.equal(vault.listeners.size, 0);
  }
});

test("timeouts, cancellation and capture failures remain incomplete and clean up", async () => {
  const controller = new AbortController();controller.abort();
  for (const options of [{ signal: controller.signal }, { timeoutMs: 1, createWorker: () => ({ postMessage() {}, terminate() {} }) }]) {
    const vault = new SyntheticVault(documents);
    const actual = await validateVaultSnapshot(vault, wasmBytes, options);
    assert.equal(actual.status, "incomplete");assert.equal(vault.listeners.size, 0);
  }
  const vault = new SyntheticVault(documents);vault.read = async () => { throw new Error("unreadable"); };
  assert.equal((await validateVaultSnapshot(vault, wasmBytes, { createWorker })).status, "incomplete");
  assert.equal(vault.listeners.size, 0);
});
