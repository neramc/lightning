// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Automations tab — the same flows, fired by triggers (§6.7). */

import { useMemo } from 'react';
import { useNavigate } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';

import { Badge } from '@lightning/ui';

import { ShortcutTile } from '../components/shortcut-tile';
import { useShortcutsStore } from '../state/shortcuts';

export function AutomationsView() {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const metas = useShortcutsStore((s) => s.metas);
  const automations = useMemo(() => metas.filter((m) => m.isAutomation), [metas]);

  return (
    <section aria-label={t('nav.automations')} className="flex flex-col gap-4">
      <header className="flex items-center gap-3">
        <h1 className="text-xl font-bold">{t('nav.automations')}</h1>
        <Badge tone="automation">{automations.length}</Badge>
      </header>
      <div className="grid grid-cols-4 gap-4">
        {automations.map((meta) => (
          <ShortcutTile
            key={meta.id}
            meta={meta}
            onOpen={() => void navigate({ to: '/editor/$id', params: { id: meta.id } })}
          />
        ))}
      </div>
    </section>
  );
}
