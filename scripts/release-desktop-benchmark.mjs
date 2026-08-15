import assert from "node:assert/strict";
import { cp, mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { spawn } from "node:child_process";
import path from "node:path";

const repo = path.resolve(import.meta.dirname, "..");
const releaseApp = path.join(repo, "src-tauri", "target", "release-benchmark", "release", "acm-os.exe");
const sampleCount = 7;
const steadyStateDelayMs = 5_000;
const startupBudgetMs = 2_500;
const ramBudgetBytes = 500 * 1024 * 1024;
const temporary = await mkdtemp(path.join(repo, ".release-desktop-benchmark-"));
const seedAppData = path.join(temporary, "seed-app-data");
const vault = path.join(temporary, "vault");
const dateFile = path.join(temporary, "date.txt");
const samples = [];
let activeApp;

try {
  await Promise.all([
    mkdir(seedAppData),
    mkdir(path.join(vault, "Problems"), { recursive: true }),
    mkdir(path.join(vault, "Knowledge"), { recursive: true }),
  ]);
  await writeFile(path.join(vault, "Knowledge", "Segment Tree.md"), "# Segment Tree\n\nRelease benchmark fixture.\n");
  await writeFile(dateFile, "2026-08-14\n");
  activeApp = launch(releaseApp, seedAppData, 19_300);
  await configureSyntheticWorkspace(19_300, activeApp);
  await stopApp(activeApp); activeApp = undefined;
  await rm(path.join(seedAppData, "webview2"), { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });

  for (let index = 0; index < sampleCount; index += 1) {
    const appData = path.join(temporary, `sample-${index + 1}-app-data`);
    await cp(seedAppData, appData, { recursive: true });
    const port = 19_301 + index;
    const startedAt = performance.now();
    activeApp = launch(releaseApp, appData, port);
    await waitForTodayInteractive(port, activeApp, 30_000);
    const startupMs = performance.now() - startedAt;
    await delay(steadyStateDelayMs);
    const memory = await processTreeMemory(activeApp.pid);
    samples.push({ sample: index + 1, startupMs: round(startupMs), workingSetBytes: memory.workingSetBytes,
      workingSetMiB: round(memory.workingSetBytes / 1024 / 1024), processCount: memory.processCount });
    await stopApp(activeApp); activeApp = undefined;
  }

  const startupP95Ms = percentile(samples.map((sample) => sample.startupMs), 0.95);
  const workingSetP95Bytes = percentile(samples.map((sample) => sample.workingSetBytes), 0.95);
  const report = {
    harness: "release-optimized Tauri + isolated desktop-e2e storage/config",
    readiness: "Today route rendered and .today-toolbar present after Loading today plan disappeared",
    memory: "main process plus descendant WebView2 process Working Set after 5 seconds",
    sampleCount, samples,
    p95: { startupMs: startupP95Ms, workingSetBytes: workingSetP95Bytes,
      workingSetMiB: round(workingSetP95Bytes / 1024 / 1024) },
    budgets: { startupMs: startupBudgetMs, workingSetBytes: ramBudgetBytes, workingSetMiB: 500 },
  };
  console.log(JSON.stringify(report, null, 2));
  assert.ok(startupP95Ms <= startupBudgetMs, `Release startup P95 ${startupP95Ms}ms exceeded ${startupBudgetMs}ms`);
  assert.ok(workingSetP95Bytes <= ramBudgetBytes, `Release RAM P95 ${workingSetP95Bytes} bytes exceeded ${ramBudgetBytes} bytes`);
  console.log("Release desktop benchmark passed.");
} finally {
  if (activeApp && activeApp.exitCode === null) await stopApp(activeApp);
  await rm(temporary, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
}

function launch(executable, appData, port, extraEnv = {}) {
  return spawn(executable, [], { cwd: repo, env: { ...process.env, ACM_OS_E2E_ROOT: appData,
    ACM_OS_E2E_DATE_FILE: dateFile, WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS: `--remote-debugging-port=${port}`,
    WEBVIEW2_USER_DATA_FOLDER: path.join(appData, "webview2"),
    ...extraEnv }, stdio: ["ignore", "ignore", "pipe"] });
}

async function configureSyntheticWorkspace(port, child) {
  const deadline = Date.now() + 30_000;
  let socketUrl;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error(`Release app exited before Setup was ready (code ${child.exitCode})`);
    try {
      const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      const page = targets.find((target) => target.type === "page" && target.webSocketDebuggerUrl);
      if (page && await evaluate(page.webSocketDebuggerUrl, `document.body.innerText.includes("Connect your workspace")`)) {
        socketUrl = page.webSocketDebuggerUrl;
        break;
      }
    } catch {}
    await delay(25);
  }
  if (!socketUrl) throw new Error("Release Setup did not become ready within 30000ms");
  const draft = {
    activeVaultPath: vault,
    problemRootPath: path.join(vault, "Problems"),
    knowledgeRootPath: path.join(vault, "Knowledge"),
  };
  const schedule = { monday: 60, tuesday: 60, wednesday: 60, thursday: 60, friday: 60, saturday: 60, sunday: 60 };
  const expression = `window.__TAURI_INTERNALS__.invoke("configure_workspace", { draft: ${JSON.stringify(draft)} }).then(() => window.__TAURI_INTERNALS__.invoke("save_weekly_acm_budget", { schedule: ${JSON.stringify(schedule)} })).then(() => window.__TAURI_INTERNALS__.invoke("today_snapshot", { input: { budgetMinutes: null } })).then(value => ({ ok: true, value }), error => ({ ok: false, error: String(error) }))`;
  const result = await evaluateValue(socketUrl, expression);
  assert.equal(result?.ok, true, `Synthetic workspace configuration failed: ${JSON.stringify(result)}`);
}
async function waitForTodayInteractive(port, child, timeoutMs) {
  const deadline = Date.now() + timeoutMs;
  while (Date.now() < deadline) {
    if (child.exitCode !== null) throw new Error(`Release app exited before Today was interactive (code ${child.exitCode})`);
    try {
      const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`)).json();
      const page = targets.find((target) => target.type === "page" && target.webSocketDebuggerUrl);
      const expression = `location.pathname === "/today" && document.title.startsWith("Today") && !document.body.innerText.includes("Loading today plan...") && Boolean(document.querySelector(".today-toolbar"))`;
      if (page && await evaluate(page.webSocketDebuggerUrl, expression)) return;
    } catch {}
    await delay(25);
  }
  throw new Error(`Today did not become interactive within ${timeoutMs}ms`);
}

async function evaluate(webSocketUrl, expression) {
  return Boolean(await evaluateValue(webSocketUrl, expression));
}

async function evaluateValue(webSocketUrl, expression) {
  const socket = new WebSocket(webSocketUrl);
  await new Promise((resolve, reject) => { socket.addEventListener("open", resolve, { once: true }); socket.addEventListener("error", reject, { once: true }); });
  try {
    const id = 1;
    const result = new Promise((resolve, reject) => {
      const timeout = setTimeout(() => reject(new Error("CDP evaluation timed out")), 2_000);
      socket.addEventListener("message", (event) => { const message = JSON.parse(String(event.data));
        if (message.id !== id) return; clearTimeout(timeout); resolve(message.result?.result?.value); });
    });
    socket.send(JSON.stringify({ id, method: "Runtime.evaluate", params: { expression, awaitPromise: true, returnByValue: true } }));
    return await result;
  } finally { socket.close(); }
}

async function processTreeMemory(rootPid) {
  const command = [
    "$items=Get-CimInstance Win32_Process | Select-Object ProcessId,ParentProcessId,WorkingSetSize;",
    `$pending=@(${rootPid});$seen=@{};$sum=[int64]0;`,
    "while($pending.Count -gt 0){$pidValue=[int]$pending[0];if($pending.Count -eq 1){$pending=@()}else{$pending=$pending[1..($pending.Count-1)]};if($seen.ContainsKey($pidValue)){continue};$seen[$pidValue]=$true;$item=$items|Where-Object ProcessId -eq $pidValue|Select-Object -First 1;if($item){$sum+=[int64]$item.WorkingSetSize;$children=@($items|Where-Object ParentProcessId -eq $pidValue|ForEach-Object ProcessId);$pending+=@($children)}};",
    "[pscustomobject]@{workingSetBytes=$sum;processCount=$seen.Count}|ConvertTo-Json -Compress",
  ].join("");
  return JSON.parse((await capture("powershell.exe", ["-NoProfile", "-Command", command])).trim());
}


async function stopApp(child) {
  if (!child || child.exitCode !== null) return;
  const exit = new Promise((resolve) => child.once("exit", resolve));
  await captureAllowFailure("taskkill.exe", ["/PID", String(child.pid), "/T", "/F"]);
  await Promise.race([exit, delay(5_000)]);
}

async function capture(command, args) {
  return await new Promise((resolve, reject) => {
    const child = spawn(command, args, { cwd: repo, stdio: ["ignore", "pipe", "pipe"] });
    let stdout = "", stderr = "";
    child.stdout.on("data", (chunk) => { stdout += String(chunk); });
    child.stderr.on("data", (chunk) => { stderr += String(chunk); });
    child.on("error", reject);
    child.on("exit", (code) => code === 0 ? resolve(stdout) : reject(new Error(`${command} exited ${code}: ${stderr.trim()}`)));
  });
}

function percentile(values, quantile) { const sorted = [...values].sort((a, b) => a - b); return sorted[Math.ceil(sorted.length * quantile) - 1]; }
function round(value) { return Math.round(value * 100) / 100; }
function delay(milliseconds) { return new Promise((resolve) => setTimeout(resolve, milliseconds)); }
async function captureAllowFailure(command, args) {
  return await new Promise((resolve) => {
    const child = spawn(command, args, { cwd: repo, stdio: ["ignore", "ignore", "ignore"] });
    child.on("error", () => resolve());
    child.on("exit", () => resolve());
  });
}
