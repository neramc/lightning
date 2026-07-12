// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc
//
// Shared flat ESLint config. Notable house rules (CLAUDE.md §10):
// - no `any` — use `unknown` + narrowing
// - components never import `invoke` directly; only @lightning/bindings may

import js from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  { ignores: ['dist/**', 'node_modules/**', '.turbo/**'] },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/consistent-type-imports': 'error',
      'no-restricted-imports': [
        'error',
        {
          paths: [
            {
              name: '@tauri-apps/api/core',
              message:
                'Never call invoke() directly — use the generated wrappers from @lightning/bindings (CLAUDE.md §2.4).',
            },
          ],
        },
      ],
    },
  },
  {
    // The generated bindings are the one legitimate invoke() call site.
    files: ['**/packages/bindings/src/**'],
    rules: { 'no-restricted-imports': 'off' },
  },
);
