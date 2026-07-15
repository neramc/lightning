// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc
//
// Generates the detailed release body (CLAUDE.md §13): a per-OS download
// guide, a changelog grouped from Conventional Commits since the previous
// tag, and SHA-256 checksums of the artifacts in the dist directory.
// Usage: node scripts/gen-release-notes.mjs [dist-dir] > release-notes.md
// Requires full git history (fetch-depth: 0) for the changelog section.

import { execFileSync } from 'node:child_process';
import { readdirSync, statSync, createReadStream } from 'node:fs';
import { createHash } from 'node:crypto';
import { join, basename } from 'node:path';

const version = (process.env.RELEASE_VERSION ?? process.env.GITHUB_REF_NAME ?? '0.0.0')
  .replace(/^release\//, '')
  .replace(/^v/, '');

function git(...args) {
  return execFileSync('git', args, { encoding: 'utf8' }).trim();
}

// ---------------------------------------------------------------- changelog
let range = 'HEAD';
let previousTag = null;
try {
  previousTag = git('describe', '--tags', '--abbrev=0', 'HEAD^');
  range = `${previousTag}..HEAD`;
} catch {
  // First release — walk the whole history.
}

const SEP = '\x1f';
const RS = '\x1e';
const log = git('log', '--no-merges', `--pretty=format:%h%x1f%s%x1e`, range);

/** Conventional Commit groups, in display order. */
const GROUPS = [
  { key: 'feat', title: '✨ Features' },
  { key: 'fix', title: '🐛 Bug Fixes' },
  { key: 'perf', title: '⚡ Performance' },
  { key: 'refactor', title: '♻️ Refactoring' },
  { key: 'docs', title: '📚 Documentation' },
  { key: 'test', title: '🧪 Tests' },
  { key: 'build', title: '🏗️ Build & CI' },
  { key: 'ci', title: '🏗️ Build & CI' },
  { key: 'chore', title: '🧹 Chores & Packaging' },
  { key: 'other', title: '🗂️ Other Changes' },
];

const grouped = new Map();
for (const entry of log.split(RS)) {
  const line = entry.trim();
  if (!line) continue;
  const [sha, subject] = line.split(SEP);
  const m = subject?.match(/^(\w+)(?:\(([^)]+)\))?(!)?:\s*(.+)$/);
  const type = m && GROUPS.some((g) => g.key === m[1]) ? m[1] : 'other';
  const scope = m?.[2];
  const breaking = Boolean(m?.[3]);
  const text = m ? m[4] : subject;
  const title = GROUPS.find((g) => g.key === type).title;
  if (!grouped.has(title)) grouped.set(title, []);
  grouped
    .get(title)
    .push(
      `- ${breaking ? '**BREAKING** ' : ''}${scope ? `**${scope}**: ` : ''}${text} (${sha})`,
    );
}

// ------------------------------------------------------------- dist assets
const dist = process.argv[2];

function* walk(dir) {
  for (const entry of readdirSync(dir)) {
    const full = join(dir, entry);
    if (statSync(full).isDirectory()) yield* walk(full);
    else yield full;
  }
}

function sha256(path) {
  return new Promise((resolvePromise, reject) => {
    const hash = createHash('sha256');
    createReadStream(path)
      .on('data', (d) => hash.update(d))
      .on('end', () => resolvePromise(hash.digest('hex')))
      .on('error', reject);
  });
}

const ARTIFACT_RE = /\.(exe|msi|zip|dmg|deb|rpm|AppImage)$/;
const artifacts = dist
  ? [...walk(dist)].filter((f) => ARTIFACT_RE.test(f)).sort((a, b) => a.localeCompare(b))
  : [];

/** Ordered per-OS download guide; rows appear only when the asset exists. */
const DOWNLOAD_ROWS = [
  ['Windows 10/11 (x64)', /x64-setup\.exe$/, 'Installer (recommended)'],
  ['Windows 10/11 (x64)', /x64_en-US\.msi$/, 'MSI package'],
  ['Windows 10/11 (x64)', /windows-msvc-portable\.zip$/, 'Portable — no install needed'],
  ['macOS 11+ (Apple Silicon)', /aarch64\.dmg$/, 'Disk image'],
  ['macOS 11+ (Intel)', /x64\.dmg$/, 'Disk image'],
  ['Ubuntu / Debian (x64)', /amd64\.deb$/, 'apt package'],
  ['Ubuntu / Debian (ARM64)', /arm64\.deb$/, 'apt package'],
  ['Fedora / openSUSE (x64)', /x86_64\.rpm$/, 'rpm package'],
  ['Fedora / openSUSE (ARM64)', /aarch64\.rpm$/, 'rpm package'],
  ['Any glibc Linux (x64)', /amd64\.AppImage$/, 'AppImage — chmod +x and run'],
  ['Any glibc Linux (ARM64)', /aarch64\.AppImage$/, 'AppImage — chmod +x and run'],
];

// ------------------------------------------------------------------ output
const out = [];
out.push(`## Lightning ${version}`);
out.push('');
out.push(
  'Free, open-source shortcuts and automations for your desktop — ' +
    'drag-and-drop action blocks, powerful OS-level triggers, and a ' +
    'flow editor inspired by the best of mobile automation. ' +
    'Local-first: no account, no telemetry.',
);

if (artifacts.length > 0) {
  out.push('');
  out.push('### 📦 Downloads');
  out.push('');
  out.push('| Platform | File | Notes |');
  out.push('| --- | --- | --- |');
  for (const [platform, pattern, note] of DOWNLOAD_ROWS) {
    const hit = artifacts.find((f) => pattern.test(basename(f)));
    if (hit) out.push(`| ${platform} | \`${basename(hit)}\` | ${note} |`);
  }
  out.push('');
  out.push(
    '> ⚠️ Builds are not yet code-signed: on Windows, choose “More info → ' +
      'Run anyway” at the SmartScreen prompt; on macOS, right-click the app ' +
      'and choose “Open” the first time.',
  );
  out.push('');
  out.push(
    '`latest-*-stable.json` files are updater manifests consumed by the ' +
      'in-app updater — no need to download them manually.',
  );
}

out.push('');
out.push('### 📝 Changelog');
if (previousTag) {
  out.push('');
  out.push(`Changes since ${previousTag}:`);
}
for (const { title } of GROUPS.filter(
  (g, i, all) => all.findIndex((x) => x.title === g.title) === i,
)) {
  const items = grouped.get(title);
  if (!items || items.length === 0) continue;
  out.push('');
  out.push(`#### ${title}`);
  out.push('');
  out.push(...items);
}

if (artifacts.length > 0) {
  out.push('');
  out.push('<details>');
  out.push('<summary>🔒 SHA-256 checksums</summary>');
  out.push('');
  out.push('| File | SHA-256 |');
  out.push('| --- | --- |');
  for (const file of artifacts) {
    out.push(`| \`${basename(file)}\` | \`${await sha256(file)}\` |`);
  }
  out.push('');
  out.push('</details>');
}

out.push('');
console.log(out.join('\n'));
