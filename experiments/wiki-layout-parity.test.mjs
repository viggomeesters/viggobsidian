// Necessary WebAssembly platform test interface; all rule logic remains Rust.
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import { test } from "node:test";

const corpus = JSON.parse(readFileSync(new URL("../crates/wiki-layout-core/fixtures/wiki-layout-legacy.json", import.meta.url)));
const module = new WebAssembly.Module(readFileSync(process.env.WIKI_LAYOUT_WASM));
const native = process.env.WIKI_LAYOUT_NATIVE;
const base = { schema: "viggobsidian.wiki-layout-snapshot.v1", ruleset: "wiki-layout.v1", complete: true,
  profile: "numbered-v1", classification: "wiki", path: "d/wiki/wiki.md", body: "" };

function wasm(bytes) {
  const { exports: e } = new WebAssembly.Instance(module);
  const pointer = e.request_buffer(bytes.length);
  if (bytes.length && !pointer) throw new Error("request rejected");
  new Uint8Array(e.memory.buffer, pointer, bytes.length).set(bytes);
  const length = e.validate();
  return JSON.parse(new TextDecoder("utf-8", { fatal: true }).decode(new Uint8Array(e.memory.buffer, e.response_buffer(), length)));
}

test("the built WASM enforces the 256 MiB linear-memory maximum", () => {
  const { memory } = new WebAssembly.Instance(module).exports;
  const maximumPages = 256 * 1024 * 1024 / 65536;
  const initialPages = memory.buffer.byteLength / 65536;
  assert.ok(initialPages > 0 && initialPages <= maximumPages);
  // Fixed bounded virtual growth; no Markdown or other host data fills this memory.
  memory.grow(maximumPages - initialPages);
  assert.equal(memory.buffer.byteLength, 256 * 1024 * 1024);
  assert.throws(() => memory.grow(1), RangeError);
});

test("actual import-free WASM and native CLI equal frozen legacy corpus and failure states", () => {
  assert.deepEqual(WebAssembly.Module.imports(module), []);
  const cases = corpus.cases.map(c => ({ ...base, body: c.body }));
  cases.push({ ...base, complete: false }, { ...base, schema: "future" }, { ...base, ruleset: "future" },
    { ...base, classification: "note" }, { ...base, profile: "none" }, { ...base, classification: null },
    { ...base, path: "../wiki.md" }, { ...base, body: "x".repeat(1024 * 1024 + 1) });
  const portable = {...base,schema:"viggobsidian.wiki-layout-snapshot.v2",cssclasses:[]};
  for(const path of ["d/wiki/wiki.md","d/wiki/nested/wiki.md","d/wiki/note.md","d/wiki.md","d/d.md"]) {
    for(const cssclasses of [[],["vault-wiki-direct"],null,[1]]) cases.push({...portable,path,cssclasses,body:"# Example\r\n"});
  }
  for(const field of ["profile","path","cssclasses","body","complete","ruleset"]) {const missing={...portable};delete missing[field];cases.push(missing);}
  cases.push({...portable,profile:"none"},{...portable,schema:"viggobsidian.wiki-layout-snapshot.v3"});
  const requests = [...cases.map(c => Buffer.from(JSON.stringify(c))), Buffer.from("{malformed"), Buffer.from([0xff])];
  for (const input of requests) {
    const result = spawnSync(native, { input, maxBuffer: 8 * 1024 * 1024 });
    assert.ok([0, 2].includes(result.status), result.stderr?.toString());
    const expected = JSON.parse(result.stdout);
    assert.deepEqual(wasm(input), expected);
  }
  for (const c of corpus.cases) {
    const result = wasm(Buffer.from(JSON.stringify({ ...base, body: c.body })));
    assert.equal(result.report.has_facts, c.has_facts);
    assert.deepEqual(result.report.findings, c.findings.map(f => ({ path: base.path, ...f })));
  }
});
