// Optional real Obsidian host experiment. Pass reviewed local module/bytes explicitly.
// Never installs a plugin or changes the active leaf, settings or plugin state.
export async function runHostProof(app, validateVaultSnapshot, workerSource, wasmBytes) {
  const root = "fixtures/viggobsidian-link-style-spike";
  if (app.vault.getAbstractFileByPath(root)) throw new Error("fixture_collision");
  const before = { active: app.workspace.getActiveFile()?.path, plugins: [...app.plugins.enabledPlugins].sort() };
  const created = [];
  let workerURL;
  let proof;
  try {
    for (const path of [root, `${root}/wiki`, `${root}/raw`]) {
      await app.vault.createFolder(path);created.push(path);
    }
    const documents = {
      [`${root}/wiki/check.md`]: `---\nlayer: wiki\n---\n# Check\n[[${root}/raw/evidence]]\n`,
      [`${root}/raw/evidence.md`]: "# Fictional evidence\n\nHuman [[missing/path]]\n",
    };
    for (const [path, source] of Object.entries(documents)) {
      await app.vault.create(path, source);created.push(path);
    }
    workerURL = URL.createObjectURL(new Blob([workerSource], { type: "text/javascript" }));
    const vault = { getMarkdownFiles: () => app.vault.getMarkdownFiles().filter((file) => file.path.startsWith(root + "/")),
      read: (file) => app.vault.read(file), on: (...args) => app.vault.on(...args), offref: (ref) => app.vault.offref(ref) };
    const report = await validateVaultSnapshot(vault, wasmBytes, { createWorker: () => new Worker(workerURL) });
    const issues = report.report?.issues ?? [];
    if (report.status !== "complete" || !issues.some((i) => i.kind === "source-link-label") || !issues.some((i) => i.kind === "wiki-source-style-class")) {
      throw new Error("host_worker_findings_missing");
    }
    const contentUnchanged = (await Promise.all(Object.entries(documents).map(async ([path, content]) =>
      (await app.vault.read(app.vault.getAbstractFileByPath(path))) === content))).every(Boolean);
    proof = { status: "pass", host: "Obsidian desktop", scope: "fictional fixture namespace only", realVaultApi: true,
      realBrowserWorker: true, wasm: true, inventoried: report.report.inventoried, checked: report.report.checked,
      findings: issues.map(({ kind, line }) => ({ kind, line })), contentUnchanged };
    if (!contentUnchanged) throw new Error("fixture_changed");
  } catch (error) {
    proof = { status: "fail", reason: String(error) };
  } finally {
    if (workerURL) URL.revokeObjectURL(workerURL);
    for (const path of created.reverse()) {
      const file = app.vault.getAbstractFileByPath(path);
      if (file) await app.vault.delete(file, true);
    }
    const after = { active: app.workspace.getActiveFile()?.path, plugins: [...app.plugins.enabledPlugins].sort() };
    proof.statePreserved = JSON.stringify(before) === JSON.stringify(after);
    proof.fixturesRemoved = !app.vault.getAbstractFileByPath(root);
    if (!proof.statePreserved || !proof.fixturesRemoved) proof.status = "fail";
  }
  return proof;
}
