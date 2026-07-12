// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

import { describe, expect, it } from 'vitest';

import type { ActionDefDto, CapabilitySnapshotDto } from '@lightning/bindings';

import { effectiveSupport } from './support';

const full = { level: 'full', note: null };

function def(overrides: Partial<ActionDefDto> = {}): ActionDefDto {
  return {
    id: 'test.action',
    category: 'system',
    icon: 'gear',
    params: [],
    output: null,
    supports: { windows: full, macos: full, linux: full, freebsd: full },
    permission: null,
    requiresCapability: null,
    scriptParam: null,
    ...overrides,
  };
}

function snapshot(overrides: Partial<CapabilitySnapshotDto> = {}): CapabilitySnapshotDto {
  return {
    os: 'linux',
    environment: 'Wayland',
    osLabel: 'Linux (Wayland)',
    capabilities: [],
    ...overrides,
  };
}

describe('effectiveSupport (§6.6: static ∩ probe)', () => {
  it('full static support with a clean probe is supported', () => {
    expect(effectiveSupport(def(), snapshot()).kind).toBe('supported');
  });

  it('static none wins regardless of the probe', () => {
    const d = def({
      supports: { windows: full, macos: full, linux: { level: 'none', note: null }, freebsd: full },
    });
    expect(effectiveSupport(d, snapshot()).kind).toBe('unsupported');
  });

  it('an unavailable capability downgrades full support and carries the fix', () => {
    const d = def({ requiresCapability: 'InputInjection' });
    const s = snapshot({
      capabilities: [
        {
          capability: 'InputInjection',
          status: 'unavailable',
          reason: 'Wayland session without ydotool',
          fixTool: 'ydotool',
          fixHint: 'install ydotool',
          fixPermission: null,
        },
      ],
    });
    const result = effectiveSupport(d, s);
    expect(result).toMatchObject({ kind: 'unsupported', fixTool: 'ydotool' });
  });

  it('a degraded capability yields partial', () => {
    const d = def({ requiresCapability: 'Screenshot' });
    const s = snapshot({
      capabilities: [
        {
          capability: 'Screenshot',
          status: 'degraded',
          reason: 'via xdg portal',
          fixTool: null,
          fixHint: null,
          fixPermission: null,
        },
      ],
    });
    expect(effectiveSupport(d, s)).toMatchObject({ kind: 'partial', note: 'via xdg portal' });
  });
});
