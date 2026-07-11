// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! `.lightning` file IO — pretty JSON, atomic write-then-rename, migration
//! on load. Files are the source of truth (§6.3).

use std::path::{Path, PathBuf};

use lightning_core::Shortcut;

use crate::StoreError;

/// File name for a shortcut id.
fn file_name(shortcut: &Shortcut) -> String {
    format!("{}.lightning", shortcut.id)
}

/// Save atomically: write `<id>.lightning.tmp` in the same directory, fsync,
/// then rename over the target so a crash never leaves a torn file.
pub fn save_shortcut(dir: &Path, shortcut: &Shortcut) -> Result<PathBuf, StoreError> {
    std::fs::create_dir_all(dir)?;
    let target = dir.join(file_name(shortcut));
    let tmp = dir.join(format!("{}.tmp", file_name(shortcut)));
    let json = shortcut.to_pretty_json()?;
    {
        use std::io::Write;
        let mut file = std::fs::File::create(&tmp)?;
        file.write_all(json.as_bytes())?;
        file.write_all(b"\n")?;
        file.sync_all()?;
    }
    std::fs::rename(&tmp, &target)?;
    tracing::debug!(path = %target.display(), "shortcut saved");
    Ok(target)
}

/// Load one `.lightning` file, migrating older schema versions.
pub fn load_shortcut(path: &Path) -> Result<Shortcut, StoreError> {
    let source = std::fs::read_to_string(path)?;
    Ok(Shortcut::from_json_str(&source)?)
}

/// All `.lightning` files in a directory (non-recursive), sorted by name.
pub fn list_shortcut_files(dir: &Path) -> Result<Vec<PathBuf>, StoreError> {
    let mut paths = Vec::new();
    if !dir.exists() {
        return Ok(paths);
    }
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.extension().is_some_and(|ext| ext == "lightning") {
            paths.push(path);
        }
    }
    paths.sort();
    Ok(paths)
}

/// Remove a shortcut's file. (User-facing "Delete Files" actions trash by
/// default — this is the store-internal removal that follows an explicit,
/// confirmed delete in the UI.)
pub fn delete_shortcut_file(dir: &Path, shortcut_id: uuid::Uuid) -> Result<(), StoreError> {
    let path = dir.join(format!("{shortcut_id}.lightning"));
    std::fs::remove_file(path)?;
    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use lightning_core::{Content, Step};

    #[test]
    fn save_load_round_trip() {
        let dir = tempfile::tempdir().unwrap();
        let mut shortcut = Shortcut::new("Round Trip");
        shortcut
            .steps
            .push(Step::new("text.text").with_param("text", Content::Text("hi".into())));
        let path = save_shortcut(dir.path(), &shortcut).unwrap();
        assert!(path.exists());
        let loaded = load_shortcut(&path).unwrap();
        assert_eq!(loaded, shortcut);
    }

    #[test]
    fn save_is_atomic_no_tmp_left_behind() {
        let dir = tempfile::tempdir().unwrap();
        let shortcut = Shortcut::new("Atomic");
        save_shortcut(dir.path(), &shortcut).unwrap();
        let leftovers: Vec<_> = std::fs::read_dir(dir.path())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().is_some_and(|x| x == "tmp"))
            .collect();
        assert!(leftovers.is_empty());
    }

    #[test]
    fn list_filters_to_lightning_files() {
        let dir = tempfile::tempdir().unwrap();
        save_shortcut(dir.path(), &Shortcut::new("A")).unwrap();
        std::fs::write(dir.path().join("noise.txt"), "x").unwrap();
        assert_eq!(list_shortcut_files(dir.path()).unwrap().len(), 1);
    }
}
