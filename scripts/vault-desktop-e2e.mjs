import assert from 'node:assert/strict';
import {spawn} from 'node:child_process';
import {mkdir,mkdtemp,readFile,writeFile,rm,copyFile,readdir} from 'node:fs/promises';
import {createWriteStream} from 'node:fs';
import {createHash} from 'node:crypto';
import path from 'node:path';
const repo=process.cwd();const evidence=path.join(repo,'docs/verification/r6-phase2');
const temporary=await mkdtemp(path.join(repo,'.desktop-e2e-r6-'));const appData=path.join(temporary,'app-data');
for(const root of [appData,'vault','vault-b','vault-c'].map(p=>path.isAbsolute(p)?p:path.join(temporary,p)))await mkdir(root,{recursive:true});
await writeFile(path.join(appData,'legacy-sentinel.txt'),'Legacy data must stay byte-for-byte untouched.');
const sentinelBefore=await readFile(path.join(appData,'legacy-sentinel.txt'));
let child;const delay=ms=>new Promise(r=>setTimeout(r,ms));
async function command(label,exe,args,extra={}){const log=createWriteStream(path.join(evidence,`${label}.log`));const code=await new Promise((resolve,reject)=>{const p=spawn(exe,args,{cwd:repo,env:{...process.env,...extra},windowsHide:true,stdio:['ignore','pipe','pipe']});p.stdout.pipe(log,{end:false});p.stderr.pipe(log,{end:false});p.on('error',reject);p.on('close',resolve);});await new Promise(resolve=>log.end(resolve));assert.equal(code,0,label);console.log(`${label}: PASS`);}
async function stop(){if(!child||child.exitCode!==null)return;const processId=child.pid;await command('desktop-stop','taskkill.exe',['/PID',String(processId),'/T','/F']);await delay(500);}
async function phase(name){
 await rm(path.join(appData,'desktop-e2e-result.txt'),{force:true});
 const target=process.env.CARGO_TARGET_DIR??path.join(repo,'src-tauri/target');
 child=spawn(path.join(target,'debug/acm-os.exe'),[],{cwd:repo,env:{...process.env,ACM_OS_E2E_ROOT:appData,ACM_OS_E2E_PHASE:name,WEBVIEW2_USER_DATA_FOLDER:path.join(temporary,'webview')},windowsHide:true,stdio:['ignore','pipe','pipe']});
 const stderr=createWriteStream(path.join(evidence,`desktop-${name}-stderr.log`));child.stderr.pipe(stderr);
 for(let i=0;i<1800;i++){
   const result=await readFile(path.join(appData,'desktop-e2e-result.txt'),'utf8').catch(()=>null);
   if(result){await writeFile(path.join(evidence,`desktop-${name}-result.txt`),result);return result;}
   if(child.exitCode!==null)throw Error(`Desktop exited ${child.exitCode}`);
   await delay(100);
 }
 throw Error(`Desktop timeout; stage: ${await readFile(path.join(appData,'desktop-e2e-stage.txt'),'utf8').catch(()=>'?')}`);
}
try{
 await command('desktop-vite-build',process.execPath,['node_modules/vite/bin/vite.js','build'],{VITE_ACM_OS_VAULT_E2E:'1'});
 await command('desktop-tauri-build',process.execPath,['node_modules/@tauri-apps/cli/tauri.js','build','--debug','--no-bundle','--features','desktop-e2e','--config','src-tauri/tauri.e2e.conf.json']);
 assert.equal(await phase('initial'),'restart');await stop();assert.equal(await phase('verify-restart'),'passed');await stop();
 assert.deepEqual(await readFile(path.join(appData,'legacy-sentinel.txt')),sentinelBefore);
 const registry=JSON.parse(await readFile(path.join(appData,'vault-manager.json'),'utf8'));assert.equal(registry.vaults.length,2);assert.equal(registry.vaults.find(v=>v.id===registry.current_id).name,'B');
 for(const v of registry.vaults){const bytes=await readFile(v.databasePath);assert.equal(bytes.subarray(0,15).toString(),'SQLite format 3');}
 assert.ok((await readdir(path.join(temporary,'vault-c'))).some(n=>n.startsWith('.acm-os-deleted-')));
 await writeFile(path.join(evidence,'desktop-results.json'),JSON.stringify({result:'PASS',temporary,createFromUi:true,openFromUi:true,deleteFromUi:true,crossVaultWriteBlocked:true,restartPersistence:true,sqliteFiles:registry.vaults.map(v=>({name:v.name,path:v.databasePath})),sentinelSha256:createHash('sha256').update(sentinelBefore).digest('hex')},null,2));
 console.log('REAL TAURI VAULT UI AND RESTART: PASS');
}catch(e){await writeFile(path.join(evidence,'desktop-failure.txt'),String(e.stack??e));throw e;}
finally{await stop();console.log(`Test-only data retained: ${temporary}`);}
