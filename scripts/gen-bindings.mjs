// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc
//
// `pnpm bindings` — regenerates packages/bindings/src/index.ts from the
// specta-annotated types and command table in crates/ipc-types.
// `pnpm bindings --check` fails if the committed bindings are stale (CI gate).

import { spawnSync } from 'node:child_process';
import { readFileSync, mkdtempSync, rmSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join, resolve } from 'node:path';

const repoRoot = resolve(import.meta.dirname, '..');
const target = join(repoRoot, 'packages', 'bindings', 'src', 'index.ts');
const check = process.argv.includes('--check');

const outDir = check ? mkdtempSync(join(tmpdir(), 'lightning-bindings-')) : null;
const outFile = check ? join(outDir, 'index.ts') : target;

const res = spawnSync(
  'cargo',
  [
    'run',
    '--quiet',
    '-p',
    'lightning-ipc-types',
    '--features',
    'export',
    '--bin',
    'export-bindings',
    '--',
    outFile,
  ],
  { stdio: 'inherit', cwd: repoRoot },
);
if (res.status !== 0) process.exit(res.status ?? 1);

// Keep generated output stable under prettier so fmt:check never fights the generator.
spawnSync('pnpm', ['exec', 'prettier', '--write', outFile], {
  stdio: 'inherit',
  cwd: repoRoot,
  shell: process.platform === 'win32',
});

if (check) {
  const fresh = readFileSync(outFile, 'utf8');
  const committed = readFileSync(target, 'utf8');
  rmSync(outDir, { recursive: true, force: true });
  if (fresh !== committed) {
    console.error('✗ packages/bindings/src/index.ts is stale — run `pnpm bindings` and commit.');
    process.exit(1);
  }
  console.log('✓ bindings are fresh');
} else {
  console.log(`✓ wrote ${target}`);
}
