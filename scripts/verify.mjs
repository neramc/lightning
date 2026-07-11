// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc
//
// `pnpm verify` — the full local gate (CLAUDE.md §5):
// fmt-check + clippy + lint + typecheck + unit tests.
// Flags: --skip-rust / --skip-js to run half the gate on constrained machines.

import { spawnSync } from 'node:child_process';

const args = new Set(process.argv.slice(2));
const skipRust = args.has('--skip-rust');
const skipJs = args.has('--skip-js');

const steps = [];
if (!skipJs) steps.push(['prettier (fmt:check)', 'pnpm', ['fmt:check']]);
if (!skipRust) {
  steps.push(['cargo fmt --check', 'cargo', ['fmt', '--all', '--check']]);
  steps.push([
    'cargo clippy',
    'cargo',
    ['clippy', '--workspace', '--all-targets', '--', '-D', 'warnings'],
  ]);
}
if (!skipJs) {
  steps.push(['eslint', 'pnpm', ['lint']]);
  steps.push(['tsc --noEmit', 'pnpm', ['typecheck']]);
  steps.push(['vitest', 'pnpm', ['test']]);
}
if (!skipRust) steps.push(['cargo test', 'cargo', ['test', '--workspace']]);

const failures = [];
for (const [name, cmd, cmdArgs] of steps) {
  console.log(`\n\x1b[1m▶ ${name}\x1b[0m`);
  const res = spawnSync(cmd, cmdArgs, { stdio: 'inherit', shell: process.platform === 'win32' });
  if (res.status !== 0) failures.push(name);
}

if (failures.length > 0) {
  console.error(`\n\x1b[31m✗ verify failed:\x1b[0m ${failures.join(', ')}`);
  process.exit(1);
}
console.log('\n\x1b[32m✓ verify passed\x1b[0m');
