// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Gallery of starter templates (§9.1). Original content only — never
 *  copied from Apple's gallery (§2.5). Template names/descriptions are
 *  data-driven and localized like everything else. */

import { useTranslation } from 'react-i18next';

import { gradientCss, radii } from '@lightning/ui';

/** Starter templates rendered as data; ids double as i18n keys later. */
const TEMPLATES = [
  { id: 'daily-note', glyph: '📝', gradient: 'text' },
  { id: 'resize-batch', glyph: '🖼', gradient: 'images' },
  { id: 'focus-timer', glyph: '⏳', gradient: 'system' },
  { id: 'clipboard-clean', glyph: '📋', gradient: 'clipboard' },
] as const;

export function GalleryView() {
  const { t } = useTranslation();

  return (
    <section aria-label={t('nav.gallery')} className="flex flex-col gap-4">
      <h1 className="text-xl font-bold">{t('nav.gallery')}</h1>
      <div className="grid grid-cols-4 gap-4">
        {TEMPLATES.map((template) => (
          <div
            key={template.id}
            className="flex h-32 flex-col justify-between p-4 text-white"
            style={{ backgroundImage: gradientCss(template.gradient), borderRadius: radii.tile }}
          >
            <span aria-hidden className="text-2xl">
              {template.glyph}
            </span>
            <span className="text-sm font-semibold">{template.id}</span>
          </div>
        ))}
      </div>
    </section>
  );
}
