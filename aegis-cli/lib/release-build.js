import {spawn} from 'node:child_process';
import path from 'node:path';

/** Execute a trusted developer build, selecting only this Cargo build's reported binary. */
export async function executeReleasePlan(plan,{log=text=>process.stdout.write(text)}={}) {
  let coreExecutable;
  for(const command of plan.commands) {
    if(command.useCoreArtifact && !coreExecutable) throw new Error('Cargo did not report an aegis-core executable');
    const executable=command.useCoreArtifact?coreExecutable:command.executable;
    await new Promise((resolve,reject)=>{
      const child=spawn(executable,command.args,{cwd:command.cwd,shell:false,windowsHide:true,
        stdio:command.captureCoreArtifact?['ignore','pipe','pipe']:['ignore','inherit','inherit']});
      let pending='',failure;
      const oversized=()=>{
        failure ||= new Error('Cargo diagnostic line exceeds 1 MiB');
        pending='';child.kill();
      };
      const record=line=>{
        if(failure || !line) return;
        if(Buffer.byteLength(line,'utf8')>1024*1024) {oversized();return;}
        let item;
        try {item=JSON.parse(line);} catch {log(line+'\n');return;}
        if(item.reason==='compiler-artifact' && item.target?.name==='aegis-core'
          && Array.isArray(item.target.kind) && item.target.kind.includes('bin')
          && typeof item.executable==='string' && path.isAbsolute(item.executable)) {
          coreExecutable=item.executable;
        }
        if(item.reason==='compiler-message' && typeof item.message?.rendered==='string') log(item.message.rendered);
      };
      if(command.captureCoreArtifact) {
        child.stdout.setEncoding('utf8');
        child.stdout.on('data',data=>{
          if(failure) return;
          pending+=data;
          let end;
          while((end=pending.indexOf('\n'))!==-1) {
            const line=pending.slice(0,end);pending=pending.slice(end+1);record(line);
            if(failure) return;
          }
          if(Buffer.byteLength(pending,'utf8')>1024*1024) oversized();
        });
        child.stderr.setEncoding('utf8');child.stderr.on('data',log);
      }
      child.once('error',reject);
      child.once('close',code=>{
        if(command.captureCoreArtifact) record(pending);
        if(failure) reject(failure);
        else if(code!==0) reject(new Error(executable+' failed with exit '+code));
        else if(command.captureCoreArtifact && !coreExecutable) reject(new Error('Cargo did not report an aegis-core executable'));
        else resolve();
      });
    });
  }
  return {...plan,coreExecutable};
}
