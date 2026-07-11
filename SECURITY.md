# Security Policy

## Reporting a vulnerability

Please report vulnerabilities privately via
[GitHub Security Advisories](https://github.com/neramc/lightning/security/advisories/new).
Do **not** open a public issue for security reports. You should receive a response
within 7 days.

## Scope highlights

Lightning executes user-authored automation flows, so the permission model is the
security boundary:

- Permission classes (`fs-read`, `fs-write`, `shell`, `input`, `network`,
  `system-settings`, `device`, `elevated`) are declared per action and granted per
  shortcut. `elevated` always re-prompts and is never remembered.
- Imported shortcuts (file or `lightning://` deep link) always open a full step-by-step
  permission review before saving — **imports never auto-run**.
- Script-class actions display their full script text in the review UI.
- The webhook trigger is localhost-only, token-authenticated, and off by default.
- Shell arguments are always argv arrays — never string concatenation. File actions
  canonicalize paths and confine against symlink/`..` traversal.
- Updates verify a minisign (ed25519) signature and SHA-256 hash before applying;
  downgrades are rejected.
- Secrets users store (SMTP, SSH) go to the OS keychain — never plaintext files.
- Telemetry: none. Crash reports are opt-in, stored locally, user-inspectable.

Reports that weaken any of the above (permission bypass, CSP weakening, unsigned update
acceptance, sandbox escape from the QuickJS runtime) are highest priority.

## Supported versions

Only the latest stable release receives security fixes.
