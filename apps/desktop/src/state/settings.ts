// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Settings slice — locale/theme, persisted to localStorage (the canonical
 *  settings.json lives Rust-side; this mirrors the UI-facing subset). */

import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type Theme = 'system' | 'light' | 'dark';

interface SettingsState {
  locale: 'en' | 'ko';
  theme: Theme;
  setLocale: (locale: 'en' | 'ko') => void;
  setTheme: (theme: Theme) => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    (set) => ({
      locale: 'en',
      theme: 'system',
      setLocale: (locale) => set({ locale }),
      setTheme: (theme) => set({ theme }),
    }),
    { name: 'lightning-settings' },
  ),
);
