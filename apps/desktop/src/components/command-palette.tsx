// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Ctrl/⌘-K command palette (§9.1): search shortcuts, jump to the editor. */

import { useMemo, useState } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';

import { useShortcutsStore } from '../state/shortcuts';

export function CommandPalette({ onClose }: { onClose: () => void }) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const metas = useShortcutsStore((s) => s.metas);
  const [query, setQuery] = useState('');

  const results = useMemo(() => {
    const needle = query.trim().toLowerCase();
    if (needle === '') return metas.slice(0, 8);
    return metas.filter((m) => m.name.toLowerCase().includes(needle)).slice(0, 8);
  }, [metas, query]);

  return (
    <div
      role="dialog"
      aria-modal="true"
      className="fixed inset-0 z-50 flex items-start justify-center bg-black/30 pt-[15vh]"
      onClick={onClose}
      onKeyDown={(e) => {
        if (e.key === 'Escape') onClose();
      }}
    >
      <div
        className="w-[520px] overflow-hidden rounded-2xl bg-white shadow-2xl dark:bg-zinc-900"
        onClick={(e) => e.stopPropagation()}
      >
        <input
          autoFocus
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          placeholder={t('commandPalette.placeholder')}
          className="w-full border-b border-zinc-200 bg-transparent px-4 py-3 text-sm outline-none dark:border-zinc-700"
        />
        <ul className="max-h-80 overflow-y-auto p-2">
          {results.length === 0 ? (
            <li className="px-3 py-2 text-sm text-zinc-500">{t('commandPalette.noResults')}</li>
          ) : (
            results.map((meta) => (
              <li key={meta.id}>
                <button
                  type="button"
                  className="flex w-full items-center gap-2 rounded-[10px] px-3 py-2 text-left text-sm hover:bg-zinc-500/10 focus-visible:outline focus-visible:outline-2 focus-visible:outline-sky-500"
                  onClick={() => {
                    onClose();
                    void navigate({ to: '/editor/$id', params: { id: meta.id } });
                  }}
                >
                  <span aria-hidden>{meta.iconGlyph}</span>
                  <span className="flex-1 truncate">{meta.name}</span>
                  {meta.hotkey !== null ? (
                    <kbd className="rounded bg-zinc-500/15 px-1.5 py-0.5 text-[10px]">
                      {meta.hotkey}
                    </kbd>
                  ) : null}
                </button>
              </li>
            ))
          )}
        </ul>
      </div>
    </div>
  );
}
