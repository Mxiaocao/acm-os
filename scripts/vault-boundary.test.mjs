import assert from 'node:assert/strict';
import test from 'node:test';
import {readFileSync} from 'node:fs';
const ipc=readFileSync('src-tauri/src/ipc.rs','utf8');
test('Every database-backed Tauri command requires a Vault lease and context',()=>{
 const commands=[...ipc.matchAll(/#\[tauri::command\]\s*pub (?:async )?fn (\w+)\(([\s\S]*?)\) -> [^{]*\{([\s\S]*?)(?=\n#\[|\nfn |\npub |$)/g)].filter(m=>m[2].includes('DesktopVaultManager'));
 assert.equal(commands.length,121,'Audit command coverage when IPC surface changes');
 for(const [,name,args,body] of commands){assert.match(args,/vault_context: acm_os_infrastructure::vault_manager::VaultContext/,name);assert.match(body,/manager\.database\(&vault_context\)\.await/,name);}
 assert.doesNotMatch(ipc,/tauri::State<[^\n]*DatabaseRuntime/);
 const boot=readFileSync('src-tauri/src/lib.rs','utf8');assert.doesNotMatch(boot,/app\.manage\(database\)|start_database\(/);
});
test('Runtime switching fully reloads the WebView and revokes the old frontend context',()=>{
 const bridge=readFileSync('src/ipc/vault-manager.ts','utf8');assert.match(bridge,/setVaultContext\(null\)/);
 assert.match(readFileSync('src/app/App.tsx','utf8'),/window\.location\.reload\(\)/);
});
