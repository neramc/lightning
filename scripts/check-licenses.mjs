// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc
//
// `pnpm licenses:check` — JS-side GPL-3.0 compatibility gate (CLAUDE.md §16).
// The Rust side is covered by `cargo deny check` (deny.toml).

import { spawnSync } from 'node:child_process';

const ALLOWED = new Set([
  'MIT',
  'Apache-2.0',
  'BSD-2-Clause',
  'BSD-3-Clause',
  'ISC',
  'MPL-2.0',
  'LGPL-2.1',
  'LGPL-3.0',
  'GPL-3.0',
  'GPL-3.0-only',
  'GPL-3.0-or-later',
  'Zlib',
  '0BSD',
  'CC0-1.0',
  'CC-BY-4.0',
  'BlueOak-1.0.0',
  'Unlicense',
  'Python-2.0',
]);

function allowed(expr) {
  // Accept compound SPDX expressions when every referenced license is allowed.
  const ids = expr.split(/\s+(?:OR|AND)\s+|[()]/).filter(Boolean);
  return ids.every((id) => ALLOWED.has(id.trim()));
}

const res = spawnSync('pnpm', ['licenses', 'list', '--json', '--prod'], {
  encoding: 'utf8',
  shell: process.platform === 'win32',
});
if (res.status !== 0) {
  console.error(res.stderr || 'pnpm licenses failed');
  process.exit(1);
}

const report = JSON.parse(res.stdout || '{}');
const violations = [];
for (const [license, packages] of Object.entries(report)) {
  if (license === 'Unknown' || !allowed(license)) {
    for (const pkg of packages) violations.push(`${pkg.name}@${pkg.versions?.join(',')} — ${license}`);
  }
}

if (violations.length > 0) {
  console.error('✗ GPL-3.0-incompatible or unknown licenses found:');
  for (const v of violations) console.error(`  ${v}`);
  console.error('If a license is a false positive, document the exception in an ADR first.');
  process.exit(1);
}
console.log('✓ all JS dependencies are GPL-3.0-compatible');
