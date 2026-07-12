# Contributing to Lightning

Thanks for helping! This is a condensed guide — the full operating manual is
[`CLAUDE.md`](CLAUDE.md), which applies to human contributors too.

## Ground rules

1. **License**: contributions are accepted under **GPL-3.0-only** (inbound = outbound),
   with a [DCO](https://developercertificate.org/) sign-off (`git commit -s`). No CLA.
2. Every new source file (Rust / TS / C#) starts with:
   ```
   // SPDX-License-Identifier: GPL-3.0-only
   // Copyright (C) 2026 neramc
   ```
3. Never commit secrets, signing keys, or tokens — not even in tests or fixtures.
4. No Apple assets, ever. Original iconography and strings only.
5. All user-facing strings go through `packages/i18n` — English is the source locale,
   Korean must be complete before release.

## Workflow

- Trunk-based development. Branch from `main`: `feat/<scope>-<desc>`, `fix/…`, `chore/…`.
- **Conventional Commits** with enforced scopes:
  `core, actions, triggers, platform-win, platform-mac, platform-linux, platform-bsd,
engine, store, scripting, ipc, ui, editor, i18n, installer, updater, packaging, ci,
docs, deps`.
- Keep PRs small and focused. Include the platform-impact checklist (tested on /
  capability changes / schema changes / i18n done), link an issue, and attach a screen
  recording for UI changes.
- Never force-push shared branches. `--no-verify` is forbidden.

## Before you push

```sh
pnpm verify          # fmt-check + clippy + lint + typecheck + unit tests
pnpm check:targets   # if you touched crates/platform-*
pnpm bindings        # if you touched any Rust command/event/type
```

Definition of Done: code formatted and linted · tests added (a regression test for every
fix) · i18n en+ko added · capability matrix + `docs/actions/` updated for action changes
· docs/ADR updated for behavior or architecture changes.

## Adding an action or trigger

Follow the recipes in CLAUDE.md §8.6 / §8.7 exactly — spec first in `docs/actions/`,
honest per-platform support declaration, no `todo!()`, no silent no-ops.
