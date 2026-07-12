// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Shortcut grid slice — indexed metadata, refreshed on store://changed. */

import { create } from 'zustand';

import {
  deleteShortcut,
  listShortcuts,
  onStoreChanged,
  type ShortcutMetaDto,
} from '@lightning/bindings';

interface ShortcutsState {
  metas: ShortcutMetaDto[];
  loading: boolean;
  subscribed: boolean;
  refresh: () => Promise<void>;
  remove: (id: string) => Promise<void>;
  subscribe: () => Promise<void>;
}

export const useShortcutsStore = create<ShortcutsState>()((set, get) => ({
  metas: [],
  loading: false,
  subscribed: false,

  refresh: async () => {
    set({ loading: true });
    try {
      const metas = await listShortcuts();
      set({ metas, loading: false });
    } catch {
      set({ loading: false });
    }
  },

  remove: async (id) => {
    await deleteShortcut(id);
    set({ metas: get().metas.filter((m) => m.id !== id) });
  },

  subscribe: async () => {
    if (get().subscribed) return;
    set({ subscribed: true });
    await onStoreChanged(() => void get().refresh());
  },
}));
