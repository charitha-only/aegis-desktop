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
test('SDK validates platform data before exposing typed success and allows additive fields',async()=>{
  const valid={os:'linux',architecture:'x86_64',embeddedWebview:false,lifecycleAdapterCompiled:false,installerAdapterCompiled:false,osKeyStore:false,encryptedFileStorage:true};
  let data;
  await fixture(async(request,response)=>{
    if(request.url==='/handshake') return json(response,{token:'a'.repeat(64)});
    let body='';for await(const chunk of request) body+=chunk;
    json(response,{id:JSON.parse(body).id,success:true,data});
  },async({aegis})=>{
    for(const invalid of [null,[],{}, {...valid,os:'unknown'}, {...valid,architecture:42}, {...valid,embeddedWebview:'false'}]) {
      data=invalid;
      const result=await aegis.platformInfo();
      assert.equal(result.success,false);
      assert.equal(result.errorCode,'INVALID_RESPONSE');
      assert.equal(result.data,undefined);
    }
    data={...valid,futureFeature:'preserved'};
    const accepted=await aegis.platformInfo();
    assert.equal(accepted.success,true);
    assert.equal(accepted.data.os,'linux');
    assert.equal(accepted.data.futureFeature,'preserved');
  });
});
test('SDK rejects malformed directory results while preserving Unicode paths and backend denials',async()=>{
  let data,denied=false;
  await fixture(async(request,response)=>{
    if(request.url==='/handshake') return json(response,{token:'a'.repeat(64)});
    let body='';for await(const chunk of request) body+=chunk;
    const {id}=JSON.parse(body);
    json(response,denied?{id,success:false,error:'Fixture permission denied'}:{id,success:true,data});
  },async({aegis})=>{
    for(const invalid of [null,[],{data:'/data',config:'/config'}, {data:'/data',config:42,cache:'/cache'}, {data:'',config:'/config',cache:'/cache'}]) {
      data=invalid;
      const result=await aegis.platformDirectories('fixture');
      assert.equal(result.success,false);
      assert.equal(result.errorCode,'INVALID_RESPONSE');
    }
    data={data:'/දත්ත',config:'/config',cache:'/cache'};
    assert.equal((await aegis.platformDirectories('fixture')).data.data,'/දත්ත');
    denied=true;
    const result=await aegis.platformDirectories('fixture');
    assert.equal(result.success,false);
    assert.equal(result.error,'Fixture permission denied');
    assert.equal(result.errorCode,undefined);
  });
});
test('SDK validates nested security flags without treating false as a missing value',async()=>{
  const caps={fsRead:false,fsWrite:false,shellExecution:false,systemInfo:false,crypto:false};
  const valid={engine:'Fixture Core',nonceVerified:false,sandboxEnforced:false,activeCapabilities:caps};
  let data;
  await fixture(async(request,response)=>{
    if(request.url==='/handshake') return json(response,{token:'a'.repeat(64)});
    let body='';for await(const chunk of request) body+=chunk;
    json(response,{id:JSON.parse(body).id,success:true,data});
  },async({aegis})=>{
    for(const invalid of [null,{}, {...valid,engine:42}, {...valid,nonceVerified:'true'}, {...valid,activeCapabilities:[]}, {...valid,activeCapabilities:{...caps,crypto:1}}, {...valid,windowsSessionAvailable:'yes'}, {...valid,activeCapabilities:{...caps,encryptedVault:'enabled'}}]) {
      data=invalid;
      const result=await aegis.getSecurityStatus();
      assert.equal(result.success,false);
      assert.equal(result.errorCode,'INVALID_RESPONSE');
      assert.equal(result.data,undefined);
    }
    data={...valid,windowsSessionAvailable:false,activeCapabilities:{...caps,encryptedVault:false,futureGrant:'kept'}};
    const accepted=await aegis.getSecurityStatus();
    assert.equal(accepted.success,true);
    assert.equal(accepted.data.sandboxEnforced,false);
    assert.equal(accepted.data.activeCapabilities.crypto,false);
    assert.equal(accepted.data.activeCapabilities.futureGrant,'kept');
  });
});
test('SDK requires a confirmed lock acknowledgement before returning vault lock success',async()=>{
  let data;
  await fixture(async(request,response)=>{
    if(request.url==='/handshake') return json(response,{token:'a'.repeat(64)});
    let body='';for await(const chunk of request) body+=chunk;
    json(response,{id:JSON.parse(body).id,success:true,data});
  },async({aegis})=>{
    for(const invalid of [null,{}, {locked:false}, {locked:'true'}]) {
      data=invalid;
      const result=await aegis.vaultSessionLock();
      assert.equal(result.success,false);
      assert.equal(result.errorCode,'INVALID_RESPONSE');
    }
    data={locked:true};
    assert.equal((await aegis.vaultSessionLock()).success,true);
  });
});
test('SDK accepts only valid vault session credentials and positive ordered timeout values',async()=>{
  const valid={sessionToken:'d'.repeat(64),idleTimeoutSeconds:60,absoluteTimeoutSeconds:300};
  let data;
  await fixture(async(request,response)=>{
    if(request.url==='/handshake') return json(response,{token:'a'.repeat(64)});
    let body='';for await(const chunk of request) body+=chunk;
    json(response,{id:JSON.parse(body).id,success:true,data});
  },async({aegis})=>{
    for(const invalid of [null,{}, {...valid,sessionToken:'bad'}, {...valid,idleTimeoutSeconds:0}, {...valid,idleTimeoutSeconds:'60'}, {...valid,absoluteTimeoutSeconds:30}, {...valid,absoluteTimeoutSeconds:300.5}]) {
      data=invalid;
      const result=await aegis.vaultSessionUnlock('fixture.vault','TEST_ONLY_password');
      assert.equal(result.success,false);
      assert.equal(result.errorCode,'INVALID_RESPONSE');
      assert.equal(result.data,undefined);
    }
    data=valid;
    const accepted=await aegis.vaultSessionUnlock('fixture.vault','TEST_ONLY_password');
    assert.equal(accepted.success,true);
    assert.equal(accepted.data.idleTimeoutSeconds,60);
    assert.equal(aegis.getNonce(),'a'.repeat(64),'Vault credentials must not replace the process handshake token');
  });
});
test('SDK rejects unverified or malformed vault plaintext without exposing it as successful data',async()=>{
  let data;
  await fixture(async(request,response)=>{
    if(request.url==='/handshake') return json(response,{token:'a'.repeat(64)});
    let body='';for await(const chunk of request) body+=chunk;
    json(response,{id:JSON.parse(body).id,success:true,data});
  },async({aegis})=>{
    for(const call of [()=>aegis.vaultSessionRead('fixture.vault','d'.repeat(64)),()=>aegis.vaultDecryptRetrieve('fixture.vault','TEST_ONLY_password')]) {
      for(const invalid of [null,{}, {filename:'fixture.vault',decryptedContent:'TEST_ONLY_plaintext',integrityVerified:false}, {filename:'fixture.vault',decryptedContent:42,integrityVerified:true}]) {
        data=invalid;
        const result=await call();
        assert.equal(result.success,false);
        assert.equal(result.errorCode,'INVALID_RESPONSE');
        assert.equal(result.data,undefined);
        assert.equal(JSON.stringify(result).includes('TEST_ONLY_plaintext'),false);
      }
      data={filename:'fixture.vault',decryptedContent:'',integrityVerified:true};
      const accepted=await call();
      assert.equal(accepted.success,true);
      assert.equal(accepted.data.decryptedContent,'');
    }
  });
});
test('SDK validates vault write receipts and leaves backend errors unchanged',async()=>{
  let data,denied=false;
  await fixture(async(request,response)=>{
    if(request.url==='/handshake') return json(response,{token:'a'.repeat(64)});
    let body='';for await(const chunk of request) body+=chunk;
    const {id}=JSON.parse(body);
    json(response,denied?{id,success:false,error:'Fixture write denied'}:{id,success:true,data});
  },async({aegis})=>{
    for(const invalid of [null,{}, {filename:'fixture.vault',status:'stored',envelopeLength:-1}, {filename:'fixture.vault',status:'stored',envelopeLength:'200'}, {filename:'',status:'stored',envelopeLength:200}]) {
      data=invalid;
      const result=await aegis.vaultEncryptStore('fixture','TEST_ONLY_plaintext','TEST_ONLY_password');
      assert.equal(result.success,false);
      assert.equal(result.errorCode,'INVALID_RESPONSE');
    }
    data={filename:'fixture.vault',status:'stored',envelopeLength:200};
    assert.equal((await aegis.vaultEncryptStore('fixture','TEST_ONLY_plaintext','TEST_ONLY_password')).success,true);
    denied=true;
    const result=await aegis.vaultEncryptStore('fixture','TEST_ONLY_plaintext','TEST_ONLY_password');
    assert.equal(result.success,false);
    assert.equal(result.error,'Fixture write denied');
    assert.equal(result.errorCode,undefined);
  });
});
