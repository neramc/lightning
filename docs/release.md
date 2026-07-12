# Release process

1. `pnpm version:sync <x.y.z>` — one version everywhere (package.json,
   Cargo workspace, tauri.conf.json, installer Directory.Build.props).
2. Land the version bump PR; ensure CI is green.
3. Tag `v<x.y.z>` — release.yml takes over (§13):
   version-consistency check → Tier-1/2 build matrix → signing (Windows
   Authenticode · macOS codesign + notarize + staple · minisign for updater
   manifests · GPG for repo artifacts) → Windows installer with embedded
   payload → per-target `latest.json` → draft GitHub Release with the
   Conventional-Commits changelog.
4. Run `docs/qa/macos-smoke.md` on the release candidate .dmg.
5. Approve the `release` environment gate → publish.
6. Packaging bump PRs (AUR / ebuild / FreeBSD port) open automatically from
   `packaging/`; review and merge.

Channels: stable · beta (`x.y.z-beta.n`) · nightly (nightly.yml). Updater
manifests are per target/arch/channel; downgrades are rejected by version
monotonicity. deb/AUR/ebuild/port installs are updated by the distro — the
in-app check only notifies (§6.11).
