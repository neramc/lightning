# macOS manual smoke checklist

tauri-driver does not support macOS (§11), so every release candidate runs
this checklist by hand on the oldest and newest supported macOS.

## Setup

- [ ] Fresh install from the signed .dmg; Gatekeeper shows no warnings
- [ ] App launches; grid, gallery, settings render
- [ ] `Lightning-dev` profile is NOT used (release build writes to
      `~/Library/Application Support/Lightning`)

## Shortcuts

- [ ] Create a shortcut with Text → Change Case → Show Result; run: blocks
      light up top-to-bottom, output correct
- [ ] Drag to reorder blocks — spring motion, no dropped frames
- [ ] System Settings → Accessibility → Reduce Motion ON: animations
      collapse to fades; no confetti on run
- [ ] Save, quit, relaunch — the shortcut persists

## Capability honesty (§8.1)

- [ ] A Windows-only action (e.g. Read Registry) shows the grayed block
      with the badge "macOS에서 이 기능을 지원하지 않음" under Korean locale
      and "Not supported on macOS" under English
- [ ] Input-injection action without the Accessibility permission surfaces
      the needs-permission message with a working "Fix it" link

## Automations & tray

- [ ] Interval automation fires while the window is closed (tray mode)
- [ ] Quit from the tray stops triggers

## Import review (§14)

- [ ] Double-click a `.lightning` file: the permission review opens; the
      shortcut does NOT auto-run
- [ ] `lightning://import?...` deep link: same review, no auto-run
