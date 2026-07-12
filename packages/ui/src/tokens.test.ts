// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

import { describe, expect, it } from 'vitest';

import { duration, spring } from './motion';
import { CATEGORY_IDS, categoryGradients, gradientCss } from './tokens';

describe('design tokens', () => {
  it('every category id has a gradient', () => {
    for (const id of CATEGORY_IDS) {
      expect(categoryGradients[id], `gradient for ${id}`).toBeDefined();
      expect(categoryGradients[id].from).toMatch(/^#[0-9a-f]{6}$/);
      expect(categoryGradients[id].to).toMatch(/^#[0-9a-f]{6}$/);
    }
  });

  it('unknown categories fall back to the system gradient', () => {
    expect(gradientCss('definitely-not-a-category')).toContain(
      categoryGradients.system.from,
    );
  });

  it('motion tokens match the spec (§9.2)', () => {
    expect(spring.snappy).toMatchObject({ stiffness: 420, damping: 30 });
    expect(spring.smooth).toMatchObject({ stiffness: 260, damping: 28 });
    expect(spring.gentle).toMatchObject({ stiffness: 170, damping: 26 });
    expect(duration).toMatchObject({ fast: 120, base: 200, slow: 320 });
  });
});
