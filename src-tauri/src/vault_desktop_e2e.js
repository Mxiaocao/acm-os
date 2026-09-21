// Bundled only by the dedicated desktop test build. All manager mutations use real UI controls.
import { invoke } from '@tauri-apps/api/core';
const wait=ms=>new Promise(r=>setTimeout(r,ms));
async function until(fn,label){for(let i=0;i<300;i++){const v=await fn();if(v)return v;await wait(100);}throw Error(`Timed out: ${label}; ${document.body.innerText}`);}
function assert(value,message){if(!value)throw Error(message);}
async function button(text){return until(()=>[...document.querySelectorAll('button')].find(b=>b.textContent===text&&!b.disabled),text);}
async function click(text){(await button(text)).click();await wait(80);}
async function fill(id,value){const el=await until(()=>document.getElementById(id),id);Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set.call(el,value);el.dispatchEvent(new Event('input',{bubbles:true}));el.dispatchEvent(new Event('change',{bubbles:true}));await wait(50);}
const status=()=>invoke('vault_manager_status');
async function create(name,path){const disclosure=await until(()=>document.querySelector(".vault-create"),"create form");if(!disclosure.open)disclosure.querySelector("summary").click();await fill('vault-name',name);await fill('vault-path',path);await click('创建 Vault');await until(async()=> (await status()).vaults.find(v=>v.name===name),`created ${name}`);await until(()=>!document.getElementById('vault-name').value,'create settled');}
async function switchTo(name,next){sessionStorage.setItem('r6-step',next);await click(`打开 ${name}`);}
async function rejects(fn,code){try{await fn();}catch(e){assert(String(typeof e==='object'?e.code: e).includes(code),`Expected ${code}, got ${JSON.stringify(e)}`);return;}throw Error(`Expected rejection: ${code}`);}
async function manual(context,id,title,problem){return invoke('import_manual_codeforces_contest',{vaultContext:context,input:{contestId:id,title,sourceUrl:`https://codeforces.com/contest/${id}`,startsAtUtc:null,problems:[{index:'A',title:problem,sourceUrl:`https://codeforces.com/contest/${id}/problem/A`,statementText:'Real test statement'}]}});}
async function run(){
 const env=await invoke('desktop_e2e_context');
 await until(()=>document.querySelector('[data-testid="vault-manager"]'),'manager UI');
 let s=await status();
 if(env.phase==='verify-restart'){
   assert(s.current.state==='ready'&&s.current.vault.name==='B','restart current B');
   const contests=await invoke('contest_shelf',{vaultContext:s.context});assert(contests.length===1&&contests[0].title==='Contest B','restart B contest');
   await invoke('desktop_e2e_finish',{input:{result:'passed'}});return;
 }
 const step=sessionStorage.getItem('r6-step')??'create';
 await invoke('desktop_e2e_log',{input:{stage:`r6-${step}`}});
 if(step==='create'){
   assert(s.current.state==='none','initial no Vault');await create('A',env.vault);await create('B',`${env.vault}-b`);await create('C',`${env.vault}-c`);
   s=await status();assert(new Set(s.vaults.map(v=>v.databasePath)).size===3,'independent database paths');
   await switchTo('A','a');return;
 }
 if(step==='a'){
   assert(s.current.vault.name==='A','UI opened A');sessionStorage.setItem('r6-old-a',JSON.stringify(s.context));
   await manual(s.context,88001,'Contest A','Problem A');await invoke('activate_reward',{vaultContext:s.context});
   await invoke('create_custom_reward',{vaultContext:s.context,input:{name:'A only',coinCost:1}});
   await switchTo('B','b');return;
 }
 if(step==='b'){
   assert(s.current.vault.name==='B','UI opened B');assert((await invoke('contest_shelf',{vaultContext:s.context})).length===0,'Contest A absent in B');
   assert((await invoke('list_custom_rewards',{vaultContext:s.context})).length===0,'Reward A absent in B');
   const balance=await invoke('reward_account_summary',{vaultContext:s.context});assert(balance.xp===0&&balance.coin===0,'B XP/Coin untouched');
   await manual(s.context,88002,'Contest B','Problem B');
   const ca=JSON.parse(sessionStorage.getItem('r6-old-a'));
   await rejects(()=>invoke('create_custom_reward',{vaultContext:ca,input:{name:'must not leak',coinCost:1}}),'stale_vault_context');
   try{await invoke('create_custom_reward',{input:{name:'missing context',coinCost:1}});throw Error('MISSING_CONTEXT_ACCEPTED');}catch(e){if(String(e).includes('MISSING_CONTEXT_ACCEPTED'))throw e;assert(/vaultContext|vault_context/.test(String(e)),`Missing-context rejection must come from IPC deserialization: ${e}`);}
   await rejects(()=>invoke('preview_delete_vault',{input:{vaultId:s.current.vault.id}}),'current_vault_delete_forbidden');
   await click('删除 C');await until(()=>document.getElementById('vault-delete-confirmation'),'delete confirmation');
   await fill('vault-delete-confirmation','DELETE wrong');assert([...document.querySelectorAll('button')].find(b=>b.textContent==='确认删除').disabled,'wrong confirmation disabled');
   await fill('vault-delete-confirmation','DELETE C');await click('确认删除');
   await until(async()=>!(await status()).vaults.some(v=>v.name==='C'),'C removed');
   await switchTo('A','a2');return;
 }
 if(step==='a2'){
   const contests=await invoke('contest_shelf',{vaultContext:s.context});assert(contests.length===1&&contests[0].title==='Contest A','A still isolated');
   const problems=await invoke('lightweight_problems',{vaultContext:s.context});assert(!problems.some(p=>p.title==='Problem B'),'Problem B absent in A');
   await rejects(()=>invoke('create_custom_reward',{vaultContext:JSON.parse(sessionStorage.getItem('r6-old-a')),input:{name:'old A epoch',coinCost:1}}),'stale_vault_context');
   await switchTo('B','b2');return;
 }
 if(step==='b2'){
   assert(s.current.vault.name==='B','final B');assert((await invoke('list_custom_rewards',{vaultContext:s.context})).length===0,'stale writes never leaked');
   await invoke('desktop_e2e_finish',{input:{result:'restart'}});return;
 }
 throw Error(`unknown step ${step}`);
}
run().catch(async error=>{await invoke('desktop_e2e_finish',{input:{result:`failed: ${error.stack??String(error)}; UI: ${document.body.innerText}`}});});
