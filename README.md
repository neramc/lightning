# ⚡ Lightning

**Shortcuts and automations for your real computer.**

Lightning is a free, open-source, cross-platform desktop app for building drag-and-drop
shortcuts and event-driven automations — inspired by the ergonomics of Apple's Shortcuts
app, but built for the desktop, where far more powerful OS-level actions are possible.

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL%203.0-blue.svg)](LICENSE.md)
[![CI](https://github.com/neramc/lightning/actions/workflows/ci.yml/badge.svg)](https://github.com/neramc/lightning/actions/workflows/ci.yml)

---

## Headline features

- **Shortcuts** — compose a vertical flow of action blocks with parameters, variables and
  control flow, then run it from the app, a global hotkey, the tray, the CLI
  (`lightning run <name>`), a `lightning://` deep link, or a `.lightning` file.
- **Automations** — the same flows, fired automatically by triggers: schedules, file
  changes, Wi-Fi networks, USB devices, login, clipboard changes, webhooks, and more.

## Product principles

- **"It just runs."** A shortcut created on one OS opens on every other; actions the
  current platform can't perform are clearly badged — never silently dropped.
- **Honest capability.** Every action declares exactly what it supports per platform,
  refined by runtime probes (X11 vs Wayland, permissions, installed tools).
- **Feels alive.** Spring-based motion everywhere — always interruptible, GPU-cheap, and
  disabled under reduced-motion.
- **Local-first & private.** No account, no telemetry. Everything lives on your disk.

## Platforms

| Platform                                | Package                                    |
| --------------------------------------- | ------------------------------------------ |
| Windows 10 1809+ / 11 (x64, ARM64, x86) | Dedicated installer (.exe) · portable .zip |
| macOS 11+ (Apple Silicon & Intel)       | .dmg (signed + notarized)                  |
| Ubuntu 22.04+ / Debian 12+              | .deb                                       |
| Arch Linux                              | AUR (`lightning`, `lightning-bin`)         |
| Any glibc Linux                         | AppImage                                   |
| Gentoo · FreeBSD 14+                    | ebuild · port (community, best-effort)     |

## Repository layout

Single monorepo: pnpm workspaces + Turborepo (JS), one Cargo workspace (Rust), one .NET
solution (Windows installer).

```
apps/desktop/       Tauri 2 app — React 19 + TypeScript frontend, thin Rust shell
crates/             All real logic: core engine, actions, triggers, platform layers,
                    scripting (QuickJS), store, IPC types
packages/           Design system, flow editor, generated bindings, i18n, shared config
installer/windows/  C# / WinUI 3 installer + uninstaller + updater
packaging/          .deb, AUR, ebuild, AppImage, FreeBSD port, macOS dmg/notarize
docs/               ADRs, per-action specs, QA checklists
tests/e2e/          WebdriverIO + tauri-driver suites
```

## Development

Prerequisites: Node ≥ 22 LTS, pnpm 10, Rust stable (picked up from
`rust-toolchain.toml`), and on Linux the Tauri system dependencies:

```sh
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

```sh
corepack enable && pnpm install   # one-time setup
pnpm dev                          # run the app with hot reload
pnpm verify                       # full local gate: fmt + clippy + lint + typecheck + tests
pnpm bindings                     # regenerate TS bindings after touching Rust IPC types
```

The full command reference, architecture guide, action catalog, and contribution rules
live in [`CLAUDE.md`](CLAUDE.md) and [`docs/`](docs/).

## Contributing

Contributions are welcome under GPL-3.0 (inbound = outbound, DCO sign-off, no CLA).
See [`CONTRIBUTING.md`](CONTRIBUTING.md).

Lightning is an original work _inspired by_ Apple's Shortcuts. It contains no Apple
assets, icons, or strings, and is not affiliated with or endorsed by Apple.

## License

Copyright (C) 2026 neramc — [GNU General Public License v3.0](LICENSE.md) (`GPL-3.0-only`).
