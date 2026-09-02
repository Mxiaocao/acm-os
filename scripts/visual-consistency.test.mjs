import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";
import test from "node:test";

const cssUrl = new URL("../src/app/app.css", import.meta.url);
const shellsUrl = new URL("../src/app/shells.tsx", import.meta.url);

function luminance(hex) {
  const rgb = hex.match(/[0-9a-f]{2}/gi).map((part) => Number.parseInt(part, 16) / 255);
  const linear = rgb.map((channel) => (channel <= 0.03928 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4));
  return 0.2126 * linear[0] + 0.7152 * linear[1] + 0.0722 * linear[2];
}

test("M9-D defines a complete shared action and status vocabulary", async () => {
  const [css, shells] = await Promise.all([readFile(cssUrl, "utf8"), readFile(shellsUrl, "utf8")]);
  for (const selector of [".primary-action", ".secondary-action", ".danger-action", ".button-row", ".action-row", ".error-copy", ".status-message", ".empty-state"]) {
    assert.match(css, new RegExp(selector.replace(/[.]/g, "\\.")));
  }
  assert.match(shells, /className="danger-action"/);
  assert.match(shells, /className="button-row"/);
  assert.match(shells, /className="error-copy"/);
});

test("critical action colors meet WCAG AA contrast against their surfaces", async () => {
  const css = await readFile(cssUrl, "utf8");
  const accent = css.match(/--accent:\s*(#[0-9a-f]{6})/i)?.[1];
  const danger = css.match(/--danger:\s*(#[0-9a-f]{6})/i)?.[1];
  assert.ok(accent && danger);
  const white = luminance("#ffffff");
  for (const foreground of [accent, danger]) {
    const ratio = (white + 0.05) / (luminance(foreground) + 0.05);
    assert.ok(ratio >= 4.5, `${foreground} contrast ratio ${ratio.toFixed(2)} is below 4.5:1`);
  }
  assert.match(css, /--danger-surface:\s*#fff4f5/i);
  assert.match(css, /--success-surface:\s*#f1faf4/i);
});

test("responsive action rows remain usable at narrow viewports", async () => {
  const css = await readFile(cssUrl, "utf8");
  assert.match(css, /@media\s*\(max-width:\s*760px\)[\s\S]*\.button-row, \.action-row[\s\S]*\.button-row > button, \.action-row > button \{\s*width:\s*100%/);
  assert.match(css, /@media\s*\(max-width:\s*560px\)[\s\S]*\.gate-panel, \.content-panel, \.empty-state \{\s*padding:\s*18px/);
});

test("programmatically focused route headings keep a content-sized focus ring", async () => {
  const css = await readFile(cssUrl, "utf8");
  assert.match(css, /\.page-header h1\s*\{[\s\S]*?width:\s*fit-content;[\s\S]*?max-width:\s*100%;/);
  assert.match(css, /\.page-header h1:focus-visible\s*\{[\s\S]*?outline:\s*3px solid var\(--focus\);/);
});

test("fullscreen normal pages share one centered content rail", async () => {
  const css = await readFile(cssUrl, "utf8");
  assert.match(css, /\.normal-content\s*\{[\s\S]*?width:\s*100%;[\s\S]*?max-width:\s*1280px;[\s\S]*?justify-self:\s*center;/);
  assert.match(css, /\.content-panel\s*\{[\s\S]*?width:\s*100%;[\s\S]*?max-width:\s*none;/);
  assert.match(css, /\.contest-import-form,\s*\.manual-import-panel\s*\{[\s\S]*?width:\s*100%;[\s\S]*?max-width:\s*none;/);
});

test("semantic detail lists keep definition grids separate from full-width item lists", async () => {
  const css = await readFile(cssUrl, "utf8");
  assert.match(css, /\.detail-list\s*\{[\s\S]*?grid-template-columns:\s*minmax\(150px,\s*0\.35fr\)\s+minmax\(0,\s*1fr\);/);
  assert.match(css, /ul\.detail-list\s*\{[\s\S]*?grid-template-columns:\s*minmax\(0,\s*1fr\);/);
  assert.match(css, /ul\.detail-list li\s*\{[\s\S]*?width:\s*100%;[\s\S]*?min-width:\s*0;/);
  assert.match(css, /ul\.detail-list li > div\s*\{\s*min-width:\s*0;\s*overflow-wrap:\s*anywhere;/);
  assert.match(css, /\.modal-backdrop > div > h2\s*\{[\s\S]*?overflow-wrap:\s*anywhere;/);
});

test("sanitized Codeforces statements retain readable metadata and section hierarchy", async () => {
  const css = await readFile(cssUrl, "utf8");
  assert.match(css, /\.statement-view \.problem-statement \.header\s*\{[\s\S]*?border-bottom:\s*1px solid var\(--line\);/);
  assert.match(css, /\.statement-view \.problem-statement \.title\s*\{[\s\S]*?font-size:\s*1\.35rem;/);
  assert.match(css, /\.statement-view \.problem-statement \.time-limit,[\s\S]*?grid-template-columns:\s*minmax\(150px,\s*\.4fr\)\s+minmax\(0,\s*1fr\);/);
  assert.match(css, /\.statement-view \.problem-statement \.section-title\s*\{[\s\S]*?font-weight:\s*760;/);
});

test("Contest taxonomy filters stay peer-level and create chips wrap with their options", async () => {
  const [css, shells] = await Promise.all([readFile(cssUrl, "utf8"), readFile(shellsUrl, "utf8")]);
  assert.equal(shells.match(/className="taxonomy-filter-group"/g)?.length, 3);
  assert.equal(shells.match(/className="filter-option filter-option--create"/g)?.length, 3);
  assert.equal(shells.match(/className="text-button taxonomy-manage-trigger"/g)?.length, 3);
  assert.doesNotMatch(shells, /contest-library-create-panel/);
  assert.doesNotMatch(shells, /management-list|management-list__row|className="inline-form"/);
  assert.match(css, /\.filter-options\s*\{\s*display:\s*flex;\s*flex-wrap:\s*wrap;/);
  assert.match(css, /\.taxonomy-filter-group__header\s*\{[^}]*justify-content:\s*space-between;/);
  assert.match(css, /\.filter-option--create\s*\{[\s\S]*?min-width:\s*40px;[\s\S]*?border-style:\s*dashed;/);
  assert.doesNotMatch(css, /\.filter-options\s*\{[^}]*overflow-x:/);
});

test("Contest taxonomy management reuses chips in one scoped dialog instead of persistent row actions", async () => {
  const [css, shells] = await Promise.all([readFile(cssUrl, "utf8"), readFile(shellsUrl, "utf8")]);
  assert.match(shells, /className="contest-taxonomy-management-dialog"/);
  assert.match(shells, /className="taxonomy-management-options"/);
  assert.match(shells, /managedFamilyId === item\.familyId \? "taxonomy-management-option taxonomy-management-option--selected"/);
  assert.match(css, /\.taxonomy-management-options\s*\{\s*display:\s*flex;\s*flex-wrap:\s*wrap;/);
  assert.match(css, /\.taxonomy-management-option--selected\s*\{[^}]*border-color:\s*var\(--accent\)/);
  assert.doesNotMatch(css, /\.management-list(?:__row)?\s*\{/);
});

test("Contest safe-delete dialogs use one structured destructive hierarchy", async () => {
  const [css, shells] = await Promise.all([readFile(cssUrl, "utf8"), readFile(shellsUrl, "utf8")]);
  assert.match(shells, /function TaxonomyDeleteDialog/);
  assert.match(shells, /className="taxonomy-delete-current"/);
  assert.match(shells, /className="taxonomy-delete-section taxonomy-delete-replacement"/);
  assert.match(shells, /className="taxonomy-delete-risk"/);
  assert.match(shells, /className="action-row taxonomy-delete-footer"/);
  assert.match(css, /\.taxonomy-delete-footer\s*\{[^}]*justify-content:\s*flex-end;/);
  assert.match(css, /\.taxonomy-delete-risk\s*\{[^}]*border-left:\s*3px solid var\(--danger\);/);
  assert.doesNotMatch(shells, /deleteFamilyTarget !== null \? <div className="modal-backdrop"><div className="content-panel"/);
});

test("Contest cabinet switches only between full and compact column presentation", async () => {
  const [css, shells] = await Promise.all([readFile(cssUrl, "utf8"), readFile(shellsUrl, "utf8")]);
  assert.match(css, /\.contest-cabinet-prototype\s*\{[\s\S]*?container-name:\s*contest-cabinet-presentation;[\s\S]*?container-type:\s*inline-size;/);
  assert.match(css, /\.contest-book-slot\[data-compact-active="false"\]\s*\{\s*display:\s*none;/);
  assert.match(css, /@container\s+contest-cabinet-presentation\s*\(min-width:\s*950px\)[\s\S]*?data-compact-active="false"\][\s\S]*?display:\s*block;[\s\S]*?\.contest-cabinet-pager\s*\{\s*display:\s*none;/);
  assert.match(shells, /data-logical-column=\{index \+ 1\}/);
  assert.match(shells, /aria-label=\{t\("contest\.compactNav"\)\}/);
  assert.doesNotMatch(shells, /tier === 1 && index === 0/);
});
