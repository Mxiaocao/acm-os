import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

test("all native interactive controls share the visible focus indicator", async () => {
  const css = await readFile(new URL("../src/app/app.css", import.meta.url), "utf8");
  assert.match(
    css,
    /button:focus-visible,\s*a:focus-visible,\s*input:focus-visible,\s*textarea:focus-visible,\s*select:focus-visible\s*\{/,
  );
  assert.match(css, /outline:\s*3px\s+solid\s+var\(--focus\)/);
  assert.match(css, /outline-offset:\s*3px/);
});

test("reduced-motion users are not forced into animated transitions", async () => {
  const css = await readFile(new URL("../src/app/app.css", import.meta.url), "utf8");
  assert.match(css, /@media\s*\(prefers-reduced-motion:\s*reduce\)/);
  assert.match(css, /animation-duration:\s*0\.01ms\s*!important/);
  assert.match(css, /transition-duration:\s*0\.01ms\s*!important/);
  assert.match(css, /scroll-behavior:\s*auto\s*!important/);
});

test("core layout has narrow-viewport fallbacks for zoomed keyboard use", async () => {
  const css = await readFile(new URL("../src/app/app.css", import.meta.url), "utf8");
  assert.match(css, /@media\s*\(max-width:\s*760px\)/);
  assert.match(css, /\.normal-shell\s*\{\s*grid-template-columns:\s*1fr;/);
  assert.match(css, /\.today-entry\s*\{\s*grid-template-columns:\s*34px\s+minmax\(0,\s*1fr\);/);
  assert.match(css, /@media\s*\(max-width:\s*560px\)/);
  assert.match(css, /\.contest-facts-list\s+li\s*\{\s*grid-template-columns:\s*1fr;/);
});

test("inactive Compact cabinet columns stay out of the interactive accessibility set", async () => {
  const css = await readFile(new URL("../src/app/app.css", import.meta.url), "utf8");
  assert.match(
    css,
    /\.contest-book-slot\[data-compact-active="false"\]\s*\{\s*display:\s*none;/,
  );
  assert.match(
    css,
    /@container\s+contest-cabinet-presentation\s*\(min-width:\s*950px\)[\s\S]*?\.contest-book-slot\[data-compact-active="false"\]\s*\{\s*display:\s*block;/,
  );
});

test("Contest taxonomy destructive workflow exposes labelled dialog and contained focus states", async () => {
  const shells = await readFile(new URL("../src/app/shells.tsx", import.meta.url), "utf8");
  assert.match(shells, /aria-describedby=\{`\$\{descriptionId\} \$\{riskId\}/);
  assert.match(shells, /aria-labelledby=\{titleId\}/);
  assert.match(shells, /aria-modal="true"/);
  assert.match(shells, /role="dialog"/);
  assert.match(shells, /if \(!busy\) onCancel\(\)/);
  assert.match(shells, /select:not\(\[disabled\]\), button:not\(\[disabled\]\)/);
  assert.match(shells, /aria-live="assertive"[\s\S]*?role="alert"/);
});

test("Contest Add panel keeps native disclosure and one labelled live status region", async () => {
  const shells = await readFile(new URL("../src/app/shells.tsx", import.meta.url), "utf8");
  assert.match(shells, /<section aria-labelledby="contest-add-heading" className="content-panel contest-add-panel">/);
  assert.match(shells, /<h2 id="contest-add-heading">\{t\("contest\.addContest"\)\}<\/h2>/);
  assert.match(shells, /<label>\{t\("contest\.url"\)\}<input[^>]*disabled=\{importing\}/);
  assert.match(shells, /<details className="manual-import-panel"><summary>\{t\("contest\.manualImport"\)\}<\/summary>/);
  assert.match(shells, /aria-live="polite" className="system-caption contest-add-panel__status"/);
});

test("Tauri runtime configurations keep a restrictive content security policy", async () => {
  const config = JSON.parse(await readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"));
  const e2eConfig = JSON.parse(await readFile(new URL("../src-tauri/tauri.e2e.conf.json", import.meta.url), "utf8"));
  for (const csp of [config.app.security.csp, e2eConfig.app.security.csp]) {
    assert.equal(typeof csp, "string");
    assert.match(csp, /default-src 'self'/);
    assert.match(csp, /script-src 'self'/);
    assert.match(csp, /object-src 'none'/);
    assert.match(csp, /frame-ancestors 'none'/);
    assert.doesNotMatch(csp, /csp-null-placeholder/);
  }
});
