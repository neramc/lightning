// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc
//
// Generates per-target/arch/channel `latest.json` updater manifests from a
// directory of built release artifacts (CLAUDE.md §6.10/§6.11). Signature
// (minisign) is applied by the release workflow after generation — keys never
// touch this script.

import { readdirSync, statSync, writeFileSync, createReadStream } from 'node:fs';
import { createHash } from 'node:crypto';
import { join, basename } from 'node:path';

const dist = process.argv[2];
if (!dist) {
  console.error('usage: node scripts/gen-latest-json.mjs <dist-dir>');
  process.exit(2);
}

const version = process.env.GITHUB_REF_NAME?.replace(/^v/, '') ?? '0.0.0';
const channel = version.includes('-beta') ? 'beta' : version.includes('-nightly') ? 'nightly' : 'stable';

const TARGET_PATTERNS = [
  { target: 'windows-x86_64', pattern: /x86_64-pc-windows/ },
  { target: 'windows-aarch64', pattern: /aarch64-pc-windows/ },
  { target: 'darwin-aarch64', pattern: /aarch64-apple-darwin/ },
  { target: 'darwin-x86_64', pattern: /x86_64-apple-darwin/ },
  { target: 'linux-x86_64', pattern: /x86_64-unknown-linux/ },
  { target: 'linux-aarch64', pattern: /aarch64-unknown-linux/ },
];

function sha256(path) {
  return new Promise((resolvePromise, reject) => {
    const hash = createHash('sha256');
    createReadStream(path)
      .on('data', (d) => hash.update(d))
      .on('end', () => resolvePromise(hash.digest('hex')))
      .on('error', reject);
  });
}

function* walk(dir) {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) yield* walk(full);
    else yield full;
  }
}

const files = [...walk(dist)];
for (const { target, pattern } of TARGET_PATTERNS) {
  const artifact = files.find((f) => pattern.test(f) && /\.(zip|exe|dmg|AppImage|deb)$/.test(f));
  if (!artifact) continue;
  const manifest = {
    version,
    channel,
    pub_date: new Date().toISOString(),
    url: `https://github.com/neramc/lightning/releases/download/v${version}/${basename(artifact)}`,
    sha256: await sha256(artifact),
    signature: null, // filled by the minisign step in release.yml
    notes: `Lightning ${version}`,
  };
  const out = join(dist, `latest-${target}-${channel}.json`);
  writeFileSync(out, JSON.stringify(manifest, null, 2));
  console.log(`✓ ${out}`);
}
