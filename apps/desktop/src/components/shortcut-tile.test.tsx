// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

import { cleanup, render, screen } from '@testing-library/react';
import { afterEach, describe, expect, it, vi } from 'vitest';

import type { ShortcutMetaDto } from '@lightning/bindings';

import { ShortcutTile } from './shortcut-tile';

const meta: ShortcutMetaDto = {
  id: '00000000-0000-0000-0000-000000000001',
  name: 'Morning Routine',
  iconGlyph: '☀️',
  gradient: 'system',
  hotkey: 'Ctrl+Shift+M',
  isAutomation: false,
};

afterEach(cleanup);

describe('ShortcutTile', () => {
  it('renders name, glyph and hotkey chip', () => {
    render(<ShortcutTile meta={meta} />);
    expect(screen.getByText('Morning Routine')).toBeDefined();
    expect(screen.getByText('Ctrl+Shift+M')).toBeDefined();
  });

  it('omits the hotkey chip when none is set', () => {
    render(<ShortcutTile meta={{ ...meta, hotkey: null }} />);
    expect(screen.queryByText('Ctrl+Shift+M')).toBeNull();
  });

  it('invokes onOpen when clicked', () => {
    const onOpen = vi.fn();
    render(<ShortcutTile meta={meta} onOpen={onOpen} />);
    screen.getByTestId('shortcut-tile').click();
    expect(onOpen).toHaveBeenCalledOnce();
  });
});
