// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc
//
// `pnpm version:sync <x.y.z>` — one version everywhere (CLAUDE.md §13):
// package.json (root) · Cargo.toml [workspace.package] · tauri.conf.json ·
// installer/windows/Directory.Build.props.
// `--check <x.y.z>` verifies without writing (used by release.yml).

import { readFileSync, writeFileSync } from 'node:fs';
import { resolve, join } from 'node:path';

const repoRoot = resolve(import.meta.dirname, '..');
const args = process.argv.slice(2);
const check = args[0] === '--check';
const version = check ? args[1] : args[0];

if (!/^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/.test(version ?? '')) {
  console.error('usage: node scripts/sync-version.mjs [--check] <x.y.z>');
  process.exit(2);
}

/** @type {Array<[string, RegExp, string]>} */
const targets = [
  [
    join(repoRoot, 'package.json'),
    /("version":\s*")[^"]+(")/,
    `$1${version}$2`,
  ],
  [
    join(repoRoot, 'Cargo.toml'),
    /(\[workspace\.package\][\s\S]*?version\s*=\s*")[^"]+(")/,
    `$1${version}$2`,
  ],
  [
    join(repoRoot, 'apps', 'desktop', 'src-tauri', 'tauri.conf.json'),
    /("version":\s*")[^"]+(")/,
    `$1${version}$2`,
  ],
  [
    join(repoRoot, 'installer', 'windows', 'Directory.Build.props'),
    /(<Version>)[^<]+(<\/Version>)/,
    `$1${version}$2`,
  ],
];

let stale = [];
for (const [file, pattern, replacement] of targets) {
  const before = readFileSync(file, 'utf8');
  const after = before.replace(pattern, replacement);
  if (!pattern.test(before)) {
    console.error(`✗ no version field found in ${file}`);
    process.exit(1);
  }
  if (before !== after) {
    if (check) stale.push(file);
    else writeFileSync(file, after);
  }
}

if (check) {
  if (stale.length > 0) {
    console.error(`✗ version mismatch (expected ${version}) in:\n  ${stale.join('\n  ')}`);
    process.exit(1);
  }
  console.log(`✓ all version fields are ${version}`);
} else {
  console.log(`✓ synced version ${version} across ${targets.length} files`);
}
