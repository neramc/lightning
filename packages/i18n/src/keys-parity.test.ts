// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * Release gate (CLAUDE.md §15): `ko` must be 100% of `en`, and the §8.1
 * canonical string must stay byte-exact.
 */

import { describe, expect, it } from 'vitest';

import { NAMESPACES, resources } from './index';

function flattenKeys(value: unknown, prefix = ''): string[] {
  if (typeof value !== 'object' || value === null) return [prefix];
  return Object.entries(value as Record<string, unknown>).flatMap(([key, child]) =>
    flattenKeys(child, prefix === '' ? key : `${prefix}.${key}`),
  );
}

describe('locale completeness', () => {
  for (const ns of NAMESPACES) {
    it(`ko covers 100% of en in "${ns}"`, () => {
      const enKeys = new Set(flattenKeys(resources.en[ns]));
      const koKeys = new Set(flattenKeys(resources.ko[ns]));
      const missing = [...enKeys].filter((k) => !koKeys.has(k));
      const extra = [...koKeys].filter((k) => !enKeys.has(k));
      expect(missing, `missing in ko/${ns}.json`).toEqual([]);
      expect(extra, `stale keys in ko/${ns}.json`).toEqual([]);
    });
  }

  it('the canonical unsupported string is byte-exact (§8.1)', () => {
    expect(resources.ko.common.action.unsupportedOnOs).toBe('{{os}}에서 이 기능을 지원하지 않음');
    expect(resources.en.common.action.unsupportedOnOs).toBe('Not supported on {{os}}');
  });

  it('no English value leaked into ko action names', () => {
    // Heuristic: ko action names must contain at least one Hangul syllable
    // or be an accepted technical term (UUID, Base64, JavaScript…).
    const allowlist = new Set(['Base64', 'UUID 생성', 'JavaScript 실행']);
    const names = Object.values(resources.ko.actions).flatMap((group) =>
      typeof group === 'object'
        ? Object.values(group).map((entry) =>
            typeof entry === 'object' && entry !== null && 'name' in entry
              ? String((entry as { name: string }).name)
              : String(entry),
          )
        : [],
    );
    for (const name of names) {
      if (allowlist.has(name)) continue;
      expect(name, `suspiciously untranslated: ${name}`).toMatch(/\p{Script=Hangul}|·/u);
    }
  });
});
