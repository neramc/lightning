// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

import { beforeEach, describe, expect, it, vi } from 'vitest';

vi.mock('@lightning/bindings', () => ({
  cancelRun: vi.fn(),
  listActions: vi.fn(async () => []),
  loadShortcut: vi.fn(async (id: string) => ({
    schemaVersion: 1,
    id,
    name: 'Mock',
    description: '',
    icon: { glyph: '⚡', gradient: 'system' },
    hotkey: null,
    steps: [],
    trigger: null,
  })),
  onRunProgress: vi.fn(async () => () => {}),
  runShortcut: vi.fn(),
  saveShortcut: vi.fn(async () => ({})),
}));

import { saveShortcut } from '@lightning/bindings';

import { useEditorStore } from './editor';

describe('editor store', () => {
  beforeEach(() => {
    useEditorStore.setState({
      shortcut: null,
      catalog: [],
      dirty: false,
      running: false,
      runId: null,
      runStates: {},
      lastResult: null,
    });
  });

  it('open loads the document and clears dirty', async () => {
    await useEditorStore.getState().open('abc');
    expect(useEditorStore.getState().shortcut?.name).toBe('Mock');
    expect(useEditorStore.getState().dirty).toBe(false);
  });

  it('setSteps and rename mark the document dirty', async () => {
    await useEditorStore.getState().open('abc');
    useEditorStore.getState().rename('New Name');
    expect(useEditorStore.getState().dirty).toBe(true);
    expect(useEditorStore.getState().shortcut?.name).toBe('New Name');
  });

  it('save persists and clears dirty', async () => {
    await useEditorStore.getState().open('abc');
    useEditorStore.getState().rename('Changed');
    await useEditorStore.getState().save();
    expect(saveShortcut).toHaveBeenCalledOnce();
    expect(useEditorStore.getState().dirty).toBe(false);
  });
});
