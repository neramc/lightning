// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * The one sanctioned way to animate (CLAUDE.md §9.2): under
 * `prefers-reduced-motion`, springs collapse to 80 ms fades — no parallax,
 * no confetti. Always animate through this hook.
 */

import { useEffect, useState } from 'react';

import {
  duration,
  reducedMotionFadeMs,
  spring,
  type DurationName,
  type SpringName,
  type SpringToken,
} from './motion';

export interface MotionPrefs {
  reducedMotion: boolean;
  /** A spring transition, or a short fade when reduced motion is on. */
  spring: (name: SpringName) => SpringToken | { duration: number };
  /** A duration in seconds (motion expects seconds). */
  duration: (name: DurationName) => number;
}

const QUERY = '(prefers-reduced-motion: reduce)';

export function useMotionPrefs(): MotionPrefs {
  const [reducedMotion, setReducedMotion] = useState<boolean>(() =>
    typeof window !== 'undefined' && 'matchMedia' in window
      ? window.matchMedia(QUERY).matches
      : false,
  );

  useEffect(() => {
    if (typeof window === 'undefined' || !('matchMedia' in window)) return;
    const media = window.matchMedia(QUERY);
    const onChange = (event: MediaQueryListEvent) => setReducedMotion(event.matches);
    media.addEventListener('change', onChange);
    return () => media.removeEventListener('change', onChange);
  }, []);

  return {
    reducedMotion,
    spring: (name) =>
      reducedMotion ? { duration: reducedMotionFadeMs / 1000 } : spring[name],
    duration: (name) =>
      (reducedMotion ? Math.min(duration[name], reducedMotionFadeMs) : duration[name]) / 1000,
  };
}
