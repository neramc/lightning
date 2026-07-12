// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** The shortcut grid — virtualized by rows beyond 60 tiles (§9.4). */

import { useMemo, useRef } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useVirtualizer } from '@tanstack/react-virtual';
import { useTranslation } from 'react-i18next';

import type { ShortcutMetaDto } from '@lightning/bindings';

import { ShortcutTile } from '../components/shortcut-tile';
import { useShortcutsStore } from '../state/shortcuts';

const COLUMNS = 4;
const ROW_HEIGHT = 144; // tile 128 + gap 16

export function ShortcutsView() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const metas = useShortcutsStore((s) => s.metas);
  const nonAutomations = useMemo(() => metas.filter((m) => !m.isAutomation), [metas]);

  const rows = useMemo(() => {
    const chunks: ShortcutMetaDto[][] = [];
    for (let i = 0; i < nonAutomations.length; i += COLUMNS) {
      chunks.push(nonAutomations.slice(i, i + COLUMNS));
    }
    return chunks;
  }, [nonAutomations]);

  const scrollRef = useRef<HTMLDivElement>(null);
  const virtualizer = useVirtualizer({
    count: rows.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 4,
  });

  function openTile(meta: ShortcutMetaDto) {
    void navigate({ to: '/editor/$id', params: { id: meta.id } });
  }

  return (
    <section aria-label={t('nav.shortcuts')} className="flex h-full flex-col gap-4">
      <header className="flex items-center justify-between">
        <h1 className="text-xl font-bold">{t('nav.shortcuts')}</h1>
      </header>
      <div ref={scrollRef} className="min-h-0 flex-1 overflow-y-auto">
        <div style={{ height: virtualizer.getTotalSize(), position: 'relative' }}>
          {virtualizer.getVirtualItems().map((virtualRow) => (
            <div
              key={virtualRow.key}
              className="absolute inset-x-0 grid grid-cols-4 gap-4 pr-1"
              style={{ transform: `translateY(${virtualRow.start}px)` }}
            >
              {(rows[virtualRow.index] ?? []).map((meta) => (
                <ShortcutTile key={meta.id} meta={meta} onOpen={() => openTile(meta)} />
              ))}
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
