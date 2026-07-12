// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * Badges — including the capability badge that renders the localized
 * `action.unsupportedOnOs` / `action.needsTool` / `action.needsPermission`
 * strings (§8.1). The badge receives already-localized text; it never
 * hardcodes user-facing strings.
 */

import type { ReactNode } from 'react';

export type BadgeTone = 'neutral' | 'unsupported' | 'needsSetup' | 'automation';

const toneClasses: Record<BadgeTone, string> = {
  neutral: 'bg-zinc-500/15 text-zinc-600 dark:text-zinc-300',
  unsupported: 'bg-zinc-800/70 text-zinc-100',
  needsSetup: 'bg-amber-500/20 text-amber-700 dark:text-amber-300',
  automation: 'bg-violet-500/20 text-violet-700 dark:text-violet-300',
};

export interface BadgeProps {
  tone?: BadgeTone;
  children: ReactNode;
}

export function Badge({ tone = 'neutral', children }: BadgeProps) {
  return (
    <span
      className={`inline-flex items-center gap-1 rounded-full px-2 py-0.5 text-xs font-medium ${toneClasses[tone]}`}
    >
      {children}
    </span>
  );
}
