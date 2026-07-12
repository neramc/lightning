// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * i18n bootstrap (CLAUDE.md §15). Keys are stable IDs — never
 * English-as-key. `en` is the source locale; `ko` ships 100% complete
 * (release-blocking, enforced by keys-parity.test.ts).
 */

import i18next, { type i18n } from 'i18next';
import { initReactI18next } from 'react-i18next';

import enActions from '../locales/en/actions.json';
import enCommon from '../locales/en/common.json';
import enEditor from '../locales/en/editor.json';
import enInstaller from '../locales/en/installer.json';
import enSettings from '../locales/en/settings.json';
import koActions from '../locales/ko/actions.json';
import koCommon from '../locales/ko/common.json';
import koEditor from '../locales/ko/editor.json';
import koInstaller from '../locales/ko/installer.json';
import koSettings from '../locales/ko/settings.json';

export const NAMESPACES = ['common', 'actions', 'editor', 'settings', 'installer'] as const;
export type Namespace = (typeof NAMESPACES)[number];

export const SUPPORTED_LOCALES = ['en', 'ko'] as const;
export type Locale = (typeof SUPPORTED_LOCALES)[number];

export const resources = {
  en: {
    common: enCommon,
    actions: enActions,
    editor: enEditor,
    settings: enSettings,
    installer: enInstaller,
  },
  ko: {
    common: koCommon,
    actions: koActions,
    editor: koEditor,
    settings: koSettings,
    installer: koInstaller,
  },
} as const;

/** Initialize i18next for the app (React binding included). */
export async function createI18n(locale: Locale = 'en'): Promise<i18n> {
  const instance = i18next.createInstance();
  await instance.use(initReactI18next).init({
    lng: locale,
    fallbackLng: 'en',
    ns: [...NAMESPACES],
    defaultNS: 'common',
    resources,
    interpolation: {
      // React already escapes; keep {{var}} raw.
      escapeValue: false,
    },
    returnNull: false,
  });
  return instance;
}

/** The i18n key for an action's display name. */
export function actionNameKey(actionId: string): string {
  return `actions:${actionId}.name`;
}

/** The i18n key for an action's description. */
export function actionDescriptionKey(actionId: string): string {
  return `actions:${actionId}.description`;
}

/** The i18n key for an action param's label. */
export function actionParamKey(actionId: string, paramKey: string): string {
  return `actions:${actionId}.params.${paramKey}`;
}
