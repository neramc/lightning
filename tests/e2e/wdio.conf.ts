// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * WebdriverIO + tauri-driver config (CLAUDE.md §11). Linux and Windows only —
 * tauri-driver does not support macOS (manual checklist: docs/qa/macos-smoke.md).
 * Do not run alongside `pnpm dev` (port 1420 clash — §5).
 */

import { spawn, type ChildProcess } from 'node:child_process';
import path from 'node:path';

let tauriDriver: ChildProcess | undefined;

const appBinary = path.resolve(
  import.meta.dirname,
  '../../apps/desktop/src-tauri/target/release/lightning-desktop',
);

export const config: WebdriverIO.Config = {
  runner: 'local',
  specs: ['./specs/**/*.e2e.ts'],
  maxInstances: 1,
  hostname: '127.0.0.1',
  port: 4444,
  capabilities: [
    {
      // @ts-expect-error tauri-specific capability, not in the wdio types
      'tauri:options': { application: appBinary },
      browserName: 'wry',
    },
  ],
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: { timeout: 60_000 },

  onPrepare: () => {
    tauriDriver = spawn('tauri-driver', [], { stdio: [null, process.stdout, process.stderr] });
  },
  onComplete: () => {
    tauriDriver?.kill();
  },
};
