# ADR-0002: `.lightning` files are versioned pretty JSON; the DB is a cache

- Status: accepted
- Date: 2026-07-12

## Context

Shortcuts must be shareable, diffable, and durable across app versions
(§2.6: old files always open). We also need fast search/list and run
history.

## Decision

- One pretty-printed JSON file per shortcut with a top-level
  `schemaVersion`. Writes are atomic (write `.tmp`, fsync, rename).
- Breaking schema changes bump `CURRENT_SCHEMA_VERSION` and register a step
  migration in `crates/core/src/migrate/`; documents newer than the build
  are rejected, never mangled.
- SQLite (`index.db`) holds names/icons/hotkeys/automation flags and the
  run-history ring buffer (default 1000). `SqliteIndex::reindex()` rebuilds
  everything from files and skips corrupt documents with a warning.

## Consequences

- Users can sync/back up the shortcuts directory with any file tool.
- Deleting `index.db` is always safe.
- Schema changes are visible in review (snapshot test pins the shape).
