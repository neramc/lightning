// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Type-safe routes (CLAUDE.md §4): /shortcuts · /editor/$id · /automations
 *  · /gallery · /settings. */

import { createRootRoute, createRoute, createRouter } from '@tanstack/react-router';

import { AppShell } from './components/app-shell';
import { AutomationsView } from './views/automations-view';
import { EditorView } from './views/editor-view';
import { GalleryView } from './views/gallery-view';
import { SettingsView } from './views/settings-view';
import { ShortcutsView } from './views/shortcuts-view';

const rootRoute = createRootRoute({ component: AppShell });

const indexRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/',
  component: ShortcutsView,
});

const shortcutsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/shortcuts',
  component: ShortcutsView,
});

const editorRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/editor/$id',
  component: EditorView,
});

const automationsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/automations',
  component: AutomationsView,
});

const galleryRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/gallery',
  component: GalleryView,
});

const settingsRoute = createRoute({
  getParentRoute: () => rootRoute,
  path: '/settings',
  component: SettingsView,
});

const routeTree = rootRoute.addChildren([
  indexRoute,
  shortcutsRoute,
  editorRoute,
  automationsRoute,
  galleryRoute,
  settingsRoute,
]);

export const router = createRouter({ routeTree });

declare module '@tanstack/react-router' {
  interface Register {
    router: typeof router;
  }
}
