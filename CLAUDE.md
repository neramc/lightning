# CLAUDE.md — Lightning

> Operating manual for Claude Code (and human contributors) working in this repository.
> Read this file before changing anything. When it conflicts with your defaults, **this file wins**.
> Canonical repo: `https://github.com/neramc/lightning`

---

## 1. Project Overview

**Lightning** is a free, open-source, cross-platform desktop app that lets anyone build **shortcuts and automations** on a real computer — fast. It is heavily inspired by Apple's Shortcuts app (drag-and-drop action blocks, colorful gradient cards, a gallery), but goes much further, because desktops allow far more powerful OS-level actions.

| Field             | Value                                                                                                                    |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------ |
| Product           | Lightning                                                                                                                |
| Owner / Copyright | Copyright (C) 2026 neramc                                                                                                |
| License           | GNU General Public License v3.0 (`GPL-3.0-only`) — see §16                                                               |
| Repository        | `github.com/neramc/lightning` — **single monorepo**: app, engine, installer, packaging, docs                             |
| App stack         | Tauri 2 (Rust backend) + React + TypeScript + Vite frontend                                                              |
| Windows installer | Dedicated **C# / WinUI 3** suite: installer + uninstaller + updater                                                      |
| Platforms         | Windows, macOS, Linux (Ubuntu/Debian, Arch, Gentoo, AppImage), FreeBSD · x86_64 / x86 (32-bit) / ARM — full matrix in §7 |
| Design language   | Apple-Shortcuts-inspired: gradient action cards, spring animations, drag-and-drop flow editor                            |

### The two headline features

1. **Shortcuts** — the user composes a vertical flow of **actions** (blocks) with parameters, variables, and control flow, then runs it on demand: from the app, a global hotkey, the tray, the CLI (`lightning run <name>`), a `lightning://` deep link, or a `.lightning` file.
2. **Automations** — the same flows, fired automatically by **triggers**: schedules, file changes, Wi-Fi networks, USB devices, login, clipboard changes, webhooks, and more (§8.5).

### Product principles

- **"It just runs."** A shortcut created on one OS opens on every other; actions the current platform can't perform are clearly badged — never silently dropped.
- **Honest capability.** Every action declares exactly what it supports. When the current OS/environment can't do it, the UI shows the localized string — Korean source of truth, byte-exact: **`{OS}에서 이 기능을 지원하지 않음`** (§8.1).
- **Feels alive.** Motion is a feature, not decoration — but always interruptible, GPU-cheap, and disabled under reduced-motion (§9).
- **Local-first & private.** No account, no telemetry by default. Everything lives on the user's disk.

---

## 2. Non-Negotiable Rules (read before every task)

1. **GPL-3.0 discipline.** Every new source file starts with an SPDX header (§16). Never add a dependency whose license is GPL-3.0-incompatible (no proprietary SDKs, no SSPL/BUSL, no NC clauses). `cargo deny check` and `pnpm licenses:check` must pass.
2. **Never commit secrets.** No signing keys, tokens, or API keys — not even in tests, fixtures, or docs. CI secrets live only in GitHub Actions secrets.
3. **The capability system is sacred.** Every action MUST declare per-platform support plus a runtime probe where relevant (§6.6, §8.1). The frontend renders capability _data_ from the engine — never hardcode per-OS UI hacks in React.
4. **Type-safe IPC only.** All Tauri commands/events are defined in Rust with `specta` and exported via `pnpm bindings`. Never hand-write invoke strings or duplicate types in TS. `packages/bindings/src/` is generated — do not edit.
5. **No Apple assets, ever.** "Inspired by" only. Never copy Apple icons, SF Symbols, marketing copy, or screenshots, and never use "Shortcuts" / "단축어 앱" as our product name. Original iconography and strings only.
6. **`.lightning` schema stability.** The file format is versioned (`schemaVersion`). Any breaking change requires a version bump + a migration in `crates/core/src/migrate/` + round-trip tests. Old files must always open.
7. **Security defaults hold.** Strict CSP, minimal Tauri capabilities per window, dangerous action classes gated behind per-shortcut permission grants (§14). Never widen these "to make a test pass".
8. **Cross-platform is everyone's job.** Code touching `crates/platform-*` must compile for all Tier-1 targets — run `pnpm check:targets` before finishing. Never break one Tier-1 platform to fix another.
9. **All user-facing strings go through i18n** (`packages/i18n`). English is the source locale; Korean (`ko`) must be 100% before release. No string literals in JSX, Rust notifications, or installer XAML.
10. **Animations follow §9**: transform/opacity only, spring tokens from the design system, `prefers-reduced-motion` respected — including inside the WinUI installer.
11. **Ask before destructive git.** Never force-push, never rewrite `main`/`release/*` history, never delete branches you did not create in this session.
12. **Definition of Done** (§18) applies to every task — unformatted, unlinted, untested, undocumented code is not done.

---

## 3. Repository Layout (monorepo)

pnpm workspaces + Turborepo for JS · a single Cargo workspace for Rust · one .NET solution for the installer.

```
lightning/
├─ CLAUDE.md                    ← you are here
├─ LICENSE                      # GPL-3.0 full text
├─ package.json                 # pnpm root — scripts orchestrate everything
├─ pnpm-workspace.yaml
├─ turbo.json
├─ Cargo.toml                   # [workspace] for all crates
├─ rust-toolchain.toml          # pinned stable toolchain + targets
├─ .github/
│  ├─ workflows/                # ci.yml · nightly.yml · release.yml
│  └─ CODEOWNERS
├─ apps/
│  └─ desktop/                  # the Tauri application
│     ├─ src/                   # React frontend (views, routing, state)
│     ├─ src-tauri/             # THIN Tauri shell: setup, command glue, tray, windows
│     │  ├─ tauri.conf.json
│     │  ├─ capabilities/       # per-window permission sets
│     │  └─ src/
│     └─ index.html · vite.config.ts
├─ crates/                      # all real logic lives here, not in src-tauri
│  ├─ core/                     # lightning-core: shortcut model, content types, engine, migrations
│  ├─ actions/                  # lightning-actions: Action trait, registry, built-in actions
│  ├─ triggers/                 # lightning-triggers: Trigger trait, scheduler, event bus
│  ├─ platform/                 # lightning-platform: OS abstraction traits + capability probe
│  ├─ platform-windows/         # Win32 / WinRT implementations
│  ├─ platform-macos/           # AppKit / CoreFoundation / AppleScript implementations
│  ├─ platform-linux/           # X11 / Wayland / D-Bus / systemd implementations
│  ├─ platform-bsd/             # FreeBSD (Tier 3, best-effort)
│  ├─ scripting/                # embedded JS runtime (rquickjs) for "Run JavaScript"
│  ├─ store/                    # persistence: .lightning files + SQLite index + run history
│  └─ ipc-types/                # specta-annotated shared types — single source of truth
├─ packages/
│  ├─ ui/                       # design system: tokens, primitives, motion presets
│  ├─ editor/                   # flow-editor canvas (dnd-kit, virtualized)
│  ├─ bindings/                 # GENERATED TS bindings from Rust — never edit by hand
│  ├─ i18n/                     # locales (en = source, ko = required)
│  └─ config/                   # shared eslint / tsconfig / prettier presets
├─ installer/
│  └─ windows/                  # C# WinUI 3 suite — Lightning.Installer.sln
│     ├─ Lightning.Installer/       # install wizard UI
│     ├─ Lightning.Uninstaller/     # uninstall UI + cleanup
│     ├─ Lightning.Updater/         # background update check + apply + rollback
│     └─ Lightning.Deploy.Core/     # shared: payload, registry, manifest, rollback logic
├─ packaging/
│  ├─ linux/  (debian/ · arch/ · gentoo/ · appimage/)
│  ├─ bsd/    (FreeBSD port)
│  └─ macos/  (dmg layout, entitlements, notarize scripts)
├─ scripts/                     # sync-version.mjs · gen-bindings.mjs · release helpers
├─ docs/                        # ADRs, per-action specs, QA checklists
└─ tests/
   └─ e2e/                      # WebdriverIO + tauri-driver suites
```

**Where does my change go?**

- New action logic → `crates/actions` (+ `crates/platform-*` if OS-specific) — recipe in §8.6
- New trigger → `crates/triggers` — recipe in §8.7
- Engine / variables / content-type semantics → `crates/core`
- Anything users see → `apps/desktop/src` + `packages/ui` / `packages/editor`
- IPC surface → `crates/ipc-types`, then regenerate bindings
- Windows install/update behavior → `installer/windows`
- Distro packaging → `packaging/`

---

## 4. Tech Stack & Version Baselines

Exact versions are pinned in lockfiles / toolchain files; this table is the intent. Do not downgrade without an ADR in `docs/architecture/`.

### App (apps/desktop + crates/*)

| Layer         | Choice                                                    | Notes                                                                                                                                                                                   |
| ------------- | --------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Shell         | **Tauri 2.x**                                             | official plugins: global-shortcut, autostart, single-instance, deep-link, notification, dialog, fs, shell (scoped), clipboard-manager, updater (macOS/AppImage only); tray via core API |
| Rust          | stable, pinned in `rust-toolchain.toml`, **edition 2024** | `clippy -D warnings` + `rustfmt` enforced                                                                                                                                               |
| Async         | tokio 1 (multi-thread)                                    | engine + triggers run on tokio; never block the main thread                                                                                                                             |
| Errors        | `thiserror` 2 in crates · `anyhow` 1 at binary edges      | no `unwrap()` outside tests                                                                                                                                                             |
| Serialization | serde / serde_json                                        | `.lightning` files are pretty-printed JSON                                                                                                                                              |
| IPC typegen   | `specta` + `tauri-specta`                                 | `pnpm bindings` regenerates TS                                                                                                                                                          |
| Embedded JS   | `rquickjs` (QuickJS)                                      | powers the cross-platform "Run JavaScript" action, sandboxed                                                                                                                            |
| Storage       | `rusqlite` (bundled) index + JSON files                   | files are truth, DB is a rebuildable cache (§6.3)                                                                                                                                       |
| File watching | `notify`                                                  | debounced inside `crates/triggers`                                                                                                                                                      |
| Logging       | `tracing` + rotating file appender                        | user-visible run log ≠ dev log                                                                                                                                                          |
| Media         | **ffmpeg sidecar** binaries per target                    | GPL builds — license-compatible (§16)                                                                                                                                                   |

### Frontend (apps/desktop/src + packages/*)

| Layer       | Choice                                              | Notes                                                                                  |
| ----------- | --------------------------------------------------- | -------------------------------------------------------------------------------------- |
| Framework   | React 19 + TypeScript 5 (strict) + Vite 7           |                                                                                        |
| State       | Zustand (slices per domain)                         | no Redux                                                                               |
| Routing     | TanStack Router                                     | type-safe routes: `/shortcuts`, `/editor/$id`, `/automations`, `/gallery`, `/settings` |
| Drag & drop | dnd-kit                                             | editor canvas + list reordering                                                        |
| Animation   | `motion` (Framer Motion successor) + CSS transforms | motion tokens live in `packages/ui/motion.ts` (§9.2)                                   |
| Styling     | Tailwind CSS 4 + design tokens in `packages/ui`     | category gradients are tokens, never ad-hoc hex                                        |
| i18n        | i18next + react-i18next                             | `{{var}}` interpolation, ICU plurals                                                   |
| Long lists  | TanStack Virtual                                    | shortcut grid, action library, run logs are virtualized                                |

### Windows installer suite (installer/windows)

| Layer      | Choice                                                                                                                                                                                                 | Notes                                                             |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------------------------------------------------------- |
| Runtime    | **.NET 10 (LTS)**, C# 14, `<Nullable>enable</Nullable>`, warnings-as-errors                                                                                                                            |                                                                   |
| UI         | WinUI 3 (Windows App SDK latest stable), unpackaged, self-contained publish                                                                                                                            | Mica backdrop, per-monitor DPI, animated progress                 |
| MVVM       | CommunityToolkit.Mvvm                                                                                                                                                                                  | `[ObservableProperty]`, `[RelayCommand]`; no logic in code-behind |
| Hard rules | no `async void` (except event handlers) · no `.Result`/`.Wait()` · one shared `HttpClient` · `using`/`await using` for disposables · `throw;` not `throw ex;` · CancellationToken flows through all IO |                                                                   |

### Tooling

Node ≥ 22 LTS · pnpm 10 · Turborepo · Rust stable (toolchain file) · .NET 10 SDK · GitHub Actions runners for Windows/macOS/Linux

---

## 5. Setup & Everyday Commands

### One-time setup

```sh
corepack enable && pnpm install          # JS workspace
# Rust: rustup picks the toolchain + targets from rust-toolchain.toml automatically
# Linux build deps (Debian/Ubuntu):
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libssl-dev libayatana-appindicator3-dev librsvg2-dev
# Installer work additionally needs: .NET 10 SDK + Windows App SDK (Windows machine/VM)
```

### Commands (run from repo root unless noted)

| Task                                                                     | Command                                                             |
| ------------------------------------------------------------------------ | ------------------------------------------------------------------- |
| Run the app (dev, hot reload)                                            | `pnpm dev`                                                          |
| Frontend-only dev server                                                 | `pnpm --filter desktop dev:web`                                     |
| Production build (current OS)                                            | `pnpm build`                                                        |
| **Regenerate TS bindings** after touching any Rust command/event/type    | `pnpm bindings` (CI checks freshness with `pnpm bindings --check`)  |
| Frontend unit tests                                                      | `pnpm test` (Vitest)                                                |
| Rust tests (all crates)                                                  | `cargo test --workspace` (or `cargo nextest run`)                   |
| Lint / typecheck TS                                                      | `pnpm lint` · `pnpm typecheck`                                      |
| Rust lint                                                                | `cargo clippy --workspace --all-targets -- -D warnings`             |
| Format                                                                   | `pnpm fmt` · `cargo fmt --all`                                      |
| Cross-target compile check (Tier 1)                                      | `pnpm check:targets`                                                |
| License audit                                                            | `cargo deny check` + `pnpm licenses:check`                          |
| E2E (Linux/Windows only)                                                 | `pnpm e2e` (requires `tauri-driver`)                                |
| Installer build                                                          | `dotnet build installer/windows/Lightning.Installer.sln -c Release` |
| Installer tests                                                          | `dotnet test installer/windows`                                     |
| **Full local gate — run before declaring any task done**                 | `pnpm verify` = fmt-check + clippy + lint + typecheck + unit tests  |
| Sync version across package.json / Cargo.toml / tauri.conf.json / csproj | `pnpm version:sync <x.y.z>`                                         |

Notes:

- `pnpm dev` wraps `tauri dev`. Do not run it simultaneously with `pnpm e2e` (port 1420 clash).
- `tauri-driver` does **not** support macOS — macOS coverage is unit tests + the manual smoke checklist (§11).
- If a command needs the app data dir, dev builds use an isolated `Lightning-dev` profile so you never touch a real user's shortcuts.

---

## 6. Architecture

### 6.1 Big picture

```
┌────────────────────────── apps/desktop ───────────────────────────┐
│  React UI · editor · gallery · automations · settings             │
│      └── packages/bindings (generated, type-safe)                 │
│                 │  invoke / events (tauri-specta)                  │
├─────────────────▼───────── src-tauri (thin) ──────────────────────┤
│  command glue · windows/tray · native permission prompts          │
├─────────────────▼─────────────── crates ──────────────────────────┤
│  core (model + engine) ─ actions (registry) ─ triggers (bus)      │
│            │                   │                   │              │
│            └───── platform (traits + capability probe) ───────────│
│          platform-windows / -macos / -linux / -bsd                │
│  scripting (QuickJS) · store (files + SQLite index)               │
└────────────────────────────────────────────────────────────────────┘
   installer/windows (.NET 10 / WinUI 3): install · uninstall · update
```

Rule of thumb: **src-tauri stays thin.** If a function is more than glue, it belongs in a crate.

### 6.2 Content model (Apple-style "magic variables")

Every action consumes and produces typed **Content**: `Text · Number · Boolean · Date · List · Dictionary · File · Image · URL · RichText · Error · Nothing`. `crates/core/src/content/` owns:

- the type definitions + serde,
- the **coercion table** (e.g. Number→Text, File→Image when decodable, List→Text join). Coercions are explicit and unit-tested — never add an implicit lossy coercion.
- Every step's output is addressable downstream as a magic variable; named variables exist via Set/Get Variable.

### 6.3 Shortcut model & the `.lightning` file format

- A shortcut = ordered tree of `Step { action_id, params, uuid }`; control-flow steps (If / Repeat / Menu) own child steps.
- Serialized as pretty JSON with a top-level `schemaVersion` into the user data dir (§6.9), one file per shortcut.
- `crates/store` maintains an SQLite index (names, tags, icon, hotkey, run stats, history). **Files are the source of truth; the DB is a cache** — a full rebuild from files must always succeed (`store::reindex()` is tested).
- Import paths: `.lightning` double-click (file association) and `lightning://import?...` deep link → both always open the **permission review screen** first; imports are never auto-run (§14).

### 6.4 Execution engine (crates/core)

- Tree-walking async interpreter over steps (tokio); each action's `execute(ctx)` is awaited.
- `RunContext` carries: variable scope, coercer, cancellation token, granted permissions, `CapabilitySnapshot`, structured run logger.
- Per-step error policy: `Stop` (default) · `Continue` · `Retry { n, backoff }` — user-configurable per step.
- The engine emits `run://progress` events (step started / finished / output preview) which drive the editor's run animation (§9.3).
- Hard limits: `Run Shortcut` recursion depth ≤ 16 · loop iteration cap (default 10,000, user-raisable) · per-run wall-clock timeout for automations. Runaway automations must be impossible.

### 6.5 Action system (crates/actions)

```rust
trait Action: Send + Sync {
    fn def(&self) -> ActionDef;   // id, category, icon, param schema, support matrix, permission class
    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError>;
}
```

- `ActionDef.supports: PlatformSupport` — per-OS `Full | Partial(note) | None` (§8.1).
- `ActionDef.permission: PermissionClass` (§14).
- Actions self-register into the registry; the frontend fetches the whole catalog (defs only) through one `list_actions` command and renders everything from data. **The frontend never hardcodes an action list.**
- Param schemas are declared once in Rust and exported to TS — the editor auto-renders parameter UIs (text, number, enum, toggle, file picker, variable slot) from the schema.

### 6.6 Capability probing (crates/platform)

Compile-target support is necessary but not sufficient. On startup and on relevant system changes, the platform crate produces a `CapabilitySnapshot`, e.g.:

- **Linux**: X11 vs Wayland · ydotool/libei portal present (input injection) · screenshot portal · NetworkManager (`nmcli`) · MPRIS players · notification daemon capabilities
- **macOS**: Accessibility permission · Screen Recording permission · per-app Automation (AppleEvents) consent · `shortcuts` CLI availability
- **Windows**: elevation state · PowerShell version · optional features
  Effective status = static support ∩ runtime probe. `Unsupported { reason }` carries a localized, human-readable reason surfaced in the UI. Capability changes emit `capability://changed` so badges update live.

### 6.7 Triggers & automation runtime (crates/triggers)

- Each `Trigger` runs as a tokio task publishing `TriggerEvent` to a broadcast bus; the scheduler matches events to enabled automations and enqueues runs with per-automation debounce/cooldown (default 2 s).
- Automations persist as `.lightning` files with a `trigger` block; run history lives in SQLite (ring buffer, default 1,000 entries).
- The app runs headless in tray mode (`--tray`), auto-started at login via the autostart plugin. Closing the window keeps triggers alive; only Quit stops them.

### 6.8 IPC contract

- Commands + events are declared in `crates/ipc-types` with specta annotations; `pnpm bindings` writes `packages/bindings/src/`. CI fails on stale bindings.
- Event naming: `run://progress`, `triggers://fired`, `capability://changed`, `store://changed`.
- Payloads stay backward-compatible within a minor version — updater flows can briefly mismatch frontend/backend.

### 6.9 Data & paths (per-OS conventions)

| Purpose                 | Windows                                                      | macOS                                               | Linux/BSD                            |
| ----------------------- | ------------------------------------------------------------ | --------------------------------------------------- | ------------------------------------ |
| Shortcuts & automations | `%APPDATA%\Lightning\shortcuts`                              | `~/Library/Application Support/Lightning/shortcuts` | `$XDG_DATA_HOME/lightning/shortcuts` |
| Index / history DB      | `…\Lightning\index.db`                                       | same root                                           | same root                            |
| Logs (rotated, 7 days)  | `…\Lightning\logs`                                           | same root                                           | `$XDG_STATE_HOME/lightning/logs`     |
| Settings                | `settings.json` — schema-versioned, atomic write-then-rename | ←                                                   | ←                                    |

### 6.10 Windows installer suite (installer/windows)

- **Installer** — WinUI 3 wizard (Mica, animated progress). Steps: GPL-3.0 license → install path → options (autostart · add to PATH · `.lightning` file association · `lightning://` protocol · Start-menu/desktop shortcuts) → install → launch. Writes `HKCU\Software\neramc\Lightning` + the uninstall registry entry. Per-user by default; per-machine (elevated, `HKLM`) is an explicit option.
- **Uninstaller** — removes payload, registry, shortcuts, protocol handlers; asks whether to keep user data (**default: keep**).
- **Updater** — background check against GitHub Releases `latest.json` (per target/arch/channel), downloads the full package, verifies **minisign (ed25519) signature + SHA-256**, stages, applies on next app start with rollback (previous payload kept until first successful launch). Channels: stable · beta · nightly. Downgrades are rejected (version monotonicity).
- The installer never contains secrets; all signing happens in CI.

### 6.11 Update strategy per platform

| Platform                      | Mechanism                                                                                                  |
| ----------------------------- | ---------------------------------------------------------------------------------------------------------- |
| Windows                       | `Lightning.Updater` (§6.10) — the only auto-update path on Windows                                         |
| macOS (.dmg)                  | tauri-plugin-updater (signed + notarized)                                                                  |
| Linux AppImage                | tauri-plugin-updater                                                                                       |
| deb / Arch / Gentoo / FreeBSD | distro package manager (repo, AUR, ebuild, port) — the in-app check only **notifies**, never self-modifies |

---

## 7. Platform Support Matrix

| Target                     | Arch                    | Tier  | Package                                    | CI                                    |
| -------------------------- | ----------------------- | ----- | ------------------------------------------ | ------------------------------------- |
| Windows 10 1809+ / 11      | x86_64                  | **1** | Dedicated installer (.exe) + portable .zip | build + test + e2e · release-blocking |
| Windows 11                 | aarch64                 | 2     | installer                                  | build + smoke                         |
| Windows 10+                | i686 (32-bit)           | 3     | portable .zip                              | build only                            |
| macOS 11+                  | aarch64 (Apple Silicon) | **1** | .dmg (signed + notarized)                  | build + unit tests                    |
| macOS 11+                  | x86_64                  | **1** | .dmg                                       | build + unit tests                    |
| Ubuntu 22.04+ / Debian 12+ | x86_64                  | **1** | .deb                                       | build + test + e2e                    |
| Ubuntu / Debian            | aarch64                 | 2     | .deb                                       | build                                 |
| Arch Linux                 | x86_64                  | 2     | AUR: `lightning`, `lightning-bin`          | build                                 |
| Any glibc Linux            | x86_64 / aarch64        | 2     | AppImage                                   | build + smoke                         |
| Linux                      | armv7 / i686            | 3     | AppImage / .deb                            | build only                            |
| Gentoo                     | x86_64                  | 3     | ebuild (`packaging/linux/gentoo`)          | manual                                |
| FreeBSD 14+                | x86_64                  | 3     | port (`packaging/bsd`)                     | manual · best-effort                  |

- **macOS 32-bit does not exist** (dropped in macOS 10.15) — never attempt that target.
- Tier 1 breakage blocks release. Tier 2 builds in CI; failures open issues but don't block. Tier 3 is best-effort/community.
- X11 vs Wayland is a **runtime capability** matter (§6.6), never a build split.

---

## 8. Action Catalog & Capability Rules

### 8.1 The "unsupported" rule (user-visible, non-negotiable)

When an action — or one of its options — is unavailable on the current OS/environment:

- The block still renders in the editor (grayed + badge) so cross-OS shortcuts stay editable and shareable.
- Badge, tooltip, and runtime error all use i18n key `action.unsupportedOnOs`:
  - `ko` (source of the requirement — keep byte-exact): `{{os}}에서 이 기능을 지원하지 않음`
  - `en`: `Not supported on {{os}}`
  - `{{os}}` renders as `Windows` / `macOS` / `Linux` / `FreeBSD`, with an environment suffix when the limit is environmental, e.g. `Linux (Wayland)`.
- Runtime: the engine returns `ActionError::Unsupported { os, reason }`; the default error policy stops the run with that message; `Continue` policy skips the step with a logged warning.
- If the limitation is a **missing permission or tool** rather than the OS itself, use `action.needsPermission` / `action.needsTool` instead, with a "Fix it" button (opens OS settings or shows the install hint). Never blame the OS for a solvable setup issue.

### 8.2 Legend

✅ full · ⚠️ partial/conditional (see note — usually a permission, tool, or environment dependency) · ❌ not supported (badge shown) · ◆ Lightning-original (no Apple equivalent). Columns: **W**indows / **M**acOS / **L**inux; FreeBSD generally follows L minus systemd (§8.4).
Summaries below are the governing overview; the canonical per-action spec lives in `docs/actions/`.

### 8.3 Universal actions (Apple Shortcuts parity + desktop power)

**A. Control flow & scripting — engine-level, ✅ on all platforms**
If / Otherwise · Repeat (n times) · Repeat with Each · While ◆ · Wait · Wait Until ◆ · Exit Shortcut · Stop and Output · Choose from Menu · Choose from List · Run Shortcut (sub-shortcut) · Comment · Nothing · Set / Get / Add to Variable · Get Type · Count · Get Item from List · Filter List · Sort List · Dictionary · Get / Set Dictionary Value · Show Result · Show Alert · Show Notification · Ask for Input (text / number / date / file / choice) · Get Name / Set Name

**B. Text — ✅ on all platforms**
Text literal · Combine · Split · Replace (regex option) · Match Text / Get Group from Matched Text · Change Case · Trim ◆ · Text Statistics ◆ (chars/words/lines) · Generate UUID · Markdown → HTML / Rich Text · HTML → Plain Text · URL Encode/Decode · Base64 Encode/Decode · Hash Text (MD5/SHA-1/SHA-256)
Exception: **Translate Text — ⚠️ all** (requires a translation provider configured in Settings; no keyless default exists).

**C. Math & numbers — ✅ all** Calculate · Random Number · Round · Format Number · Convert Measurement (units) · List Statistics ◆ (min/max/avg/sum/median)

**D. Date & time — ✅ all** Current Date · Adjust Date · Format Date (ICU patterns) · Time Between Dates · Get Dates from Input · Set Timer ◆ (fires a notification)

**E. Web & network — ✅ all** Open URL · Get Contents of URL (GET/POST/PUT/PATCH/DELETE, headers, JSON/form body) · Download File · Expand URL · Get URLs from Input · Get Article from Web Page (readability extraction) · Get Items from RSS Feed · Get IP Address (local/public) · Get Network Details · Ping ◆ · DNS Lookup ◆ · Run Script over SSH · Send Webhook ◆

**F. Files & folders**

| Action                                                   | W   | M   | L   | Notes                                                                     |
| -------------------------------------------------------- | --- | --- | --- | ------------------------------------------------------------------------- |
| Get File · Select File (dialog) · Save File              | ✅  | ✅  | ✅  |                                                                           |
| Read / Write / Append Text File                          | ✅  | ✅  | ✅  |                                                                           |
| Delete Files                                             | ✅  | ✅  | ✅  | → Trash by default; permanent delete is opt-in. L: FreeDesktop trash spec |
| Move / Copy / Rename · Create Folder                     | ✅  | ✅  | ✅  |                                                                           |
| Get Folder Contents (recursive, glob) · Get File Details | ✅  | ✅  | ✅  |                                                                           |
| Zip / Extract Archive                                    | ✅  | ✅  | ✅  | zip + tar.gz native; 7z/rar extract ⚠️ if tool installed                  |
| Reveal in File Manager                                   | ✅  | ✅  | ✅  | L: `org.freedesktop.FileManager1` D-Bus, xdg-open fallback                |
| Get Selected Files in File Manager                       | ✅  | ✅  | ❌  | W: Explorer Shell COM · M: Finder AppleScript · L: no cross-DE API        |
| Empty Trash                                              | ✅  | ✅  | ✅  | L: `gio trash --empty`                                                    |
| Hash File · Base64 File · Get Disk Usage ◆               | ✅  | ✅  | ✅  |                                                                           |
| Mount / Unmount Volume                                   | ⚠️  | ⚠️  | ⚠️  | W: ISO/VHD native · M: hdiutil · L: udisks2                               |

**G. Clipboard**

| Action                                       | W   | M   | L   | Notes                                                                    |
| -------------------------------------------- | --- | --- | --- | ------------------------------------------------------------------------ |
| Get Clipboard (text/image/file list)         | ✅  | ✅  | ⚠️  | Wayland: file-list formats vary by compositor                            |
| Copy to Clipboard (text/image/files) · Clear | ✅  | ✅  | ✅  |                                                                          |
| Get Clipboard History ◆                      | ⚠️  | ⚠️  | ⚠️  | Lightning's own history — recorded only while that permission is granted |

**H. Images & graphics**

| Action                                                                           | W   | M   | L   | Notes                                                              |
| -------------------------------------------------------------------------------- | --- | --- | --- | ------------------------------------------------------------------ |
| Resize / Crop / Rotate / Flip · Combine · Overlay Text/Image · Get Image Details | ✅  | ✅  | ✅  |                                                                    |
| Convert Image (png/jpeg/webp/gif/bmp/tiff)                                       | ✅  | ✅  | ✅  | HEIC decode ⚠️ on W/L                                              |
| Create QR / Barcode · Scan QR from Image                                         | ✅  | ✅  | ✅  |                                                                    |
| Extract Text from Image (OCR)                                                    | ✅  | ✅  | ⚠️  | W: Windows.Media.Ocr · M: Vision · L: Tesseract if installed       |
| Remove Background                                                                | ⚠️  | ✅  | ⚠️  | M: Vision · W/L: optional ONNX model download                      |
| Take Screenshot (screen/window/region)                                           | ✅  | ⚠️  | ⚠️  | M: Screen Recording permission · L: X11 ✅, Wayland via xdg portal |
| Pick Color from Screen ◆                                                         | ✅  | ⚠️  | ⚠️  | same constraints as screenshot                                     |

**I. PDF & documents — ✅ all** Make PDF (from text/HTML/images) · Merge PDF · Split PDF · Get Text from PDF · Print / Print to PDF (L: CUPS) · Make Rich Text from Markdown

**J. Audio, video & speech**

| Action                            | W   | M   | L   | Notes                                                               |
| --------------------------------- | --- | --- | --- | ------------------------------------------------------------------- |
| Play Sound · Get Media Details    | ✅  | ✅  | ✅  |                                                                     |
| Record Audio                      | ⚠️  | ⚠️  | ⚠️  | microphone permission everywhere                                    |
| Trim Media · Encode/Convert Media | ✅  | ✅  | ✅  | ffmpeg sidecar                                                      |
| Speak Text (TTS)                  | ✅  | ✅  | ⚠️  | W: WinRT/SAPI · M: AVSpeech · L: speech-dispatcher if present       |
| Dictate Text (STT)                | ⚠️  | ⚠️  | ⚠️  | optional local whisper model download ◆                             |
| Play / Pause / Next / Previous    | ✅  | ✅  | ✅  | W: media keys + SMTC · L: MPRIS                                     |
| Get Now Playing                   | ✅  | ⚠️  | ✅  | W: GSMTC · M: Music.app via AppleScript, other apps vary · L: MPRIS |
| Set System Volume / Mute          | ✅  | ✅  | ✅  |                                                                     |
| Set Audio Output / Input Device   | ✅  | ✅  | ✅  | M: CoreAudio · L: pactl/PipeWire                                    |

**K. Apps & windows**

| Action                                                      | W   | M   | L   | Notes                                                                                                                                     |
| ----------------------------------------------------------- | --- | --- | --- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| Open App · Open File With                                   | ✅  | ✅  | ✅  |                                                                                                                                           |
| Quit App (graceful) · Force Quit / Kill Process             | ✅  | ✅  | ✅  |                                                                                                                                           |
| Hide App                                                    | ⚠️  | ✅  | ⚠️  | native hide is macOS-only; W/L fall back to minimize                                                                                      |
| Get Running Apps · Get Frontmost App                        | ✅  | ✅  | ⚠️  | L: X11 ✅ · Wayland limited                                                                                                               |
| Focus App or Window                                         | ✅  | ⚠️  | ⚠️  | M: Accessibility perm · L: X11 ✅ / Wayland compositor-dependent                                                                          |
| Move / Resize / Minimize / Maximize / Fullscreen / Center ◆ | ✅  | ⚠️  | ⚠️  | M: AX perm · L: X11 ✅ / Wayland ⚠️                                                                                                       |
| Snap Window (halves/quarters) ◆                             | ✅  | ⚠️  | ⚠️  | M: computed frames via AX                                                                                                                 |
| Set Always on Top                                           | ✅  | ⚠️  | ⚠️  | L: X11 ✅                                                                                                                                 |
| Switch Virtual Desktop · Move Window to Desktop             | ⚠️  | ⚠️  | ⚠️  | W: partially documented COM · M: no public Spaces API, keystroke fallback · L: wmctrl (X11) ✅, sway/i3/Hyprland IPC ✅, GNOME Wayland ⚠️ |
| Get Window List · Take Window Screenshot                    | ✅  | ⚠️  | ⚠️  | M: perms · L: portal on Wayland                                                                                                           |

**L. System & device**

| Action                                                 | W   | M   | L   | Notes                                                                                               |
| ------------------------------------------------------ | --- | --- | --- | --------------------------------------------------------------------------------------------------- |
| Lock Screen · Log Out                                  | ✅  | ✅  | ✅  |                                                                                                     |
| Sleep / Shut Down / Restart                            | ✅  | ✅  | ✅  | confirmation dialog by default                                                                      |
| Keep Awake (duration / while running) ◆                | ✅  | ✅  | ✅  | W: SetThreadExecutionState · M: caffeinate/IOKit · L: systemd-inhibit                               |
| Get Battery / Power Status · Power Source Changed info | ✅  | ✅  | ✅  |                                                                                                     |
| Set Power Mode                                         | ✅  | ⚠️  | ⚠️  | W: power plan/overlay · M: Low Power Mode needs admin (pmset) · L: power-profiles-daemon if present |
| Set Brightness                                         | ✅  | ✅  | ⚠️  | external monitors via DDC/CI ⚠️ everywhere · L: brightnessctl                                       |
| Toggle Dark Mode                                       | ✅  | ✅  | ⚠️  | L: GNOME/KDE only                                                                                   |
| Night Light / Night Shift / Night Color                | ✅  | ✅  | ⚠️  |                                                                                                     |
| Set Wallpaper                                          | ✅  | ✅  | ⚠️  | L: DE-specific backends                                                                             |
| Do Not Disturb / Focus                                 | ⚠️  | ⚠️  | ⚠️  | W: Focus Assist (limited API) · M: via Shortcuts-CLI interop · L: notification daemon               |
| Toggle Wi-Fi · Connect to SSID                         | ✅  | ✅  | ⚠️  | W: netsh · M: networksetup · L: nmcli (NetworkManager)                                              |
| Toggle Bluetooth                                       | ⚠️  | ⚠️  | ✅  | W: Radios API · M: private API · L: bluetoothctl                                                    |
| Send Notification (buttons, text input)                | ✅  | ⚠️  | ⚠️  | button/action support varies                                                                        |
| Get Device Details (host/OS/CPU/RAM/uptime)            | ✅  | ✅  | ✅  |                                                                                                     |
| Get / Set Environment Variable                         | ✅  | ✅  | ✅  | session ✅ · persist ⚠️ (W: registry · M/L: shell rc)                                               |
| Get Display Info · Eject Disk                          | ✅  | ✅  | ✅  |                                                                                                     |
| Set Resolution / Rotate Display                        | ⚠️  | ⚠️  | ⚠️  | L: X11 via xrandr ✅                                                                                |

**M. Input & UI automation — permission class `input` (§14)**
Type Text · Press Key/Hotkey · Mouse Move/Click/Scroll/Drag · Get Mouse Position
→ **W ✅** (SendInput) · **M ⚠️** (Accessibility permission required) · **L ⚠️** (X11 ✅; Wayland needs ydotool daemon or libei portal — otherwise `Unsupported` with reason)

**N. Scripting bridges**

| Action                               | W   | M   | L   | Notes                                                                               |
| ------------------------------------ | --- | --- | --- | ----------------------------------------------------------------------------------- |
| Run JavaScript ◆                     | ✅  | ✅  | ✅  | embedded QuickJS, sandboxed — no fs/net unless the shortcut holds those permissions |
| Run Shell Script (sh/bash/zsh)       | ⚠️  | ✅  | ✅  | W: via WSL or Git-Bash if present                                                   |
| Run PowerShell                       | ✅  | ⚠️  | ⚠️  | M/L: pwsh if installed                                                              |
| Run CMD                              | ✅  | ❌  | ❌  |                                                                                     |
| Run AppleScript · Run JXA            | ❌  | ✅  | ❌  |                                                                                     |
| Run Apple Shortcut (`shortcuts run`) | ❌  | ✅  | ❌  | direct interop with Apple's app                                                     |
| Run Python                           | ⚠️  | ⚠️  | ⚠️  | detected interpreter                                                                |

**O. Communication & sharing**

| Action                                      | W   | M   | L   | Notes                            |
| ------------------------------------------- | --- | --- | --- | -------------------------------- |
| Compose Email (default client, attachments) | ✅  | ✅  | ✅  |                                  |
| Send Email (SMTP profile)                   | ✅  | ✅  | ✅  | credentials in OS keychain (§14) |
| AirDrop                                     | ❌  | ✅  | ❌  | badge on W/L                     |
| System Share Sheet                          | ✅  | ✅  | ❌  |                                  |
| Send SMS / iMessage                         | ❌  | ❌  | ❌  | needs a phone — badge explains   |

**P. Productivity (calendar / tasks / contacts)**

| Action                                     | W   | M   | L   | Notes                                          |
| ------------------------------------------ | --- | --- | --- | ---------------------------------------------- |
| Create Event (.ics → default calendar app) | ✅  | ✅  | ✅  |                                                |
| Create Event (native calendar)             | ⚠️  | ✅  | ❌  | W: Outlook COM if present · M: EventKit + perm |
| Get Upcoming Events                        | ⚠️  | ✅  | ❌  |                                                |
| Create Reminder                            | ❌  | ✅  | ❌  | M: Reminders + perm                            |
| Find Contacts                              | ⚠️  | ✅  | ❌  |                                                |
| Create Note                                | ❌  | ✅  | ❌  | M: Notes via AppleScript                       |

**Q. Location & weather**

| Action                         | W   | M   | L   | Notes                              |
| ------------------------------ | --- | --- | --- | ---------------------------------- |
| Get Approximate Location (IP)  | ✅  | ✅  | ✅  |                                    |
| Get Precise Location           | ✅  | ✅  | ⚠️  | OS consent · L: GeoClue if present |
| Get Current Weather / Forecast | ✅  | ✅  | ✅  | Open-Meteo, keyless                |

### 8.4 OS-exclusive actions (each shows the §8.1 badge on the other platforms)

**Windows-only (`crates/platform-windows`)**
Read/Write Registry (HKCU freely · HKLM ⚠️ elevation) · WMI Query · Manage Windows Service (start/stop/query) · Create Scheduled Task · Toast with buttons & text input · Set Default Audio Device · Open `ms-settings:` page · Launch Store/UWP app by AUMID · Virtual Desktop ops ⚠️ · Empty Recycle Bin · Set Power Plan · Map Network Drive · Mount ISO/VHD · Windows Search query · Focus Assist ⚠️ · Winget install/upgrade (consent prompt) · Get Windows Update status ⚠️ · Apply Snap Layout

**macOS-only (`crates/platform-macos`)**
Run AppleScript / JXA · **Run Apple Shortcut** (bridge to the original app) · Spotlight Search (`mdfind`) · Finder Tags get/set · Get Selected Finder Items · Quick Look preview · Set Focus Mode ⚠️ · `defaults` read/write · Create launchd agent · Keychain get/set (user consent, Touch ID) · Dock add/remove/spacer · Say (system voices) · caffeinate · AirDrop send · Start Time Machine backup · Toggle Stage Manager ⚠️ · Click Menu-Bar Item (AX) · Music.app control · Safari current URL / tab list (Automation consent) · Clear Notification Center · Set Hot Corners ⚠️ · brew install (consent) ⚠️

**Linux-only (`crates/platform-linux`)**
Manage systemd units & timers (user scope; system via polkit) · D-Bus Method Call (session/system) · gsettings get/set · KDE config (kwriteconfig6/qdbus) · Notification with actions (libnotify) · MPRIS raw control · X11 window primitives · Wayland compositor IPC (sway/i3/Hyprland dispatch) · Create cron job · Package Query (apt/pacman/emerge/pkg) · Package Install ⚠️ (pkexec consent) · Flatpak/Snap list & launch · brightnessctl · PipeWire/Pulse routing (pactl) · udisks2 mount · Set GTK/KDE theme · xrandr display layout (X11) · Open terminal profile
**FreeBSD note:** `platform-bsd` reuses Linux D-Bus/X11 paths where ports exist; systemd/cron actions map to rc.d & cron; systemd-only actions show the badge with `os = FreeBSD`.

### 8.5 Triggers (Automations)

| Trigger                                                                           | W   | M   | L   | Notes                                                                 |
| --------------------------------------------------------------------------------- | --- | --- | --- | --------------------------------------------------------------------- |
| Schedule (time · RRULE recurrence · cron expr) · Interval                         | ✅  | ✅  | ✅  |                                                                       |
| At Login                                                                          | ✅  | ✅  | ✅  | autostart plugin                                                      |
| Hotkey Pressed                                                                    | ✅  | ✅  | ⚠️  | Wayland: register a compositor keybind → `lightning run` CLI fallback |
| App Launched / Quit                                                               | ✅  | ✅  | ⚠️  |                                                                       |
| File / Folder Changed                                                             | ✅  | ✅  | ✅  | `notify`, debounced                                                   |
| Clipboard Changed                                                                 | ✅  | ✅  | ⚠️  | Wayland: wlr-data-control where available                             |
| Wi-Fi Network Joined / Left (SSID match)                                          | ✅  | ✅  | ⚠️  | L: NetworkManager                                                     |
| Network Up / Down · Battery Level · Charging State · Power Source Changed         | ✅  | ✅  | ✅  |                                                                       |
| USB Device Connected / Removed                                                    | ✅  | ✅  | ✅  | L: udev                                                               |
| Bluetooth Device Connected / Disconnected                                         | ⚠️  | ✅  | ✅  |                                                                       |
| Display Connected / Disconnected                                                  | ✅  | ✅  | ⚠️  |                                                                       |
| System Idle for N Minutes                                                         | ✅  | ✅  | ⚠️  | L: X11 ✅ / Wayland ext-idle-notify                                   |
| Screen Locked / Unlocked                                                          | ✅  | ⚠️  | ⚠️  | L: logind                                                             |
| On Wake from Sleep                                                                | ✅  | ✅  | ✅  | Before Sleep: ⚠️/⚠️/✅ (logind inhibitor)                             |
| Dark Mode Changed                                                                 | ✅  | ✅  | ⚠️  |                                                                       |
| Webhook Received                                                                  | ✅  | ✅  | ✅  | localhost HTTP + token, **off by default** (§14)                      |
| Tray Quick Action · Deep Link (`lightning://run/<id>`) · `.lightning` File Opened | ✅  | ✅  | ✅  |                                                                       |
| Focus Mode Changed                                                                | ❌  | ⚠️  | ❌  |                                                                       |

### 8.6 Recipe — adding a new action (follow exactly)

1. **Spec first**: add `docs/actions/<category>.md` entry — id, params, output type, support matrix, permission class.
2. Implement `Action` in `crates/actions/src/<category>/<name>.rs` — pure, OS-agnostic logic only.
3. OS-specific work → add a trait method in `crates/platform/src/traits.rs`, implement in every `platform-*` crate. Unsupported OSes return `Unsupported { reason }` — **never `todo!()`, never a silent no-op**.
4. Declare `supports` honestly, including runtime-probe requirements.
5. Register in the category's `register()`.
6. `pnpm bindings` — param schema flows to TS automatically.
7. i18n keys `actions.<id>.name/description/params.*` in **en and ko**.
8. Icon + (if a new category) gradient token in `packages/ui`.
9. Tests: unit (logic) · engine integration (fixture shortcut) · per-OS smoke behind `#[cfg(target_os)]`.
10. `pnpm verify` + `pnpm check:targets`.

### 8.7 Recipe — adding a trigger

Mirror §8.6 inside `crates/triggers`, plus: sensible debounce default · persistence round-trip test · entry in the editor's trigger picker · i18n (en+ko) · an automation fixture test proving fire → run → history row.

---

## 9. UI, Design & Animation

### 9.1 Look & feel

- Apple-Shortcuts-**inspired**, never copied: resizable sidebar + content layout · a **grid of gradient shortcut tiles** (icon, name, hotkey chip) · a **vertical block editor** · Automations tab · Gallery of starter templates · Settings · command palette (Ctrl/⌘-K).
- Category gradients are design tokens in `packages/ui/tokens.ts` — e.g. `scripting: blue→indigo`, `files: teal→cyan`, `media: pink→rose`, `web: sky→blue`, `system: slate→zinc`, `input: orange→amber`. **Never hardcode hex in components.**
- Rounded 16–20 px radii, soft elevation, light/dark follows the OS.
- Accessibility: every interactive element has a visible focus ring and is keyboard-operable; the entire editor works keyboard-only (add/move/delete/configure blocks).

### 9.2 Motion system (`packages/ui/motion.ts`)

- **Springs for spatial motion**, not eases: `spring.snappy {stiffness 420, damping 30}` · `spring.smooth {260, 28}` · `spring.gentle {170, 26}`.
- Durations for opacity/color: `fast 120ms` · `base 200ms` · `slow 320ms`.
- **Transform & opacity only.** Never animate width/height/top/left on lists — use `motion` layout/FLIP animations.
- Every animation is interruptible; user input is never gated on an animation finishing.
- `prefers-reduced-motion`: springs collapse to 80 ms fades; no parallax, no confetti. Enforced via the `useMotionPrefs()` hook — **always animate through it**. The WinUI installer honors the equivalent Windows setting.

### 9.3 Signature moments (build with the tokens above)

- **Drag a block**: lifts (scale 1.03 + shadow), siblings part with layout springs, magnetic drop indicator.
- **Insert/delete**: spring in from scale 0.95 / opacity 0; delete collapses via height-FLIP.
- **Run**: blocks light up top-to-bottom following `run://progress`, animated connector line, per-block success tick or error shake (±4 px, 2 cycles) + red pulse.
- **Completion**: subtle confetti on manual runs only — never for automations, never under reduced motion.
- **Gallery → editor**: shared-element tile expansion. Tray menu & palette: fade + scale 0.98→1.

### 9.4 Performance budget

60 fps during drag on a 2019 laptop · editor virtualizes beyond 60 blocks · drive animation from `motion` values, not `useEffect` chains.

---

## 10. Coding Standards

### Rust (`crates/*`, `src-tauri`)

- Edition 2024. `cargo fmt` + `cargo clippy --workspace --all-targets -- -D warnings` must pass.
- Libraries use `thiserror`; only binary edges use `anyhow`. **No `unwrap()`/bare `expect()` outside tests** — bubble with `?`; `expect("reason")` only for true invariants.
- No blocking in async paths (`std::thread::sleep`, sync IO, heavy CPU) — use tokio equivalents or `spawn_blocking` (ffmpeg, hashing, zip).
- `unsafe` only inside `platform-*` FFI modules, always with a `// SAFETY:` comment.
- `#![deny(missing_docs)]` in `core`, `actions`, `ipc-types`.
- Naming: crates `lightning-*` · one action per file · `tracing` with structured fields, never `println!`.

### TypeScript / React

- `strict: true` · no `any` (use `unknown` + narrowing) · non-null `!` needs an adjacent justification comment.
- Function components + hooks. Components PascalCase, files kebab-case, hooks `use-*.ts`.
- Zustand slices in `apps/desktop/src/state/`. Components never call `invoke` directly — only wrappers from `packages/bindings`.
- Tailwind + tokens; no inline hex; `style=` only for dynamic transforms.
- Exhaustive `switch` over generated unions with a `never` assertion.

### C# (`installer/windows`)

- .NET 10 · C# 14 · NRT enabled · file-scoped namespaces · `TreatWarningsAsErrors`.
- MVVM via CommunityToolkit.Mvvm; code-behind is wiring only.
- Absolute async rules: no `async void` outside event handlers · no `.Result`/`.Wait()`/`GetAwaiter().GetResult()` · one shared `HttpClient` · `using`/`await using` for disposables · rethrow with `throw;` · CancellationToken through every IO call.
- Registry/file operations are **idempotent and rollback-safe** — the installer may be re-run over a broken install.

---

## 11. Testing

| Layer                          | Tool                                        | Location                     | Gate                                                                                                         |
| ------------------------------ | ------------------------------------------- | ---------------------------- | ------------------------------------------------------------------------------------------------------------ |
| Rust unit                      | cargo test / nextest                        | each crate                   | PR-blocking                                                                                                  |
| Engine integration             | fixture `.lightning` files replayed         | `crates/core/tests/`         | PR-blocking · core+actions coverage ≥ 80 %                                                                   |
| Schema round-trip & migrations | insta snapshots                             | `crates/core/tests/schema/`  | PR-blocking                                                                                                  |
| Frontend unit                  | Vitest + Testing Library                    | `apps/desktop`, `packages/*` | PR-blocking                                                                                                  |
| E2E                            | WebdriverIO + tauri-driver                  | `tests/e2e/`                 | Linux + Windows blocking · **macOS unsupported by tauri-driver** → manual checklist `docs/qa/macos-smoke.md` |
| Installer                      | xUnit (Deploy.Core) + VM smoke matrix       | `installer/windows`          | release-blocking                                                                                             |
| Per-OS action smoke            | `#[cfg(target_os)]` + `--features os-smoke` | platform crates              | nightly CI                                                                                                   |

Rules: every bug fix lands with a regression test · capability probes get fake-environment tests (e.g. "Wayland without ydotool" ⇒ expect `Unsupported` + reason) · never `#[ignore]` a flaky test without a linked issue.

---

## 12. Git, Branches & PRs

- Trunk-based: `main` protected · branches `feat/<scope>-<desc>`, `fix/…`, `chore/…` · release branches `release/vX.Y`.
- **Conventional Commits** with enforced scopes: `core, actions, triggers, platform-win, platform-mac, platform-linux, platform-bsd, engine, store, scripting, ipc, ui, editor, i18n, installer, updater, packaging, ci, docs, deps`.
  - e.g. `feat(actions): add Extract Text from Image (Windows OCR)` · `fix(platform-linux): probe ydotool socket before declaring input support`
- PRs: small & focused · description includes the platform-impact checklist (tested on / capability changes / schema changes / i18n done) · linked issue · screen recording for UI changes · all CI green · ≥ 1 review (CODEOWNERS auto-assigns platform owners).
- Claude Code specifically: commit in logical units · **never** `git push --force` · never amend others' commits · `--no-verify` is forbidden.

---

## 13. CI/CD & Releases

- **ci.yml** (every PR): fmt/clippy/lint/typecheck → unit tests on 3 OS runners → `pnpm bindings --check` → license audit → Tier-1 target compile checks → E2E (Linux, Windows).
- **nightly.yml**: full build matrix (all tiers) · os-smoke suites · AppImage/.deb artifacts · nightly channel publish.
- **release.yml** (tag `v*`): version-consistency check → build all Tier-1/2 targets → sign (Windows Authenticode · macOS codesign + notarize + staple · minisign for updater manifests · GPG for Linux repo artifacts) → build the Windows installer with embedded payload → generate per-platform/arch `latest.json` → draft GitHub Release with changelog from Conventional Commits → manual approval → publish → open AUR/ebuild/port bump PRs via `packaging/` scripts.
- SemVer · one version everywhere (`pnpm version:sync`) · channels stable/beta/nightly (updater respects the setting).

---

## 14. Security & Privacy Model

**Permission classes** — declared per action, granted **per shortcut**, remembered, revocable in Settings → Privacy:
`fs-read` · `fs-write` (outside app dir) · `shell` (any script/OS bridge) · `input` (keystroke/mouse injection) · `network` · `system-settings` (Wi-Fi, power, registry, defaults…) · `device` (camera/mic/location) · `elevated` (admin/root — always re-prompts, never remembered).

- First run of a shortcut shows a permission sheet listing exactly what it will do. Imported shortcuts (file **or** deep link) additionally get a full step-by-step review before saving — **imports never auto-run**.
- Script-class actions display their full script text in the review UI; automations using `shell`/`input`/`elevated` carry a distinct badge in lists.
- Tauri: strict CSP (no inline scripts) · minimal capabilities per window · `shell` plugin scoped to sidecars + explicit action paths · webview never loads remote origins.
- Shell safety: arguments are always argv arrays — **never string concatenation**. File actions canonicalize paths and confine against symlink/`..` traversal.
- Webhook trigger: localhost-only, token-authenticated, **off by default**.
- Updater: signature + hash verified before apply; downgrades rejected.
- Secrets users store (SMTP, SSH) go to the OS keychain (Credential Manager / Keychain / Secret Service) — never plaintext `settings.json`.
- **Telemetry: none.** Crash reports are opt-in, stored locally, user-inspectable before send.

---

## 15. Internationalization

- Source locale `en`; `ko` must be 100 % (release-blocking); community locales welcome. Files: `packages/i18n/locales/<lang>/{common,actions,editor,settings,installer}.json`.
- Keys are stable IDs (`actions.getClipboard.name`) — never English-as-key. `{{var}}` interpolation, ICU plurals.
- The §8.1 string is canonical: `ko` ships exactly `{{os}}에서 이 기능을 지원하지 않음` (spec form: `{OS}에서 이 기능을 지원하지 않음`).
- The WinUI installer localizes via `.resw` with the same key structure, en+ko minimum.
- Dates/numbers always formatted per user locale via ICU — never hand-formatted.

---

## 16. Licensing (GPL-3.0)

- SPDX header on every source file (Rust/TS/C#):
  `// SPDX-License-Identifier: GPL-3.0-only` + `// Copyright (C) 2026 neramc`
- Dependencies must be GPL-3.0-compatible: MIT, Apache-2.0, BSD-2/3, ISC, MPL-2.0, LGPL, Zlib ✅ — proprietary, SSPL, BUSL, CC-BY-NC, "source-available" ❌. Enforced by `cargo deny` + `pnpm licenses:check`; exceptions require an ADR.
- The ffmpeg sidecar uses **GPL builds** (compatible by design). Third-party notices are generated into the About screen + `THIRD_PARTY_NOTICES.md` at release.
- Contributions accepted under GPL-3.0, inbound = outbound, DCO sign-off. **No CLA.**

---

## 17. What Claude Must NEVER Do

1. Commit secrets, signing keys, or tokens — anywhere, including fixtures.
2. Add a GPL-incompatible or unclearly licensed dependency.
3. Edit generated code (`packages/bindings/src/**`) or leave bindings stale.
4. Weaken CSP, Tauri capabilities, permission gating, or signature verification — even temporarily, even for tests.
5. Ship an action with a dishonest support matrix, a `todo!()`, or a silent no-op. **A wrong ✅ is worse than an ❌.**
6. Break `.lightning` backward compatibility without version bump + migration + tests.
7. Use Apple trademarks/assets, copy Shortcuts UI text, or name anything "Shortcuts".
8. Hardcode user-visible strings, colors, or spring values outside i18n/tokens.
9. Run destructive commands (`git push --force`, `rm -rf` outside the workspace) or code that deletes user data without the keep-data prompt.
10. Introduce telemetry, phone-home checks, or auto-run of imported shortcuts.
11. Block the UI thread (Rust main thread / WinUI dispatcher) with IO; use `.Result`/`async void` in the installer.
12. Invent platform APIs. If unsure an OS API exists, **verify first** — a confident wrong API is the worst failure mode.

---

## 18. Definition of Done & Pointers

A task is **done** when: code follows §10 · `pnpm verify` passes · `pnpm check:targets` passes if platform code changed · bindings regenerated if IPC changed · tests added (regression test for every fix) · i18n en+ko added · capability matrix + `docs/actions/` updated for action changes · docs/ADR updated for behavior or architecture changes · commits follow §12.

Deep dives live in `docs/`: `docs/architecture/` (ADRs) · `docs/actions/` (canonical per-action specs — override §8 summaries) · `docs/qa/` (smoke checklists) · `docs/release.md` · `CONTRIBUTING.md` · `SECURITY.md`.

> If reality in the repo ever contradicts this file, prefer the repo — then update this file in the same PR.
