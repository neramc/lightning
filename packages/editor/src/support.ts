// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * Effective support = static matrix ∩ runtime probe (CLAUDE.md §6.6).
 * Pure so it is unit-testable; the block component renders the result.
 */

import type { ActionDefDto, CapabilitySnapshotDto } from '@lightning/bindings';

export type EffectiveSupport =
  | { kind: 'supported' }
  | { kind: 'partial'; note: string | null }
  | {
      kind: 'unsupported';
      reason: string | null;
      fixTool: string | null;
      fixPermission: string | null;
    };

/** Compute how an action behaves on the probed system. */
export function effectiveSupport(
  def: ActionDefDto,
  snapshot: CapabilitySnapshotDto,
): EffectiveSupport {
  const byOs = {
    windows: def.supports.windows,
    macos: def.supports.macos,
    linux: def.supports.linux,
    freebsd: def.supports.freebsd,
  }[snapshot.os] ?? { level: 'none', note: null };

  if (byOs.level === 'none') {
    return { kind: 'unsupported', reason: null, fixTool: null, fixPermission: null };
  }

  if (def.requiresCapability !== null) {
    const probe = snapshot.capabilities.find((c) => c.capability === def.requiresCapability);
    if (probe && probe.status === 'unavailable') {
      return {
        kind: 'unsupported',
        reason: probe.reason,
        fixTool: probe.fixTool,
        fixPermission: probe.fixPermission,
      };
    }
    if (probe && probe.status === 'degraded') {
      return { kind: 'partial', note: probe.reason };
    }
  }

  if (byOs.level === 'partial') {
    return { kind: 'partial', note: byOs.note };
  }
  return { kind: 'supported' };
}
