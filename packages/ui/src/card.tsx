// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** Card primitive — soft elevation, 16–20 px radii (§9.1). */

import type { HTMLAttributes } from 'react';

import { elevation, radii } from './tokens';

export function Card({ className = '', style, ...rest }: HTMLAttributes<HTMLDivElement>) {
  return (
    <div
      className={`bg-white dark:bg-zinc-900 ${className}`}
      style={{ borderRadius: radii.block, boxShadow: elevation.resting, ...style }}
      {...rest}
    />
  );
}
