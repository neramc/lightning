// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * Motion tokens (CLAUDE.md §9.2).
 *
 * Springs for spatial motion, durations for opacity/color. Transform &
 * opacity only; every animation is interruptible; under reduced motion
 * springs collapse to 80 ms fades (see use-motion-prefs).
 */

export interface SpringToken {
  type: 'spring';
  stiffness: number;
  damping: number;
}

export const spring = {
  /** Drag lift, palette pop. */
  snappy: { type: 'spring', stiffness: 420, damping: 30 } as SpringToken,
  /** Layout shifts, list reflow. */
  smooth: { type: 'spring', stiffness: 260, damping: 28 } as SpringToken,
  /** Large surfaces (gallery → editor expansion). */
  gentle: { type: 'spring', stiffness: 170, damping: 26 } as SpringToken,
} as const;

export type SpringName = keyof typeof spring;

/** Durations for opacity/color transitions, in milliseconds. */
export const duration = {
  fast: 120,
  base: 200,
  slow: 320,
} as const;

export type DurationName = keyof typeof duration;

/** The single fallback used when the user prefers reduced motion. */
export const reducedMotionFadeMs = 80;

/** Error shake (§9.3): ±4 px, 2 cycles. */
export const errorShakeKeyframes = [0, -4, 4, -4, 4, 0];
