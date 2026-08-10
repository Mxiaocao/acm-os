import assert from "node:assert/strict";
import test from "node:test";

import {
  frontendAuthorityViolations,
  frontendDependencyViolations,
  rustAuthorityViolations,
  rustDependencyViolations,
} from "./check-boundaries.mjs";

const validRustPackages = () => [
  { name: "acm-os-domain", dependencies: [] },
  {
    name: "acm-os-application",
    dependencies: [{ name: "acm-os-domain", kind: null, path: "local/domain" }],
  },
  {
    name: "acm-os-infrastructure",
    dependencies: [
      { name: "acm-os-application", kind: null, path: "local/application" },
      { name: "acm-os-domain", kind: null, path: "local/domain" },
      { name: "reqwest", kind: null },
      { name: "rustls", kind: null },
      { name: "same-file", kind: null },
      { name: "serde", kind: null },
      { name: "serde_json", kind: null },
      { name: "sha2", kind: null },
      { name: "sqlx", kind: null },
      { name: "tokio", kind: null },
      { name: "tempfile", kind: "dev" },
      { name: "tokio", kind: "dev" },
    ],
  },
  {
    name: "acm-os",
    dependencies: [
      { name: "acm-os-application", kind: null, path: "local/application" },
      { name: "acm-os-domain", kind: null, path: "local/domain" },
      { name: "acm-os-infrastructure", kind: null, path: "local/infrastructure" },
      { name: "serde", kind: null },
      { name: "serde_json", kind: "dev" },
      { name: "tauri", kind: null },
      { name: "tauri-build", kind: "build" },
    ],
  },
];

const validFrontendPackage = () => ({
  dependencies: {
    "@tauri-apps/api": "^2",
    react: "^19.1.0",
    "react-dom": "^19.1.0",
  },
  devDependencies: {
    "@tauri-apps/cli": "^2",
    "@types/react": "^19.1.8",
    "@types/react-dom": "^19.1.6",
    "@vitejs/plugin-react": "^6.0.2",
    jsdom: "^30.0.1",
    tsx: "^4.23.11",
    typescript: "~6.0.3",
    vite: "^8.0.16",
  },
});

test("accepts the current explicitly allowed dependency graph", () => {
  assert.deepEqual(rustDependencyViolations(validRustPackages()), []);
  assert.deepEqual(frontendDependencyViolations(validFrontendPackage()), []);
});

test("rejects alternate persistence, HTTP, and platform crates in Domain", () => {
  for (const dependency of ["rusqlite", "ureq", "windows"]) {
    const packages = validRustPackages();
    packages[0].dependencies.push({ name: dependency, kind: null });
    assert.ok(
      rustDependencyViolations(packages).some((message) => message.includes(dependency)),
    );
  }
});

test("requires architecture dependencies to remain local workspace crates", () => {
  const packages = validRustPackages();
  packages[1].dependencies[0].path = null;
  assert.ok(
    rustDependencyViolations(packages).some((message) =>
      message.includes("must use local workspace dependency acm-os-domain"),
    ),
  );
});

test("rejects direct standard-library and native authority", () => {
  for (const source of [
    "use std::fs;",
    "use std::{collections::HashMap, net};",
    "std::process::Command::new(\"cmd\");",
    "SystemTime::now();",
    'extern "system" { fn GetCurrentProcessId() -> u32; }',
  ]) {
    assert.notDeepEqual(rustAuthorityViolations(source), []);
  }
  assert.deepEqual(rustAuthorityViolations("pub struct ProblemId(String);"), []);
});

test("rejects unapproved frontend dependencies and direct browser authority", () => {
  const packageJson = validFrontendPackage();
  packageJson.dependencies["better-sqlite3"] = "^12";
  assert.ok(
    frontendDependencyViolations(packageJson).some((message) =>
      message.includes("better-sqlite3"),
    ),
  );

  for (const source of [
    'fetch("https://example.test")',
    "indexedDB.open(\"acm-os\")",
    "showDirectoryPicker()",
  ]) {
    assert.notDeepEqual(frontendAuthorityViolations(source), []);
  }
});
