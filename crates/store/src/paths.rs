// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Per-OS data paths (CLAUDE.md §6.9). Dev builds use an isolated
//! `Lightning-dev` profile so development never touches a real user's
//! shortcuts (§5).

use std::path::PathBuf;

use crate::StoreError;

#[cfg(any(target_os = "windows", target_os = "macos"))]
fn app_dir_name() -> &'static str {
    if cfg!(debug_assertions) {
        "Lightning-dev"
    } else {
        "Lightning"
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos")))]
fn app_dir_name() -> &'static str {
    if cfg!(debug_assertions) {
        "lightning-dev"
    } else {
        "lightning"
    }
}

/// The app data root:
/// `%APPDATA%\Lightning` · `~/Library/Application Support/Lightning` ·
/// `$XDG_DATA_HOME/lightning`.
pub fn data_dir() -> Result<PathBuf, StoreError> {
    let base = dirs::data_dir().ok_or(StoreError::NoDataDir)?;
    Ok(base.join(app_dir_name()))
}

/// Where `.lightning` files live.
pub fn shortcuts_dir() -> Result<PathBuf, StoreError> {
    Ok(data_dir()?.join("shortcuts"))
}

/// The SQLite index file.
pub fn index_path() -> Result<PathBuf, StoreError> {
    Ok(data_dir()?.join("index.db"))
}

/// Rotated log directory (`$XDG_STATE_HOME/lightning/logs` on Linux/BSD,
/// under the data root elsewhere).
pub fn logs_dir() -> Result<PathBuf, StoreError> {
    #[cfg(any(target_os = "windows", target_os = "macos"))]
    {
        Ok(data_dir()?.join("logs"))
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        let base = dirs::state_dir().ok_or(StoreError::NoDataDir)?;
        Ok(base.join(app_dir_name()).join("logs"))
    }
}

/// `settings.json` — schema-versioned, written atomically.
pub fn settings_path() -> Result<PathBuf, StoreError> {
    Ok(data_dir()?.join("settings.json"))
}
