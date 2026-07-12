// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

/** App-boot smoke: window opens, navigation renders, palette toggles. */

import { browser, $, expect } from '@wdio/globals';

describe('Lightning smoke', () => {
  it('boots into the shortcuts grid', async () => {
    const title = await browser.getTitle();
    expect(title).toContain('Lightning');
    await expect($('nav')).toBeDisplayed();
  });

  it('navigates to settings', async () => {
    await $('a[href="/settings"]').click();
    await expect($('select')).toBeDisplayed();
  });

  it('opens the command palette with Ctrl+K', async () => {
    await browser.keys(['Control', 'k']);
    await expect($('[role="dialog"] input')).toBeDisplayed();
    await browser.keys(['Escape']);
  });
});
