// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { RouterProvider } from '@tanstack/react-router';
import { I18nextProvider } from 'react-i18next';

import { createI18n, type Locale } from '@lightning/i18n';

import './index.css';
import { router } from './router';
import { useSettingsStore } from './state/settings';

async function bootstrap() {
  const locale = useSettingsStore.getState().locale as Locale;
  const i18n = await createI18n(locale);
  useSettingsStore.subscribe((state, previous) => {
    if (state.locale !== previous.locale) void i18n.changeLanguage(state.locale);
  });

  const container = document.getElementById('root');
  if (container === null) throw new Error('missing #root');
  createRoot(container).render(
    <StrictMode>
      <I18nextProvider i18n={i18n}>
        <RouterProvider router={router} />
      </I18nextProvider>
    </StrictMode>,
  );
}

void bootstrap();
