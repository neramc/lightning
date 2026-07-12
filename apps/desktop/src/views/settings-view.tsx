// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Settings — language, appearance, privacy note (§14: no telemetry). */

import { useTranslation } from 'react-i18next';

import { SUPPORTED_LOCALES } from '@lightning/i18n';

import { useSettingsStore, type Theme } from '../state/settings';

const selectClasses =
  'rounded-[10px] border border-zinc-300 bg-white px-2 py-1 text-sm dark:border-zinc-700 dark:bg-zinc-900';

export function SettingsView() {
  const { t } = useTranslation('settings');
  const settings = useSettingsStore();

  return (
    <section aria-label={t('sections.general')} className="flex max-w-xl flex-col gap-8">
      <div>
        <h1 className="mb-4 text-xl font-bold">{t('sections.general')}</h1>
        <div className="flex flex-col gap-3">
          <label className="flex items-center justify-between text-sm">
            {t('general.language')}
            <select
              value={settings.locale}
              onChange={(e) => settings.setLocale(e.target.value as 'en' | 'ko')}
              className={selectClasses}
            >
              {SUPPORTED_LOCALES.map((locale) => (
                <option key={locale} value={locale}>
                  {locale === 'ko' ? '한국어' : 'English'}
                </option>
              ))}
            </select>
          </label>
          <label className="flex items-center justify-between text-sm">
            {t('general.theme')}
            <select
              value={settings.theme}
              onChange={(e) => settings.setTheme(e.target.value as Theme)}
              className={selectClasses}
            >
              <option value="system">{t('general.themeSystem')}</option>
              <option value="light">{t('general.themeLight')}</option>
              <option value="dark">{t('general.themeDark')}</option>
            </select>
          </label>
        </div>
      </div>
      <div>
        <h2 className="mb-2 text-lg font-semibold">{t('sections.privacy')}</h2>
        <p className="text-sm text-zinc-600 dark:text-zinc-300">{t('privacy.telemetryNote')}</p>
      </div>
    </section>
  );
}
