// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

// Port 1420 is fixed — `pnpm dev` (tauri dev) expects it and clashes with
// `pnpm e2e` (CLAUDE.md §5).
export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  envPrefix: ['VITE_', 'TAURI_ENV_'],
  build: {
    target: 'es2022',
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
  test: {
    environment: 'jsdom',
  },
});
