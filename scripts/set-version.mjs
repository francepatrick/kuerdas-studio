#!/usr/bin/env node
// Keep the app version in sync across npm, Cargo and Tauri.
// Usage: node scripts/set-version.mjs 0.2.0
//        node scripts/set-version.mjs --check 0.2.0
import { readFileSync, writeFileSync } from "node:fs";

const args = process.argv.slice(2);
const check = args[0] === "--check";
const version = check ? args[1] : args[0];

if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version ?? "")) {
  console.error("Usage: node scripts/set-version.mjs [--check] <semver>");
  process.exit(1);
}

const files = {
  "package.json": {
    get: (text) => JSON.parse(text).version,
    set: (text) => text.replace(/("version":\s*")[^"]+(")/, `$1${version}$2`),
  },
  "src-tauri/tauri.conf.json": {
    get: (text) => JSON.parse(text).version,
    set: (text) => text.replace(/("version":\s*")[^"]+(")/, `$1${version}$2`),
  },
  // Static shields.io badge (repo is private, so dynamic badges can't read it).
  // Shields escapes "-" as "--" inside badge text.
  "README.md": {
    get: (text) =>
      text.match(/badge\/release-v(.+?)-[0-9a-f]{6}"/)[1].replaceAll("--", "-"),
    set: (text) =>
      text.replace(
        /(badge\/release-v).+?(-[0-9a-f]{6}")/,
        `$1${version.replaceAll("-", "--")}$2`,
      ),
  },
  "Cargo.toml": {
    get: (text) =>
      text.match(/\[workspace\.package\][^[]*?version\s*=\s*"([^"]+)"/)[1],
    set: (text) =>
      text.replace(
        /(\[workspace\.package\][^[]*?version\s*=\s*")[^"]+(")/,
        `$1${version}$2`,
      ),
  },
};

let mismatched = false;
for (const [path, { get, set }] of Object.entries(files)) {
  const text = readFileSync(path, "utf8");
  const current = get(text);
  if (check) {
    if (current !== version) {
      console.error(`${path}: ${current} (expected ${version})`);
      mismatched = true;
    }
  } else if (current !== version) {
    writeFileSync(path, set(text));
    console.log(`${path}: ${current} -> ${version}`);
  }
}

if (check) {
  if (mismatched) process.exit(1);
  console.log(`All versions match ${version}`);
} else {
  console.log(
    "Run `npm install --package-lock-only` and `cargo check` to refresh lockfiles.",
  );
}
