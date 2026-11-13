// Necessary Node/WebAssembly/Worker platform proof; no structure rules here.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {performance} from 'node:perf_hooks';
import {Worker, isMainThread, parentPort, workerData} from 'node:worker_threads';
import {test} from 'node:test';

const fixture = JSON.parse(readFileSync(new URL('../crates/structure-core/fixtures/dossier.json', import.meta.url)));
const wasmBytes = readFileSync(process.env.STRUCTURE_WASM || 'target/wasm32-unknown-unknown/release/structure_core.wasm');
const module = new WebAssembly.Module(wasmBytes);
const native = process.env.STRUCTURE_NATIVE || 'target/debug/structure-snapshot-check';
const incomplete = reason => ({schema:'viggobsidian.structure-result.v1', status:'incomplete', reason});

function evaluate(input) {
  const {exports:e} = new WebAssembly.Instance(module);
  const pointer = e.structure_request_buffer(input.length);
  if (!pointer) return incomplete('request_limit');
  new Uint8Array(e.memory.buffer, pointer, input.length).set(input);
  const length = e.structure_validate();
  return JSON.parse(new TextDecoder('utf-8', {fatal:true}).decode(new Uint8Array(e.memory.buffer, e.structure_response_buffer(), length)));
}
function parity(value) {
  const input = Buffer.isBuffer(value) ? value : Buffer.from(JSON.stringify(value));
  const process = spawnSync(native, {input, maxBuffer:2*1024*1024});
  assert.ok([0,2].includes(process.status), process.stderr?.toString());
  const result = JSON.parse(process.stdout);
  assert.deepEqual(evaluate(input), result);
  return result;
}

// Models the future host's lifecycle only. Browser/Obsidian capture is not tested.
async function runWorker(input, {signal, timeoutMs=10_000, currentCapture=()=>input.capture.before, stall=false, duringCleanup=()=>{}}={}) {
  let worker, timer, abort, candidate;
  try {
    if (signal?.aborted) return incomplete('cancelled');
    worker = new Worker(new URL(import.meta.url), {workerData:{input, stall}, resourceLimits:{maxOldGenerationSizeMb:64}});
    candidate = await Promise.race([
      new Promise(resolve => {worker.once('message', resolve); worker.once('error', ()=>resolve(incomplete('worker_failed'))); worker.once('exit', code=>{if(code !== 0) resolve(incomplete('worker_failed'));});}),
      new Promise(resolve => {
        timer = setTimeout(()=>resolve(incomplete('timeout')), timeoutMs);
        abort = ()=>resolve(incomplete('cancelled'));
        signal?.addEventListener('abort', abort, {once:true});
        if (signal?.aborted) abort();
      }),
    ]);
  } finally {
    clearTimeout(timer);
    if (abort) signal?.removeEventListener('abort', abort);
    if (worker) {const terminated=worker.terminate();duringCleanup();await terminated;assert.equal(worker.threadId, -1, 'Worker is actually gone');}
  }
  // Awaited termination itself yields: keep freshness checks after cleanup.
  if (signal?.aborted) return incomplete('cancelled');
  if (currentCapture() !== input.capture.before) return incomplete('snapshot_changed');
  return candidate;
}

if (!isMainThread) {
  if (workerData.stall) setInterval(()=>{}, 1000); // Host failure injection, never rule logic.
  else parentPort.postMessage(evaluate(Buffer.from(JSON.stringify(workerData.input))));
} else {
  test('frozen fictional dossier has exact native/import-free WASM diagnostic parity', ()=>{
    assert.deepEqual(WebAssembly.Module.imports(module), []);
    assert.deepEqual(parity(fixture.input), fixture.expected);
    const input = structuredClone(fixture.input);
    input.entries.push({path:'fiction/raw/raw.md',kind:'file'});
    input.entries.find(entry=>entry.path==='fiction/wiki/wiki.md').kind='file';
    assert.deepEqual(parity(input).findings, []);
    // Path matching stays literal, case-sensitive and Unicode-preserving.
    for(const name of ['Case', 'é', 'e\u0301', '漢字']) {
      const path=`fiction/${name}`;
      const selected={...input, folders:[path], entries:[{path:'fiction',kind:'folder'},{path,kind:'folder'},{path:`${path}/${name.toLowerCase()}.md`,kind:'file'}]};
      assert.equal(parity(selected).findings.length,name==='Case'?1:0);
    }
  });
  test('missing, malformed, drifted and unknown inputs are incomplete in both engines', ()=>{
    for (const key of Object.keys(fixture.input)) {const value=structuredClone(fixture.input); delete value[key]; assert.equal(parity(value).status,'incomplete',key);}
    for (const mutation of [
      {capture:{before:'a',after:'b'}}, {capabilities:[]}, {schema:'future'}, {ruleset:'future'},
      {entries:[{path:'fiction',kind:'folder'},{path:'fiction',kind:'file'}]},
      {entries:[{path:'fiction',kind:'folder'},{path:'fiction/absent/child',kind:'folder'}]},
      {scope:'fiction\\raw'}, {scope:'../fiction'}, {folders:['fiction','fiction']},
    ]) assert.equal(parity({...fixture.input,...mutation}).status,'incomplete');
    assert.equal(parity({...fixture.input,profile:'none'}).status,'not_applicable');
    assert.equal(parity(Buffer.from('{bad')).status,'incomplete');
    assert.equal(parity(Buffer.from([255])).status,'incomplete');
    assert.equal(parity(Buffer.alloc(1024*1024+1)).reason,'request_limit');
  });
  test('maximum admitted inventory completes below a ten second evaluation budget', ()=>{
    const input={...fixture.input,folders:[],entries:[{path:'fiction',kind:'folder'}]};
    for(let i=0;i<1_000;i++) {const path=`fiction/f${i}`;input.folders.push(path);input.entries.push({path,kind:'folder'},{path:`${path}/f${i}.md`,kind:'file'});}
    for(let i=input.entries.length;i<10_000;i++) input.entries.push({path:`fiction/asset${i}`,kind:'file'});
    const started=performance.now();
    assert.deepEqual(parity(input).findings, []);
    const elapsedMs=performance.now()-started;
    assert.ok(elapsedMs<10_000, `${elapsedMs} ms`);
    console.log(JSON.stringify({proof:'structure-max-inventory',entries:10_000,elapsedMs}));
    input.entries.push({path:'fiction/over',kind:'file'});
    assert.equal(parity(input).reason,'inventory_limit');
    assert.equal(parity({...fixture.input,folders:Array(1001).fill('fiction')}).reason,'inventory_limit');
    input.entries=[{path:'fiction',kind:'folder'}]; input.folders=[];
    for(let i=0;i<101;i++){const path=`fiction/f${i}`;input.entries.push({path,kind:'folder'});input.folders.push(path);}
    assert.equal(parity(input).reason,'finding_limit');
  });
  test('actual WASM linear memory is capped at 256 MiB', ()=>{
    const {memory}=new WebAssembly.Instance(module).exports;
    memory.grow(4096-memory.buffer.byteLength/65536);
    assert.equal(memory.buffer.byteLength,256*1024*1024);
    assert.throws(()=>memory.grow(1),RangeError);
  });
  test('actual Worker success, drift, deadline, cancel and unload leave no Worker or stale success', async ()=>{
    assert.deepEqual(await runWorker(fixture.input),fixture.expected);
    assert.equal((await runWorker(fixture.input,{currentCapture:()=> 'changed'})).reason,'snapshot_changed');
    assert.equal((await runWorker(fixture.input,{stall:true,timeoutMs:25})).reason,'timeout');
    for(const lifecycle of ['cancel','unload']) {
      const controller=new AbortController();
      const pending=runWorker(fixture.input,{stall:true,signal:controller.signal});
      controller.abort(lifecycle);
      assert.equal((await pending).reason,'cancelled');
    }
    const controller=new AbortController();controller.abort();
    assert.equal((await runWorker(fixture.input,{signal:controller.signal})).reason,'cancelled');
    const cleanupAbort=new AbortController();
    assert.equal((await runWorker(fixture.input,{signal:cleanupAbort.signal,duringCleanup:()=>cleanupAbort.abort('unload')})).reason,'cancelled');
    let generation=fixture.input.capture.before;
    assert.equal((await runWorker(fixture.input,{currentCapture:()=>generation,duringCleanup:()=>{generation='changed-during-termination';}})).reason,'snapshot_changed');
  });
}
