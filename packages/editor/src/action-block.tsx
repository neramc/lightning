// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/**
 * One action block on the canvas (§9.1, §9.3): gradient card from the
 * category token, capability badge when the current system can't run it —
 * the block still renders so cross-OS shortcuts stay editable (§8.1).
 */

import { useTranslation } from 'react-i18next';
import { motion } from 'motion/react';

import type { ActionDefDto, CapabilitySnapshotDto } from '@lightning/bindings';
import { actionNameKey } from '@lightning/i18n';
import { Badge, gradientCss, radii, useMotionPrefs } from '@lightning/ui';

import { effectiveSupport } from './support';

export interface ActionBlockProps {
  def: ActionDefDto;
  snapshot: CapabilitySnapshotDto;
  /** Run-animation state driven by run://progress (§9.3). */
  runState?: 'idle' | 'running' | 'success' | 'failed';
  onConfigure?: () => void;
}

export function ActionBlock({ def, snapshot, runState = 'idle', onConfigure }: ActionBlockProps) {
  const { t } = useTranslation();
  const prefs = useMotionPrefs();
  const support = effectiveSupport(def, snapshot);
  const unsupported = support.kind === 'unsupported';

  return (
    <motion.div
      layout
      transition={prefs.spring('smooth')}
      animate={
        runState === 'failed' && !prefs.reducedMotion
          ? { x: [0, -4, 4, -4, 4, 0] } // error shake: ±4 px, 2 cycles (§9.3)
          : { x: 0 }
      }
      className={`relative select-none text-white ${unsupported ? 'opacity-60 grayscale' : ''}`}
      style={{ backgroundImage: gradientCss(def.category), borderRadius: radii.block }}
    >
      <button
        type="button"
        onClick={onConfigure}
        className="flex w-full items-center gap-3 px-4 py-3 text-left focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white"
      >
        <span aria-hidden className="text-lg">
          ⚡
        </span>
        <span className="flex-1">
          <span className="block text-sm font-semibold">{t(actionNameKey(def.id))}</span>
          {unsupported ? (
            <span className="mt-1 block">
              <Badge tone="unsupported">
                {support.fixTool !== null
                  ? t('action.needsTool', { tool: support.fixTool })
                  : support.fixPermission !== null
                    ? t('action.needsPermission', { permission: support.fixPermission })
                    : t('action.unsupportedOnOs', { os: snapshot.osLabel })}
              </Badge>
            </span>
          ) : null}
        </span>
        {runState === 'running' ? (
          <span aria-hidden className="size-2 animate-pulse rounded-full bg-white" />
        ) : null}
        {runState === 'success' ? <span aria-hidden>✓</span> : null}
      </button>
    </motion.div>
  );
}
