import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, readdir, rm, writeFile } from "node:fs/promises";
import { spawn } from "node:child_process";
import path from "node:path";

const repo = path.resolve(import.meta.dirname, "..");
const manual = process.argv.includes("--manual");
const temporary = await mkdtemp(path.join(repo, ".desktop-e2e-"));
const appData = path.join(temporary, "app-data");
const vault = path.join(temporary, "vault");
const problems = path.join(vault, "Problems");
const knowledge = path.join(vault, "Knowledge");
const dateFile = path.join(temporary, "date.txt");
const resultFile = path.join(appData, "desktop-e2e-result.txt");
let app;
let passed = false;
const appStderr = [];

try {
  await Promise.all([mkdir(appData), mkdir(problems, { recursive: true }), mkdir(knowledge, { recursive: true })]);
  await writeFile(path.join(knowledge, "Segment Tree.md"), "# Segment Tree\n\nA real Knowledge Markdown fixture.\n");
  await writeFile(dateFile, "2026-08-11\n");
  process.env.VITE_ACM_OS_DESKTOP_E2E = "1";
  await run(process.execPath, [path.join(repo, "node_modules", "vite", "bin", "vite.js"), "build"], repo);
  delete process.env.VITE_ACM_OS_DESKTOP_E2E;
  await run(process.execPath, [path.join(repo, "node_modules", "@tauri-apps", "cli", "tauri.js"), "build", "--debug", "--no-bundle", "--features", "desktop-e2e", "--config", "src-tauri/tauri.e2e.conf.json"], repo);

  app = launchApp("initial");
  let result = await waitForResult(resultFile, app, 60_000);
  assert.equal(result, "restart", `Desktop E2E initial phase failed: ${result}`);
  await stopApp(app);
  const problemNotes = (await readdir(problems)).filter((name) => name.endsWith(".md"));
  const initialProblemMarkdown = await Promise.all(problemNotes.map((name) => readFile(path.join(problems, name), "utf8")));
  assert.equal(initialProblemMarkdown.some((markdown) => markdown.includes("Fenwick Tree Intent")), false,
    "Accepted intent wrote to Problem Markdown before explicit Safe Patch");
  await writeFile(path.join(knowledge, "Fenwick Tree Intent.md"), "# Fenwick Tree Intent\n\nCreated externally after intent acceptance.\n");
  await rm(resultFile, { force: true });

  app = launchApp("verify-restart");
  result = await waitForResult(resultFile, app, 60_000);
  assert.equal(result, "passed", `Desktop E2E restart phase failed: ${result}`);
  passed = true;
  assert.ok((await readdir(problems)).some((name) => name.endsWith(".md")), "Personal Markdown was not created");
  console.log("Desktop E2E passed: Knowledge discovery/status + accepted-intent explicit Safe Patch restart + weekly budget + date-local override + core loop recall");
  if (manual) {
    console.log("Manual QA window is ready. Close the ACM-OS window when inspection is complete.");
    await new Promise((resolve) => app.once("exit", resolve));
  }
} catch (error) {
  const diagnostic = appStderr.join("").trim();
  throw new Error(`${error.message}${diagnostic ? `\nTauri stderr:\n${diagnostic}` : ""}`, { cause: error });
} finally {
  if (app && app.exitCode === null) {
    await stopApp(app);
  }
  if (passed) {
    try {
      await rm(temporary, { recursive: true, force: true, maxRetries: 5, retryDelay: 100 });
    } catch (cleanupError) {
      console.error(`Desktop E2E cleanup failed: ${cleanupError.message}`);
    }
  } else {
    console.error(`Desktop E2E diagnostics preserved at ${temporary}`);
  }
}

function launchApp(phase) {
  const child = spawn(path.join(repo, "src-tauri", "target", "debug", "acm-os.exe"), [], {
    cwd: repo,
    env: {
      ...process.env,
      ACM_OS_E2E_ROOT: appData,
      ACM_OS_E2E_DATE_FILE: dateFile,
      ACM_OS_E2E_PHASE: phase,
    },
    stdio: ["ignore", "pipe", "pipe"],
  });
  child.stderr.on("data", (chunk) => appStderr.push(String(chunk)));
  return child;
}

async function stopApp(child) {
  if (!child || child.exitCode !== null) return;
  const pid = child.pid;
  const exit = new Promise((resolve) => child.once("exit", () => resolve(true)));
  const exitedNaturally = await Promise.race([exit, delay(250).then(() => false)]);
  if (exitedNaturally || child.exitCode !== null) return;

  if (process.platform === "win32") {
    await run("taskkill.exe", ["/PID", String(pid), "/T", "/F"], repo);
  } else {
    child.kill();
  }
  const exited = child.exitCode !== null || await Promise.race([
    exit,
    delay(5_000).then(() => false),
  ]);
  if (!exited || child.exitCode === null) {
    throw new Error(`Desktop E2E application PID ${pid} did not exit within 5 seconds`);
  }
}

async function run(command, args, cwd) {
  await new Promise((resolve, reject) => {
    const child = spawn(command, args, {
      cwd,
      env: { ...process.env, PATH: `${path.dirname(process.execPath)};${process.env.PATH ?? ""}` },
      stdio: "inherit",
    });
    child.on("error", reject);
    child.on("exit", (code) => code === 0 ? resolve() : reject(new Error(`${command} exited ${code}`)));
  });
}

async function waitForResult(file, processHandle, timeout) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    try {
      return (await readFile(file, "utf8")).trim();
    } catch {}
    if (processHandle.exitCode !== null) {
      throw new Error(`Tauri application exited before CDP was ready (code ${processHandle.exitCode})`);
    }
    await delay(100);
  }
  throw new Error("Desktop E2E result did not become ready");
}

function delay(milliseconds) {
  return new Promise((resolve) => setTimeout(resolve, milliseconds));
}
