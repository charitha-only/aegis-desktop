#!/usr/bin/env node
import test from 'node:test';
import assert from 'node:assert/strict';
import {createServer as httpServer} from 'node:http';
import {createServer as viteServer} from '../../src-ui/node_modules/vite/dist/node/index.js';
import path from 'node:path';

async function fixture(handler, run) {
  const sockets=new Set();
  const server=httpServer(handler);
  server.on('connection',socket=>{sockets.add(socket);socket.on('close',()=>sockets.delete(socket));});
  await new Promise(resolve=>server.listen(0,'127.0.0.1',resolve));
  const origin=`http://127.0.0.1:${server.address().port}`;
  const vite=await viteServer({root:path.resolve('src-ui'),configFile:false,envDir:false,
    server:{middlewareMode:true,watch:null,hmr:false,ws:false},
    define:{'import.meta.env.PROD':'false','import.meta.env.VITE_AEGIS_BRIDGE_URL':JSON.stringify(origin)}});
  try {await run(await vite.ssrLoadModule('/src/aegis-bridge.ts'));}
  finally {
    await vite.close();
    for(const socket of sockets) socket.destroy();
    await new Promise(resolve=>server.close(resolve));
  }
}
function json(response,value,status=200) {
  response.writeHead(status,{'Content-Type':'application/json'});response.end(JSON.stringify(value));
}
test('SDK rejects malformed handshakes without retaining credentials',async()=>{
  await fixture((_,response)=>json(response,{token:42}),async({aegis})=>{
    assert.equal((await aegis.init()).success,false);
    assert.equal(aegis.getNonce(),null);
  });
});
test('SDK rejects HTTP failures and mismatched or malformed IPC envelopes',async()=>{
  for(const kind of ['http','id','shape','json']) {
    await fixture(async(request,response)=>{
      let body='';for await(const chunk of request) body+=chunk;
      const {id}=JSON.parse(body);
      if(kind==='json') {response.writeHead(200,{'Content-Type':'application/json'});response.end('{');return;}
      json(response,kind==='shape'?{id,success:'yes'}:{id:kind==='id'?'other':id,success:true,data:42},kind==='http'?503:200);
    },async({aegis})=>{
      const result=await aegis.invoke('fixture',{},'a'.repeat(64));
      assert.equal(result.success,false,kind);
      assert.equal(result.errorCode,kind==='http'?'HTTP_ERROR':'INVALID_RESPONSE');
      assert.match(result.id,/^req_/);
    });
  }
});
test('SDK deadlines cover stalled handshake and IPC response bodies and allow retry',async()=>{
  let stalled=true;
  await fixture(async(request,response)=>{
    if(stalled) {response.writeHead(200,{'Content-Type':'application/json'});response.write('{');return;}
    json(response,{token:'a'.repeat(64)});
  },async({AegisBridge})=>{
    const sdk=new AegisBridge({requestTimeoutMs:500});
    let watchdog;
    try {
      const result=await Promise.race([sdk.init(),new Promise((_,reject)=>{watchdog=setTimeout(()=>reject(new Error('SDK deadline did not settle')),3000);})]);
      assert.equal(result.success,false);
      assert.equal(result.errorCode,'TIMEOUT');
      assert.equal(sdk.getNonce(),null);
      stalled=false;
      assert.equal((await sdk.init()).success,true);
      stalled=true;
      const invocation=await sdk.invoke('fixture');
      assert.equal(invocation.success,false);
      assert.equal(invocation.errorCode,'TIMEOUT');
    } finally {clearTimeout(watchdog);}
  });
});
test('SDK rate-limit errors remain strings when the server sends an invalid error field',async()=>{
  await fixture((_,response)=>json(response,{error:{untrusted:true}},429),async({aegis})=>{
    const result=await aegis.invoke('fixture',{},'a'.repeat(64));
    assert.equal(result.success,false);
    assert.equal(result.errorCode,'HTTP_ERROR');
    assert.equal(typeof result.error,'string');
  });
});
