// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc
//
// `pnpm check:targets` — cross-compile check for every Tier-1 target
// (CLAUDE.md §2.8 / §7). Checks the pure crates; the Tauri shell needs
// per-OS system libraries and is covered by the CI build matrix instead.

import { spawnSync } from 'node:child_process';

const TIER1_TARGETS = [
  'x86_64-pc-windows-msvc',
  'aarch64-apple-darwin',
  'x86_64-apple-darwin',
  'x86_64-unknown-linux-gnu',
];

// Pure-Rust crates only: crates that compile C (lightning-store via bundled
// SQLite, lightning-scripting via QuickJS, the Tauri shell) need a cross C
// toolchain and are covered by the per-OS CI build matrix instead.
const CRATES = [
  'lightning-core',
  'lightning-actions',
  'lightning-triggers',
  'lightning-platform',
  'lightning-platform-windows',
  'lightning-platform-macos',
  'lightning-platform-linux',
  'lightning-platform-bsd',
  'lightning-ipc-types',
];

function run(cmd, args) {
  const res = spawnSync(cmd, args, { stdio: 'inherit', shell: process.platform === 'win32' });
  return res.status === 0;
}

let failed = false;
for (const target of TIER1_TARGETS) {
  console.log(`\n\x1b[1m▶ rustup target add ${target}\x1b[0m`);
  if (!run('rustup', ['target', 'add', target])) {
    console.error(`✗ could not install std for ${target}`);
    failed = true;
    continue;
  }
  const pkgArgs = CRATES.flatMap((c) => ['-p', c]);
  console.log(`\x1b[1m▶ cargo check --target ${target}\x1b[0m`);
  if (!run('cargo', ['check', ...pkgArgs, '--target', target])) failed = true;
}

if (failed) {
  console.error(
    '\n\x1b[31m✗ check:targets failed — never break one Tier-1 platform to fix another.\x1b[0m',
  );
  process.exit(1);
}
console.log('\n\x1b[32m✓ all Tier-1 targets compile\x1b[0m');
