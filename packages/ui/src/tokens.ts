// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * Design tokens (CLAUDE.md §9.1). This file is the ONLY place hex values may
 * live — components consume tokens, never ad-hoc colors.
 */

/** Category ids — must stay in sync with `Category::id()` in crates/actions. */
export const CATEGORY_IDS = [
  'controlFlow',
  'text',
  'math',
  'date',
  'web',
  'files',
  'clipboard',
  'images',
  'documents',
  'media',
  'appsWindows',
  'system',
  'input',
  'scripting',
  'communication',
  'productivity',
  'location',
] as const;

export type CategoryId = (typeof CATEGORY_IDS)[number];

export interface Gradient {
  /** Start color. */
  from: string;
  /** End color. */
  to: string;
}

/**
 * Category gradients (§9.1): e.g. scripting blue→indigo, files teal→cyan,
 * media pink→rose, web sky→blue, system slate→zinc, input orange→amber.
 */
export const categoryGradients: Record<CategoryId, Gradient> = {
  controlFlow: { from: '#8b5cf6', to: '#a855f7' }, // violet → purple
  text: { from: '#10b981', to: '#34d399' }, // emerald
  math: { from: '#f59e0b', to: '#fbbf24' }, // amber
  date: { from: '#ef4444', to: '#f87171' }, // red
  web: { from: '#0ea5e9', to: '#3b82f6' }, // sky → blue
  files: { from: '#14b8a6', to: '#06b6d4' }, // teal → cyan
  clipboard: { from: '#84cc16', to: '#a3e635' }, // lime
  images: { from: '#d946ef', to: '#e879f9' }, // fuchsia
  documents: { from: '#eab308', to: '#facc15' }, // yellow
  media: { from: '#ec4899', to: '#f43f5e' }, // pink → rose
  appsWindows: { from: '#6366f1', to: '#818cf8' }, // indigo
  system: { from: '#64748b', to: '#71717a' }, // slate → zinc
  input: { from: '#f97316', to: '#f59e0b' }, // orange → amber
  scripting: { from: '#3b82f6', to: '#6366f1' }, // blue → indigo
  communication: { from: '#22c55e', to: '#4ade80' }, // green
  productivity: { from: '#a855f7', to: '#c084fc' }, // purple
  location: { from: '#06b6d4', to: '#22d3ee' }, // cyan
};

/** CSS background-image string for a category tile/block. */
export function gradientCss(category: string): string {
  const gradient =
    categoryGradients[
      (category as CategoryId) in categoryGradients ? (category as CategoryId) : 'system'
    ];
  return `linear-gradient(135deg, ${gradient.from}, ${gradient.to})`;
}

/** Corner radii (§9.1: rounded 16–20 px, soft elevation). */
export const radii = {
  block: '16px',
  tile: '20px',
  control: '10px',
} as const;

/** Elevation shadows. */
export const elevation = {
  resting: '0 1px 3px rgb(0 0 0 / 0.08), 0 4px 12px rgb(0 0 0 / 0.06)',
  lifted: '0 4px 8px rgb(0 0 0 / 0.12), 0 12px 28px rgb(0 0 0 / 0.14)',
} as const;
