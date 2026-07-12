// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Resizable-sidebar layout (§9.1) with the Ctrl/⌘-K command palette. */

import { useEffect, useState } from 'react';
import { Link, Outlet } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';

import { useCapabilitiesStore } from '../state/capabilities';
import { useShortcutsStore } from '../state/shortcuts';
import { CommandPalette } from './command-palette';

const NAV = [
  { to: '/shortcuts', key: 'nav.shortcuts', glyph: '⚡' },
  { to: '/automations', key: 'nav.automations', glyph: '⏱' },
  { to: '/gallery', key: 'nav.gallery', glyph: '✨' },
  { to: '/settings', key: 'nav.settings', glyph: '⚙' },
] as const;

export function AppShell() {
  const { t } = useTranslation();
  const [paletteOpen, setPaletteOpen] = useState(false);
  const initCapabilities = useCapabilitiesStore((s) => s.init);
  const refreshShortcuts = useShortcutsStore((s) => s.refresh);
  const subscribeShortcuts = useShortcutsStore((s) => s.subscribe);

  useEffect(() => {
    void initCapabilities();
    void refreshShortcuts();
    void subscribeShortcuts();
  }, [initCapabilities, refreshShortcuts, subscribeShortcuts]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k') {
        event.preventDefault();
        setPaletteOpen((open) => !open);
      }
    }
    window.addEventListener('keydown', onKeyDown);
    return () => window.removeEventListener('keydown', onKeyDown);
  }, []);

  return (
    <div className="flex h-screen bg-zinc-50 text-zinc-900 dark:bg-zinc-950 dark:text-zinc-100">
      <nav
        aria-label={t('nav.shortcuts')}
        className="flex w-56 shrink-0 flex-col gap-1 border-r border-zinc-200 p-3 dark:border-zinc-800"
      >
        <p className="px-2 pb-2 text-lg font-bold">⚡ Lightning</p>
        {NAV.map((item) => (
          <Link
            key={item.to}
            to={item.to}
            className="rounded-[10px] px-3 py-2 text-sm font-medium hover:bg-zinc-500/10 focus-visible:outline focus-visible:outline-2 focus-visible:outline-sky-500 [&.active]:bg-sky-500/15 [&.active]:text-sky-700 dark:[&.active]:text-sky-300"
          >
            <span aria-hidden className="mr-2">
              {item.glyph}
            </span>
            {t(item.key)}
          </Link>
        ))}
      </nav>
      <main className="min-w-0 flex-1 overflow-y-auto p-6">
        <Outlet />
      </main>
      {paletteOpen ? <CommandPalette onClose={() => setPaletteOpen(false)} /> : null}
    </div>
  );
}
