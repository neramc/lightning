// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Editor slice — the open shortcut document, dirty tracking, and the run
 *  animation state driven by run://progress (§9.3). */

import { create } from 'zustand';

import {
  cancelRun,
  listActions,
  loadShortcut,
  onRunProgress,
  runShortcut,
  saveShortcut,
  type ActionDefDto,
  type RunResultDto,
  type ShortcutDto,
  type StepDto,
} from '@lightning/bindings';

export type StepRunState = 'idle' | 'running' | 'success' | 'failed';

interface EditorState {
  shortcut: ShortcutDto | null;
  catalog: ActionDefDto[];
  dirty: boolean;
  running: boolean;
  runId: string | null;
  runStates: Record<string, StepRunState>;
  lastResult: RunResultDto | null;

  loadCatalog: () => Promise<void>;
  open: (id: string) => Promise<void>;
  setSteps: (steps: StepDto[]) => void;
  rename: (name: string) => void;
  save: () => Promise<void>;
  run: () => Promise<void>;
  stop: () => Promise<void>;
}

export const useEditorStore = create<EditorState>()((set, get) => ({
  shortcut: null,
  catalog: [],
  dirty: false,
  running: false,
  runId: null,
  runStates: {},
  lastResult: null,

  loadCatalog: async () => {
    if (get().catalog.length > 0) return;
    set({ catalog: await listActions() });
  },

  open: async (id) => {
    set({ shortcut: await loadShortcut(id), dirty: false, runStates: {}, lastResult: null });
  },

  setSteps: (steps) => {
    const shortcut = get().shortcut;
    if (shortcut === null) return;
    set({ shortcut: { ...shortcut, steps }, dirty: true });
  },

  rename: (name) => {
    const shortcut = get().shortcut;
    if (shortcut === null) return;
    set({ shortcut: { ...shortcut, name }, dirty: true });
  },

  save: async () => {
    const shortcut = get().shortcut;
    if (shortcut === null) return;
    await saveShortcut(shortcut);
    set({ dirty: false });
  },

  run: async () => {
    const shortcut = get().shortcut;
    if (shortcut === null || get().running) return;
    set({ running: true, runStates: {}, lastResult: null });

    // Blocks light up top-to-bottom following run://progress (§9.3).
    const unlisten = await onRunProgress((progress) => {
      if (get().runId !== null && progress.runId !== get().runId) return;
      set({ runId: progress.runId });
      const next: StepRunState =
        progress.phase === 'started'
          ? 'running'
          : progress.phase === 'finished'
            ? 'success'
            : 'failed';
      set({ runStates: { ...get().runStates, [progress.step]: next } });
    });

    try {
      const result = await runShortcut(shortcut.id);
      set({ lastResult: result });
    } finally {
      unlisten();
      set({ running: false, runId: null });
    }
  },

  stop: async () => {
    const runId = get().runId;
    if (runId !== null) await cancelRun(runId);
  },
}));
