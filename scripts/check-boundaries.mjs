import { execFileSync } from "node:child_process";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const EXPECTED_RUST_DEPENDENCIES = {
  "acm-os-domain": ["normal:chrono"],
  "acm-os-application": ["normal:acm-os-domain"],
  "acm-os-infrastructure": [
    "dev:tokio",
    "normal:acm-os-application",
    "normal:acm-os-domain",
    "normal:chrono",
    "normal:pulldown-cmark",
    "normal:reqwest",
    "normal:rustls",
    "normal:same-file",
    "normal:serde",
    "normal:serde_json",
    "normal:sha2",
    "normal:sqlx",
    "normal:tempfile",
    "normal:tokio",
    "normal:uuid",
  ],
  "acm-os": [
    "build:tauri-build",
    "dev:serde_json",
    "dev:tempfile",
    "normal:acm-os-application",
    "normal:acm-os-domain",
    "normal:acm-os-infrastructure",
    "normal:notify",
    "normal:serde",
    "normal:tauri",
    "normal:tauri-plugin-opener",
    "normal:url",
  ],
};

const REQUIRED_LOCAL_RUST_DEPENDENCIES = {
  "acm-os-application": ["acm-os-domain"],
  "acm-os-infrastructure": ["acm-os-application", "acm-os-domain"],
  "acm-os": ["acm-os-application", "acm-os-domain", "acm-os-infrastructure"],
};

const EXPECTED_FRONTEND_DEPENDENCIES = {
  dependencies: ["@tauri-apps/api", "katex", "react", "react-dom"],
  devDependencies: [
    "@tauri-apps/cli",
    "@types/react",
    "@types/react-dom",
    "@vitejs/plugin-react",
    "jsdom",
    "tsx",
    "typescript",
    "vite",
  ],
  optionalDependencies: [],
  peerDependencies: [],
};

const RUST_AUTHORITY_RULES = [
  {
    pattern: /(?:^|[^A-Za-z0-9_])(?:::)?std\s*::\s*(?:fs|net|process|env|os)\b/m,
    description: "direct std filesystem/network/process/environment/platform access",
  },
  {
    pattern: /\buse\s+(?:::)?std\s*::\s*\{[^;}]*\b(?:fs|net|process|env|os)\b[^;}]*\}\s*;/ms,
    description: "grouped std filesystem/network/process/environment/platform import",
  },
  {
    pattern: /\b(?:SystemTime|Instant)\s*::\s*now\s*\(/m,
    description: "direct system clock access",
  },
  {
    pattern: /(?:^|[^A-Za-z0-9_])(?:::)?std\s*::\s*io\s*::\s*(?:stdin|stdout|stderr)\b/m,
    description: "direct process I/O access",
  },
  {
    pattern: /\bextern\s*"(?:system|C)"/m,
    description: "direct native platform FFI",
  },
  {
    pattern: /(?:^|[^A-Za-z0-9_])(?:::)?(?:windows|windows_sys|winapi)\s*::/m,
    description: "direct Windows API access",
  },
];

const FRONTEND_AUTHORITY_RULES = [
  {
    pattern: /\b(?:fetch|XMLHttpRequest|WebSocket|EventSource)\s*(?:\(|\{)/m,
    description: "direct browser network access",
  },
  {
    pattern: /\bindexedDB\b/m,
    description: "direct browser database access",
  },
  {
    pattern: /\b(?:showOpenFilePicker|showSaveFilePicker|showDirectoryPicker|FileSystemHandle)\b/m,
    description: "direct browser filesystem access",
  },
];

const dependencyKey = (dependency) =>
  `${dependency.kind ?? "normal"}:${dependency.name}`;

const compareSets = (actual, expected, label) => {
  const violations = [];
  const actualSet = new Set(actual);
  const expectedSet = new Set(expected);

  for (const value of actualSet) {
    if (!expectedSet.has(value)) violations.push(`${label} has unexpected entry ${value}`);
  }
  for (const value of expectedSet) {
    if (!actualSet.has(value)) violations.push(`${label} is missing required entry ${value}`);
  }

  return violations;
};

export const rustDependencyViolations = (packages) => {
  const violations = [];
  const packagesByName = new Map(packages.map((pkg) => [pkg.name, pkg]));

  violations.push(
    ...compareSets(
      packagesByName.keys(),
      Object.keys(EXPECTED_RUST_DEPENDENCIES),
      "Rust workspace",
    ),
  );

  for (const [packageName, expectedDependencies] of Object.entries(
    EXPECTED_RUST_DEPENDENCIES,
  )) {
    const pkg = packagesByName.get(packageName);
    if (!pkg) continue;

    violations.push(
      ...compareSets(
        pkg.dependencies.map(dependencyKey),
        expectedDependencies,
        `Rust package ${packageName}`,
      ),
    );

    for (const localName of REQUIRED_LOCAL_RUST_DEPENDENCIES[packageName] ?? []) {
      const dependency = pkg.dependencies.find((item) => item.name === localName);
      if (dependency && !dependency.path) {
        violations.push(
          `Rust package ${packageName} must use local workspace dependency ${localName}`,
        );
      }
    }
  }

  return violations;
};

export const frontendDependencyViolations = (packageJson) => {
  const violations = [];

  for (const [section, expectedDependencies] of Object.entries(
    EXPECTED_FRONTEND_DEPENDENCIES,
  )) {
    violations.push(
      ...compareSets(
        Object.keys(packageJson[section] ?? {}),
        expectedDependencies,
        `package.json ${section}`,
      ),
    );
  }

  return violations;
};

const matchingAuthorityRules = (source, rules) =>
  rules
    .filter(({ pattern }) => pattern.test(source))
    .map(({ description }) => description);

export const rustAuthorityViolations = (source) =>
  matchingAuthorityRules(source, RUST_AUTHORITY_RULES);

export const frontendAuthorityViolations = (source) =>
  matchingAuthorityRules(source, FRONTEND_AUTHORITY_RULES);

const walkFiles = (directory, extension) => {
  if (!existsSync(directory)) return [];

  return readdirSync(directory).flatMap((entry) => {
    const path = resolve(directory, entry);
    return statSync(path).isDirectory()
      ? walkFiles(path, extension)
      : path.endsWith(extension)
        ? [path]
        : [];
  });
};

const cargoCandidates = () => {
  const candidates = [process.env.CARGO, "cargo"];
  if (process.platform === "win32") {
    candidates.push(resolve(homedir(), ".cargo", "bin", "cargo.exe"));
  }
  return [...new Set(candidates.filter(Boolean))];
};

const loadCargoMetadata = (root) => {
  const args = [
    "metadata",
    "--manifest-path",
    resolve(root, "src-tauri", "Cargo.toml"),
    "--format-version",
    "1",
    "--no-deps",
    "--locked",
  ];
  let missingCargoError;

  for (const cargo of cargoCandidates()) {
    if (cargo !== "cargo" && !existsSync(cargo)) continue;
    try {
      return JSON.parse(
        execFileSync(cargo, args, {
          cwd: root,
          encoding: "utf8",
          stdio: ["ignore", "pipe", "pipe"],
        }),
      );
    } catch (error) {
      if (error.code === "ENOENT") {
        missingCargoError = error;
        continue;
      }
      const details = error.stderr?.toString().trim() || error.message;
      throw new Error(`cargo metadata failed: ${details}`);
    }
  }

  throw new Error(`Cargo was not found: ${missingCargoError?.message ?? "no candidate exists"}`);
};

export const checkRepositoryBoundaries = (root) => {
  const violations = [];
  const metadata = loadCargoMetadata(root);
  violations.push(...rustDependencyViolations(metadata.packages));

  const packageJson = JSON.parse(readFileSync(resolve(root, "package.json"), "utf8"));
  violations.push(...frontendDependencyViolations(packageJson));

  const authorityRoots = [
    ["Domain", resolve(root, "src-tauri", "crates", "acm-os-domain", "src")],
    ["Application", resolve(root, "src-tauri", "crates", "acm-os-application", "src")],
  ];
  for (const [layer, directory] of authorityRoots) {
    for (const path of walkFiles(directory, ".rs")) {
      const source = readFileSync(path, "utf8");
      for (const violation of rustAuthorityViolations(source)) {
        violations.push(`${layer} source ${path}: ${violation}`);
      }
    }
  }

  for (const path of walkFiles(resolve(root, "src"), ".ts")) {
    const source = readFileSync(path, "utf8");
    for (const violation of frontendAuthorityViolations(source)) {
      violations.push(`Frontend source ${path}: ${violation}`);
    }
  }
  for (const path of walkFiles(resolve(root, "src"), ".tsx")) {
    const source = readFileSync(path, "utf8");
    for (const violation of frontendAuthorityViolations(source)) {
      violations.push(`Frontend source ${path}: ${violation}`);
    }
  }

  return violations;
};

const isMainModule =
  process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url);

if (isMainModule) {
  try {
    const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
    const violations = checkRepositoryBoundaries(root);
    if (violations.length > 0) {
      for (const violation of violations) {
        console.error(`boundary check failed: ${violation}`);
      }
      process.exitCode = 1;
    } else {
      console.log("boundary check passed");
    }
  } catch (error) {
    console.error(`boundary check failed: ${error.message}`);
    process.exitCode = 1;
  }
}
