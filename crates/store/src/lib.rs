// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! # lightning-store
//!
//! Persistence (CLAUDE.md §6.3): one pretty-printed `.lightning` file per
//! shortcut under the per-OS data dir, plus a SQLite index for names, tags,
//! hotkeys, and run history.
//!
//! **Files are the source of truth; the DB is a rebuildable cache** — a full
//! [`SqliteIndex::reindex`] from files must always succeed, and it is tested.

mod files;
mod index;
mod paths;

pub use files::{delete_shortcut_file, list_shortcut_files, load_shortcut, save_shortcut};
pub use index::{RunRecord, RunStatus, ShortcutMeta, SqliteIndex};
pub use paths::{data_dir, index_path, logs_dir, settings_path, shortcuts_dir};

/// Errors from the store.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The platform data directory could not be determined.
    #[error("no data directory available on this system")]
    NoDataDir,
    /// A `.lightning` document failed to parse or migrate.
    #[error(transparent)]
    Migrate(#[from] lightning_core::MigrateError),
    /// Serialization failed.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// SQLite failure.
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    /// Filesystem failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}
