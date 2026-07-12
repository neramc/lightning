#!/usr/bin/env bash
# SPDX-License-Identifier: GPL-3.0-only
# Copyright (C) 2026 neramc
#
# Sign + notarize + staple the .dmg (release.yml). Credentials come ONLY
# from CI secrets (§2): APPLE_ID, APPLE_TEAM_ID, APPLE_APP_PASSWORD,
# APPLE_SIGNING_IDENTITY.
set -euo pipefail

DMG="${1:?usage: notarize.sh <path-to-dmg>}"

codesign --force --options runtime \
  --entitlements "$(dirname "$0")/entitlements.plist" \
  --sign "$APPLE_SIGNING_IDENTITY" \
  "$DMG"

xcrun notarytool submit "$DMG" \
  --apple-id "$APPLE_ID" \
  --team-id "$APPLE_TEAM_ID" \
  --password "$APPLE_APP_PASSWORD" \
  --wait

xcrun stapler staple "$DMG"
echo "✓ notarized and stapled: $DMG"
