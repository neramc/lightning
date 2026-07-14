#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Copyright (C) 2026 neramc
#
# Builds the AppImage from a completed `pnpm build` (Tauri already produces
# an AppImage bundle; this script normalizes naming + updater metadata).
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
BUNDLE_DIR="$REPO_ROOT/target/release/bundle/appimage"
VERSION="$(node -p "require('$REPO_ROOT/package.json').version")"
ARCH="${ARCH:-x86_64}"

if ! compgen -G "$BUNDLE_DIR/*.AppImage" > /dev/null; then
  echo "no AppImage found — run 'pnpm build' first" >&2
  exit 1
fi

OUT="$REPO_ROOT/dist"
mkdir -p "$OUT"
for image in "$BUNDLE_DIR"/*.AppImage; do
  target="$OUT/Lightning-$VERSION-$ARCH.AppImage"
  cp "$image" "$target"
  chmod +x "$target"
  echo "✓ $target"
done
