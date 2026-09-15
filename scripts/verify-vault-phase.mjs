import { spawn } from "node:child_process";
import { createWriteStream, writeFileSync } from "node:fs";
import path from "node:path";
const phase = process.argv[2];
if (!/^phase-[abc]$/.test(phase ?? "")) throw new Error("phase-a, phase-b or phase-c required");
const commands = [
 ["cargo-test", "cargo", ["test", "--manifest-path", "src-tauri/Cargo.toml", "--workspace"]],
 ["cargo-check", "cargo", ["check", "--manifest-path", "src-tauri/Cargo.toml", "--workspace"]],
 ["frontend-build", process.execPath, ["node_modules/typescript/bin/tsc"]],
 ["vite-build", process.execPath, ["node_modules/vite/bin/vite.js", "build"]],
 ["diff-check", "git", ["diff", "--check"]],
 ["diff-stat", "git", ["diff", "--stat"]],
];
// Cargo operations share build artifacts; keep them sequential. No silent failures.
const results=[];
for(const [name,cmd,args] of commands){
 const log=createWriteStream(path.join("docs/verification/r6-phase2",`${phase}-${name}.log`));
 const code=await new Promise((resolve,reject)=>{const child=spawn(cmd,args,{stdio:["ignore","pipe","pipe"],windowsHide:true});child.stdout.pipe(log,{end:false});child.stderr.pipe(log,{end:false});child.on("error",reject);child.on("close",resolve)});
 log.end();results.push({name,cmd,args,exitCode:code});console.log(`${phase} ${name}: ${code}`);
}
writeFileSync(`docs/verification/r6-phase2/${phase}-results.json`,JSON.stringify(results,null,2)+"\n");
process.exitCode=results.some(r=>r.exitCode!==0)?1:0;
