// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** A gradient shortcut tile: icon, name, hotkey chip (§9.1). */

import type { ShortcutMetaDto } from '@lightning/bindings';
import { gradientCss, radii } from '@lightning/ui';

export interface ShortcutTileProps {
  meta: ShortcutMetaDto;
  onOpen?: () => void;
}

export function ShortcutTile({ meta, onOpen }: ShortcutTileProps) {
  return (
    <button
      type="button"
      onClick={onOpen}
      data-testid="shortcut-tile"
      className="flex h-32 w-full flex-col justify-between p-4 text-left text-white transition-transform hover:scale-[1.02] focus-visible:outline focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-500"
      style={{ backgroundImage: gradientCss(meta.gradient), borderRadius: radii.tile }}
    >
      <span aria-hidden className="text-2xl">
        {meta.iconGlyph}
      </span>
      <span>
        <span className="block truncate text-sm font-semibold">{meta.name}</span>
        {meta.hotkey !== null ? (
          <kbd className="mt-1 inline-block rounded bg-black/25 px-1.5 py-0.5 text-[10px] font-medium">
            {meta.hotkey}
          </kbd>
        ) : null}
      </span>
    </button>
  );
}
