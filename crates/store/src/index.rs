// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The SQLite index: names, icons, hotkeys, automation flags, and run
//! history (ring buffer, default 1000 entries). A rebuildable cache over the
//! `.lightning` files — never the truth (§6.3).

use std::path::Path;

use chrono::{DateTime, Utc};
use lightning_core::Shortcut;
use rusqlite::Connection;
use uuid::Uuid;

use crate::{StoreError, files};

/// Default run-history ring buffer size (§6.7).
pub const DEFAULT_HISTORY_LIMIT: u64 = 1_000;

/// Indexed metadata for one shortcut (what lists and search need).
#[derive(Debug, Clone, PartialEq)]
pub struct ShortcutMeta {
    /// Shortcut id.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Tile glyph.
    pub icon_glyph: String,
    /// Gradient token.
    pub gradient: String,
    /// Assigned hotkey, if any.
    pub hotkey: Option<String>,
    /// Whether it has a trigger block.
    pub is_automation: bool,
    /// Source file path.
    pub file_path: String,
}

/// Outcome of one run, persisted into history.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunStatus {
    /// Completed.
    Success,
    /// Failed with an error.
    Error,
    /// Cancelled by the user or a timeout.
    Cancelled,
}

impl RunStatus {
    fn as_str(self) -> &'static str {
        match self {
            RunStatus::Success => "success",
            RunStatus::Error => "error",
            RunStatus::Cancelled => "cancelled",
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "error" => RunStatus::Error,
            "cancelled" => RunStatus::Cancelled,
            _ => RunStatus::Success,
        }
    }
}

/// One run-history row.
#[derive(Debug, Clone, PartialEq)]
pub struct RunRecord {
    /// The shortcut that ran.
    pub shortcut_id: Uuid,
    /// When the run started.
    pub started_at: DateTime<Utc>,
    /// Duration in milliseconds.
    pub duration_ms: i64,
    /// Outcome.
    pub status: RunStatus,
    /// Error message when `status == Error`.
    pub error: Option<String>,
}

/// The index connection.
pub struct SqliteIndex {
    conn: Connection,
    history_limit: u64,
}

impl SqliteIndex {
    /// Open (or create) the index at `path`.
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        Self::from_connection(Connection::open(path)?)
    }

    /// An in-memory index (tests).
    pub fn in_memory() -> Result<Self, StoreError> {
        Self::from_connection(Connection::open_in_memory()?)
    }

    fn from_connection(conn: Connection) -> Result<Self, StoreError> {
        conn.execute_batch(
            "
            PRAGMA journal_mode = WAL;
            CREATE TABLE IF NOT EXISTS shortcuts (
                id            TEXT PRIMARY KEY,
                name          TEXT NOT NULL,
                icon_glyph    TEXT NOT NULL,
                gradient      TEXT NOT NULL,
                hotkey        TEXT,
                is_automation INTEGER NOT NULL DEFAULT 0,
                file_path     TEXT NOT NULL,
                indexed_at    TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_shortcuts_name ON shortcuts(name);
            CREATE TABLE IF NOT EXISTS runs (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                shortcut_id TEXT NOT NULL,
                started_at  TEXT NOT NULL,
                duration_ms INTEGER NOT NULL,
                status      TEXT NOT NULL,
                error       TEXT
            );
            CREATE INDEX IF NOT EXISTS idx_runs_shortcut ON runs(shortcut_id, started_at DESC);
            ",
        )?;
        Ok(Self {
            conn,
            history_limit: DEFAULT_HISTORY_LIMIT,
        })
    }

    /// Override the history ring-buffer size.
    #[must_use]
    pub fn with_history_limit(mut self, limit: u64) -> Self {
        self.history_limit = limit.max(1);
        self
    }

    /// Insert or update one shortcut's metadata.
    pub fn upsert(&self, shortcut: &Shortcut, file_path: &Path) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO shortcuts (id, name, icon_glyph, gradient, hotkey, is_automation, file_path, indexed_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                icon_glyph = excluded.icon_glyph,
                gradient = excluded.gradient,
                hotkey = excluded.hotkey,
                is_automation = excluded.is_automation,
                file_path = excluded.file_path,
                indexed_at = excluded.indexed_at",
            rusqlite::params![
                shortcut.id.to_string(),
                shortcut.name,
                shortcut.icon.glyph,
                shortcut.icon.gradient,
                shortcut.hotkey,
                shortcut.is_automation(),
                file_path.display().to_string(),
                Utc::now().to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    /// Remove one shortcut (and its run history).
    pub fn remove(&self, shortcut_id: Uuid) -> Result<(), StoreError> {
        self.conn.execute(
            "DELETE FROM shortcuts WHERE id = ?1",
            [shortcut_id.to_string()],
        )?;
        self.conn.execute(
            "DELETE FROM runs WHERE shortcut_id = ?1",
            [shortcut_id.to_string()],
        )?;
        Ok(())
    }

    /// All indexed shortcuts, ordered by name.
    pub fn list(&self) -> Result<Vec<ShortcutMeta>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, icon_glyph, gradient, hotkey, is_automation, file_path
             FROM shortcuts ORDER BY name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok(ShortcutMeta {
                id: row
                    .get::<_, String>(0)?
                    .parse()
                    .unwrap_or_else(|_| Uuid::nil()),
                name: row.get(1)?,
                icon_glyph: row.get(2)?,
                gradient: row.get(3)?,
                hotkey: row.get(4)?,
                is_automation: row.get(5)?,
                file_path: row.get(6)?,
            })
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Case-insensitive name search.
    pub fn search(&self, query: &str) -> Result<Vec<ShortcutMeta>, StoreError> {
        let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
        let mut stmt = self.conn.prepare(
            "SELECT id, name, icon_glyph, gradient, hotkey, is_automation, file_path
             FROM shortcuts WHERE name LIKE ?1 ESCAPE '\\' ORDER BY name COLLATE NOCASE",
        )?;
        let rows = stmt.query_map([pattern], |row| {
            Ok(ShortcutMeta {
                id: row
                    .get::<_, String>(0)?
                    .parse()
                    .unwrap_or_else(|_| Uuid::nil()),
                name: row.get(1)?,
                icon_glyph: row.get(2)?,
                gradient: row.get(3)?,
                hotkey: row.get(4)?,
                is_automation: row.get(5)?,
                file_path: row.get(6)?,
            })
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Record a run and trim the ring buffer.
    pub fn record_run(&self, record: &RunRecord) -> Result<(), StoreError> {
        self.conn.execute(
            "INSERT INTO runs (shortcut_id, started_at, duration_ms, status, error)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                record.shortcut_id.to_string(),
                record.started_at.to_rfc3339(),
                record.duration_ms,
                record.status.as_str(),
                record.error,
            ],
        )?;
        self.conn.execute(
            "DELETE FROM runs WHERE id NOT IN (SELECT id FROM runs ORDER BY id DESC LIMIT ?1)",
            [self.history_limit],
        )?;
        Ok(())
    }

    /// Most recent runs for one shortcut.
    pub fn recent_runs(&self, shortcut_id: Uuid, limit: u32) -> Result<Vec<RunRecord>, StoreError> {
        let mut stmt = self.conn.prepare(
            "SELECT shortcut_id, started_at, duration_ms, status, error
             FROM runs WHERE shortcut_id = ?1 ORDER BY id DESC LIMIT ?2",
        )?;
        let rows = stmt.query_map(rusqlite::params![shortcut_id.to_string(), limit], |row| {
            Ok(RunRecord {
                shortcut_id: row
                    .get::<_, String>(0)?
                    .parse()
                    .unwrap_or_else(|_| Uuid::nil()),
                started_at: row
                    .get::<_, String>(1)?
                    .parse::<DateTime<Utc>>()
                    .unwrap_or_else(|_| Utc::now()),
                duration_ms: row.get(2)?,
                status: RunStatus::parse(&row.get::<_, String>(3)?),
                error: row.get(4)?,
            })
        })?;
        Ok(rows.filter_map(Result::ok).collect())
    }

    /// Rebuild the whole index from the files in `dir`. Files that fail to
    /// parse are skipped with a warning — one corrupt file must never take
    /// down the index.
    pub fn reindex(&self, dir: &Path) -> Result<usize, StoreError> {
        self.conn.execute("DELETE FROM shortcuts", [])?;
        let mut indexed = 0;
        for path in files::list_shortcut_files(dir)? {
            match files::load_shortcut(&path) {
                Ok(shortcut) => {
                    self.upsert(&shortcut, &path)?;
                    indexed += 1;
                }
                Err(err) => {
                    tracing::warn!(path = %path.display(), %err, "skipping unreadable shortcut");
                }
            }
        }
        Ok(indexed)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::files::save_shortcut;

    fn sample(name: &str) -> Shortcut {
        Shortcut::new(name)
    }

    #[test]
    fn reindex_rebuilds_from_files() {
        let dir = tempfile::tempdir().unwrap();
        for name in ["Alpha", "beta", "Gamma"] {
            save_shortcut(dir.path(), &sample(name)).unwrap();
        }
        // A corrupt file must be skipped, not fatal.
        std::fs::write(dir.path().join("broken.lightning"), "{ not json").unwrap();

        let index = SqliteIndex::in_memory().unwrap();
        let indexed = index.reindex(dir.path()).unwrap();
        assert_eq!(indexed, 3);
        let listed = index.list().unwrap();
        assert_eq!(listed.len(), 3);
        // Case-insensitive name ordering.
        assert_eq!(listed[0].name, "Alpha");
        assert_eq!(listed[1].name, "beta");
    }

    #[test]
    fn search_matches_case_insensitively() {
        let index = SqliteIndex::in_memory().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let path = save_shortcut(dir.path(), &sample("Morning Routine")).unwrap();
        index.upsert(&sample("Morning Routine"), &path).unwrap();
        assert_eq!(index.search("morning").unwrap().len(), 1);
        assert_eq!(index.search("zzz").unwrap().len(), 0);
    }

    #[test]
    fn run_history_ring_buffer_trims() {
        let index = SqliteIndex::in_memory().unwrap().with_history_limit(5);
        let id = Uuid::new_v4();
        for i in 0..10 {
            index
                .record_run(&RunRecord {
                    shortcut_id: id,
                    started_at: Utc::now(),
                    duration_ms: i,
                    status: RunStatus::Success,
                    error: None,
                })
                .unwrap();
        }
        let runs = index.recent_runs(id, 100).unwrap();
        assert_eq!(runs.len(), 5, "ring buffer must trim to the limit");
        assert_eq!(runs[0].duration_ms, 9, "newest first");
    }

    #[test]
    fn upsert_updates_in_place() {
        let index = SqliteIndex::in_memory().unwrap();
        let mut shortcut = sample("Old Name");
        let path = std::path::PathBuf::from("/tmp/x.lightning");
        index.upsert(&shortcut, &path).unwrap();
        shortcut.name = "New Name".into();
        index.upsert(&shortcut, &path).unwrap();
        let listed = index.list().unwrap();
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0].name, "New Name");
    }
}
