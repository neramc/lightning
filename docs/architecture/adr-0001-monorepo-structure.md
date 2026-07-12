# ADR-0001: Single monorepo, layered crates, thin Tauri shell

- Status: accepted
- Date: 2026-07-12

## Context

Lightning spans a Tauri 2 desktop app, a large Rust engine, a C#/WinUI 3
installer suite, distro packaging, and docs. Splitting these across repos
would force cross-repo version pins for the IPC contract (§6.8) and the
`.lightning` schema (§6.3), both of which must change atomically.

## Decision

One repository (`github.com/neramc/lightning`) with pnpm workspaces +
Turborepo for JS, a single Cargo workspace for Rust, and one .NET solution.
All engine logic lives in `crates/*`; `apps/desktop/src-tauri` stays a thin
shell (command glue, tray, windows). The per-OS platform crates implement
the `lightning-platform` traits, and the only place a concrete platform is
selected is the shell's `host_platform()`.

## Consequences

- `pnpm bindings` can regenerate the TS contract from the same commit that
  changed the Rust types; CI fails on staleness.
- `cargo check`/`test` default-members exclude the Tauri shell so pure
  crates build on machines without webkit2gtk; CI runs `--workspace`.
- Cross-cutting changes (schema + migration + UI) land as one reviewable PR.
