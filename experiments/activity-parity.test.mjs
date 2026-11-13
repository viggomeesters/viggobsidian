// Necessary native/WebAssembly platform interface; no Activity rules in JavaScript.
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import {spawnSync} from 'node:child_process';
import {test} from 'node:test';
const corpus=JSON.parse(readFileSync('crates/activity-core/fixtures/activity-legacy.json'));
const module=new WebAssembly.Module(readFileSync(process.env.ACTIVITY_WASM||'target/wasm32-unknown-unknown/release/activity_core.wasm'));
const native=process.env.ACTIVITY_NATIVE||'target/debug/activity-snapshot-check';
const base={schema:'viggobsidian.activity-snapshot.v1',ruleset:'activity-text.v1',complete:true,profile:'numbered-v1',path:'example/event.md',frontmatter_present:true,metadata:{activity_contract:'provenance-v1'},body:''};
function wasm(input){const {exports:e}=new WebAssembly.Instance(module);const pointer=e.activity_request_buffer(input.length);if(!pointer)throw Error('request rejected');new Uint8Array(e.memory.buffer,pointer,input.length).set(input);const length=e.activity_validate();return JSON.parse(new TextDecoder('utf-8',{fatal:true}).decode(new Uint8Array(e.memory.buffer,e.activity_response_buffer(),length)));}
test('Activity actual native and import-free WASM preserve frozen human/agent/mail/legacy outputs',()=>{
  assert.deepEqual(WebAssembly.Module.imports(module),[]);
  for(const c of corpus.cases)for(const mode of ['provenance','chat','event','legacy']){
    const metadata=mode==='provenance'?{activity_contract:'provenance-v1'}:mode==='chat'?{activity_format:'chat-v1'}:mode==='event'?{cssclasses:['vault-event-v1-1']}:{};
    const input=Buffer.from(JSON.stringify({...base,metadata,body:c.body}));const output=spawnSync(native,{input,maxBuffer:8*1024*1024});assert.equal(output.status,0,output.stderr?.toString());const expected=JSON.parse(output.stdout);assert.deepEqual(wasm(input),expected,`${c.id} ${mode}`);
    if(mode!=='legacy'){
      const findings=[...c.mail.map(message=>({path:base.path,code:'strict-mail-draft',message})),...c.delivery.map(message=>({path:base.path,code:'strict-activity-delivery',message}))];
      const messages=mode==='provenance'?c.provenance:mode==='chat'?c.chat:c.event?[c.event]:[];
      const code=mode==='provenance'?'strict-activity-provenance':mode==='chat'?'activity-message':'event-activity-layout';findings.push(...messages.map(message=>({path:base.path,code,message})));
      assert.deepEqual(expected.report.findings,findings,`${c.id} ${mode} frozen`);
    }
  }
});
test('Activity malformed, unsupported and over-budget inputs have exact native/WASM failure parity',()=>{
  const cases=[{...base,complete:false},{...base,schema:'future'},{...base,ruleset:'future'},{...base,profile:'none'},{...base,frontmatter_present:false},{...base,path:'../note.md'},{...base,metadata:[]},{...base,metadata:{cssclasses:[1]}},{...base,body:'x'.repeat(1024*1024+1)}];
  for(const key of Object.keys(base)){const input={...base};delete input[key];cases.push(input);}
  const requests=[...cases.map(c=>Buffer.from(JSON.stringify(c))),Buffer.from('{malformed'),Buffer.from([255])];
  for(const input of requests){const output=spawnSync(native,{input,maxBuffer:8*1024*1024});assert.ok([0,2].includes(output.status));assert.deepEqual(wasm(input),JSON.parse(output.stdout));}
});
test('Activity actual WASM enforces the 256 MiB memory maximum',()=>{
  const {memory}=new WebAssembly.Instance(module).exports;const max=4096;memory.grow(max-memory.buffer.byteLength/65536);assert.equal(memory.buffer.byteLength,256*1024*1024);assert.throws(()=>memory.grow(1),RangeError);
});
