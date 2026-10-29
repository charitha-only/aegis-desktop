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
test('SDK bounds received IPC bytes for declared and chunked responses while allowing small results',async()=>{
  for(const chunked of [false,true]) {
    await fixture(async(request,response)=>{
      let body='';for await(const chunk of request) body+=chunk;
      const {id,command}=JSON.parse(body);
      const bytes=Buffer.from(JSON.stringify({id,success:true,data:command==='large'?'é'.repeat(2048):'small'}));
      response.writeHead(200,{'Content-Type':'application/json',...(chunked?{}:{'Content-Length':bytes.length})});
      if(chunked) {response.write(bytes.subarray(0,512));response.end(bytes.subarray(512));} else response.end(bytes);
    },async({AegisBridge})=>{
      const sdk=new AegisBridge({maxResponseBytes:1024});
      const large=await sdk.invoke('large',{},'a'.repeat(64));
      assert.equal(large.success,false);
      assert.equal(large.errorCode,'RESPONSE_TOO_LARGE');
      const small=await sdk.invoke('small',{},'a'.repeat(64));
      assert.equal(small.success,true);
      assert.equal(small.data,'small');
    });
  }
});
test('SDK limits handshake and rate-limit bodies independently of application response settings',async()=>{
  await fixture((request,response)=>json(response,
    request.url==='/handshake'?{token:'a'.repeat(64),padding:'x'.repeat(5000)}:{error:'x'.repeat(5000)},request.url==='/handshake'?200:429),async({aegis})=>{
    const handshake=await aegis.init();
    assert.equal(handshake.success,false);
    assert.equal(handshake.errorCode,'RESPONSE_TOO_LARGE');
    assert.equal(aegis.getNonce(),null);
    const limited=await aegis.invoke('fixture',{},'a'.repeat(64));
    assert.equal(limited.success,false);
    assert.equal(limited.errorCode,'RESPONSE_TOO_LARGE');
  });
});
test('SDK shares one in-flight handshake across simultaneous initialization and invocations',async()=>{
  let handshakes=0,invocations=0;
  await fixture(async(request,response)=>{
    if(request.url==='/handshake') {
      handshakes++;
      setTimeout(()=>json(response,{token:'b'.repeat(64)}),30);
      return;
    }
    let body='';for await(const chunk of request) body+=chunk;
    const {id,nonce}=JSON.parse(body);
    invocations++;
    assert.equal(nonce,'b'.repeat(64));
    json(response,{id,success:true,data:42});
  },async({aegis})=>{
    const results=await Promise.all([aegis.init(),...Array.from({length:8},()=>aegis.invoke('fixture'))]);
    assert.ok(results.every(result=>result.success));
    assert.equal(handshakes,1);
    assert.equal(invocations,8);
    assert.equal(aegis.getNonce(),'b'.repeat(64));
  });
});
test('SDK shares failed initialization without sending commands and permits a fresh shared retry',async()=>{
  let fail=true,handshakes=0,invocations=0;
  await fixture(async(request,response)=>{
    if(request.url==='/handshake') {
      handshakes++;
      setTimeout(()=>json(response,fail?{error:'fixture offline'}:{token:'c'.repeat(64)},fail?503:200),30);
      return;
    }
    let body='';for await(const chunk of request) body+=chunk;
    const {id,nonce}=JSON.parse(body);invocations++;
    assert.equal(nonce,'c'.repeat(64));json(response,{id,success:true});
  },async({aegis})=>{
    const failed=await Promise.all(Array.from({length:8},()=>aegis.invoke('fixture')));
    assert.ok(failed.every(result=>!result.success&&result.errorCode==='HTTP_ERROR'));
    assert.equal(handshakes,1);assert.equal(invocations,0);assert.equal(aegis.getNonce(),null);
    fail=false;
    const retried=await Promise.all(Array.from({length:8},()=>aegis.invoke('fixture')));
    assert.ok(retried.every(result=>result.success));
    assert.equal(handshakes,2);assert.equal(invocations,8);
  });
});
test('SDK rejects invalid byte budgets without creating network requests',async()=>{
  let requests=0;
  await fixture((_,response)=>{requests++;json(response,{});},async({AegisBridge})=>{
    for(const maxResponseBytes of [0,1023,1024.5,NaN,Infinity,67108865]) {
      assert.throws(()=>new AegisBridge({maxResponseBytes}),/maxResponseBytes/);
    }
    assert.equal(requests,0);
  });
});
