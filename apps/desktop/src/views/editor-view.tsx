// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** The vertical block editor: canvas + run panel (§9.1, §9.3). */

import { useEffect } from 'react';
import { useParams } from '@tanstack/react-router';
import { useTranslation } from 'react-i18next';

import { FlowCanvas } from '@lightning/editor';
import { Button } from '@lightning/ui';

import { useCapabilitiesStore } from '../state/capabilities';
import { useEditorStore } from '../state/editor';

export function EditorView() {
  const { id } = useParams({ from: '/editor/$id' });
  const { t } = useTranslation();
  const { t: te } = useTranslation('editor');
  const snapshot = useCapabilitiesStore((s) => s.snapshot);
  const editor = useEditorStore();

  const loadCatalog = useEditorStore((s) => s.loadCatalog);
  const open = useEditorStore((s) => s.open);
  useEffect(() => {
    void loadCatalog();
    void open(id);
  }, [id, loadCatalog, open]);

  if (editor.shortcut === null) return null;

  return (
    <section className="grid h-full grid-cols-[1fr_280px] gap-6">
      <div className="min-w-0">
        <header className="mb-4 flex items-center gap-3">
          <input
            value={editor.shortcut.name}
            onChange={(e) => editor.rename(e.target.value)}
            aria-label={t('general.rename')}
            className="min-w-0 flex-1 bg-transparent text-xl font-bold outline-none focus-visible:rounded focus-visible:outline-2 focus-visible:outline-sky-500"
          />
          {editor.dirty ? (
            <Button variant="ghost" onClick={() => void editor.save()}>
              {t('general.save')}
            </Button>
          ) : null}
          {editor.running ? (
            <Button variant="danger" onClick={() => void editor.stop()}>
              {t('run.stop')}
            </Button>
          ) : (
            <Button onClick={() => void editor.run()}>{t('run.run')}</Button>
          )}
        </header>
        <FlowCanvas
          steps={editor.shortcut.steps}
          catalog={editor.catalog}
          snapshot={snapshot}
          runStates={editor.runStates}
          onReorder={(steps) => editor.setSteps(steps)}
        />
      </div>
      <aside
        aria-label={te('run.panelTitle')}
        className="flex min-h-0 flex-col rounded-2xl border border-zinc-200 p-3 dark:border-zinc-800"
      >
        <h2 className="mb-2 text-sm font-semibold">{te('run.panelTitle')}</h2>
        {editor.lastResult === null ? (
          <p className="text-xs text-zinc-500">{te('run.logEmpty')}</p>
        ) : (
          <ol className="min-h-0 flex-1 space-y-1 overflow-y-auto text-xs">
            {editor.lastResult.log.map((entry, index) => (
              <li
                key={index}
                className={entry.level === 'error' ? 'text-rose-500' : 'text-zinc-600 dark:text-zinc-300'}
              >
                {entry.message}
              </li>
            ))}
          </ol>
        )}
      </aside>
    </section>
  );
}
