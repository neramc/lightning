// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * WebdriverIO + tauri-driver config (CLAUDE.md §11). Linux and Windows only —
 * tauri-driver does not support macOS (manual checklist: docs/qa/macos-smoke.md).
 * Do not run alongside `pnpm dev` (port 1420 clash — §5).
 *
 * Prerequisites: `cargo build --release -p lightning-desktop` (the CI job
 * builds it), `tauri-driver` on PATH, and on Windows an msedgedriver —
 * GitHub runners preinstall one and export its directory as EDGEWEBDRIVER.
 */

import { spawn, type ChildProcess } from 'node:child_process';
import path from 'node:path';

let tauriDriver: ChildProcess | undefined;

// Cargo workspace: artifacts land in the root target/, not src-tauri/target/.
const appBinary = path.resolve(import.meta.dirname, '../../target/release/lightning-desktop');

function tauriDriverArgs(): string[] {
  if (process.platform !== 'win32') return [];
  const native =
    process.env.TAURI_NATIVE_DRIVER ??
    (process.env.EDGEWEBDRIVER
      ? path.join(process.env.EDGEWEBDRIVER, 'msedgedriver.exe')
      : undefined);
  return native ? ['--native-driver', native] : [];
}

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
      // WebKitWebDriver/tauri-driver speak classic WebDriver only; without
      // this wdio v9 requests BiDi (webSocketUrl) and session creation fails
      // with "Failed to match capabilities".
      'wdio:enforceWebDriverClassic': true,
    },
  ],
  framework: 'mocha',
  reporters: ['spec'],
  mochaOpts: { timeout: 60_000 },
  connectionRetryCount: 3,
  waitforTimeout: 15_000,

  onPrepare: () => {
    tauriDriver = spawn('tauri-driver', tauriDriverArgs(), {
      stdio: [null, process.stdout, process.stderr],
    });
  },
  onComplete: () => {
    tauriDriver?.kill();
  },
};
