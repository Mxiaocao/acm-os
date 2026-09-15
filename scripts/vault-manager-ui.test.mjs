import assert from "node:assert/strict";
import test from "node:test";
import { JSDOM } from "jsdom";
import React, { act } from "react";


import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
globalThis.IS_REACT_ACT_ENVIRONMENT=true;
const bootDom=new JSDOM('<div/>');globalThis.window=bootDom.window;globalThis.document=bootDom.window.document;
const {createRoot}=await import('react-dom/client');
const {VaultManagerPanel}=await import('../src/app/VaultManagerPanel.tsx');
const scoped=await import('../src/ipc/invoke.ts');
test.after(()=>bootDom.window.close());
const a={id:'a',name:'A',rootPath:'C:/A',databasePath:'C:/A/.acm-os/system-facts.sqlite3',createdAtUtc:'2026-09-15T00:00:00Z',isCurrent:true};
const b={...a,id:'b',name:'B',rootPath:'C:/B',isCurrent:false};
const ca={vaultId:'a',epoch:'a1'};
const initial=()=>({vaults:[a,b],current:{state:'ready',vault:a},context:ca});
async function view(ipc){
 const dom=new JSDOM('<div id="root"></div>',{url:'https://acm-os.test/settings'});
 const previous={};for(const key of ['window','document','navigator','HTMLElement','Event']){previous[key]=Object.getOwnPropertyDescriptor(globalThis,key);Object.defineProperty(globalThis,key,{configurable:true,value:key==='window'?dom.window:dom.window[key]});}
 dom.window.HTMLElement.prototype.attachEvent=()=>{};dom.window.HTMLElement.prototype.detachEvent=()=>{};
 mockIPC(ipc);const root=createRoot(dom.window.document.getElementById('root'));await act(async()=>root.render(React.createElement(VaultManagerPanel)));
 return {document:dom.window.document,window:dom.window,cleanup:async()=>{await act(async()=>root.unmount());clearMocks();scoped.setVaultContext(null);dom.window.close();for(const [key,d]of Object.entries(previous)){if(d)Object.defineProperty(globalThis,key,d);else delete globalThis[key];}}};
}
function button(v,text){const result=[...v.document.querySelectorAll('button')].find(e=>e.textContent===text);assert.ok(result,text);return result;}
async function type(v,id,value){const el=v.document.getElementById(id);await act(async()=>{Object.getOwnPropertyDescriptor(v.window.HTMLInputElement.prototype,'value').set.call(el,value);el.dispatchEvent(new v.window.Event('input',{bubbles:true}));el.dispatchEvent(new v.window.Event('change',{bubbles:true}));});}
test('UI lists current Vault and creates through typed IPC',async()=>{
 let status=initial();const calls=[];const v=await view((cmd,args)=>{calls.push([cmd,args]);if(cmd==='vault_manager_status')return status;if(cmd==='create_vault'){assert.deepEqual(args.input,{name:'C',rootPath:'C:/C'});const c={...b,id:'c',name:'C',rootPath:'C:/C'};status={...status,vaults:[...status.vaults,c]};return c;}throw Error(cmd)});
 try{assert.match(v.document.querySelector('[data-testid=current-vault]').textContent,/A.*C:\/A/s);assert.equal(button(v,'Delete / 删除 A').disabled,true);await type(v,'vault-name','C');await type(v,'vault-path','C:/C');await act(async()=>v.document.querySelector('form').dispatchEvent(new v.window.Event('submit',{bubbles:true,cancelable:true})));assert.equal(calls.filter(([c])=>c==='create_vault').length,1);assert.ok(v.document.querySelector('[data-vault-id=c]'));}finally{await v.cleanup()}
});
test('Delete needs preview and exact second confirmation; cancellation writes nothing',async()=>{
 const calls=[];let status=initial();const p={vaultId:'b',name:'B',rootPath:'C:/B',retainedDataPath:'C:/B/.acm-os-deleted-b',confirmationText:'DELETE B',token:'token'};
 const v=await view((cmd,args)=>{calls.push([cmd,args]);if(cmd==='vault_manager_status')return status;if(cmd==='preview_delete_vault')return p;if(cmd==='delete_vault'){assert.deepEqual(args.input,{vaultId:'b',token:'token',confirmation:'DELETE B'});status={...status,vaults:[a]};return status;}throw Error(cmd)});
 try{await act(async()=>button(v,'Delete / 删除 B').click());assert.equal(calls.some(([c])=>c==='delete_vault'),false);assert.equal(button(v,'Confirm Delete / 确认删除').disabled,true);await type(v,'vault-delete-confirmation','DELETE A');assert.equal(button(v,'Confirm Delete / 确认删除').disabled,true);await act(async()=>button(v,'Cancel / 取消').click());assert.equal(calls.some(([c])=>c==='delete_vault'),false);await act(async()=>button(v,'Delete / 删除 B').click());await type(v,'vault-delete-confirmation','DELETE B');await act(async()=>button(v,'Confirm Delete / 确认删除').click());assert.equal(v.document.querySelector('[data-vault-id=b]'),null);}finally{await v.cleanup()}
});
test('Open revokes old context and requests workspace replacement',async()=>{
 const v=await view((cmd,args)=>{if(cmd==='vault_manager_status')return initial();if(cmd==='open_vault'){assert.deepEqual(args.input,{vaultId:'b'});return{vaults:[{...a,isCurrent:false},{...b,isCurrent:true}],current:{state:'ready',vault:b},context:{vaultId:'b',epoch:'b1'}};}throw Error(cmd)});
 try{scoped.setVaultContext(ca);let changed=0;v.window.addEventListener('vault-context-changed',()=>changed++);await act(async()=>button(v,'Open / 打开 B').click());assert.equal(changed,1);assert.equal(scoped.getVaultContext(),null);await assert.rejects(scoped.invoke('create_custom_reward',{input:{name:'old',coinCost:1}}),e=>e.code==='no_active_vault');}finally{await v.cleanup()}
});
test('Late business response is rejected after a binding change',async()=>{
 let resolve;let args;const v=await view((cmd,input)=>{if(cmd==='vault_manager_status')return initial();args=input;return new Promise(r=>resolve=r)});
 try{scoped.setVaultContext(ca);const pending=scoped.invoke('contest_shelf');const rejected=assert.rejects(pending,e=>e.code==='stale_vault_context');assert.deepEqual(args.vaultContext,ca);scoped.setVaultContext({vaultId:'b',epoch:'b1'});resolve([{title:'Contest A'}]);await rejected;}finally{await v.cleanup()}
});
test('Backend errors are shown without optimistic create/delete success',async()=>{
 const v=await view(cmd=>{if(cmd==='vault_manager_status')return initial();throw {code:'vault_unavailable'}});
 try{await act(async()=>button(v,'Open / 打开 B').click());assert.match(v.document.querySelector('[role=alert]').textContent,/不可用/);assert.ok(v.document.querySelector('[data-vault-id=b]'));}finally{await v.cleanup()}
});
