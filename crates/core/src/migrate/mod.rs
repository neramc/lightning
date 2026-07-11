// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! `.lightning` schema migrations (CLAUDE.md §2.6).
//!
//! The file format is versioned with a top-level `schemaVersion`. Any
//! breaking change bumps [`CURRENT_SCHEMA_VERSION`] and registers a step
//! migration here, with round-trip tests in `tests/schema_roundtrip.rs`.
//! **Old files must always open.**

use serde_json::Value;

use crate::error::MigrateError;

/// The schema version this build writes.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

/// One step migration: transforms a document from `from` to `from + 1`.
struct Migration {
    from: u32,
    apply: fn(&mut Value) -> Result<(), MigrateError>,
}

/// Registered step migrations, ordered by `from`.
///
/// Example of what a future entry looks like:
/// ```ignore
/// Migration { from: 1, apply: v1_to_v2 } // v2 renamed steps[].params keys
/// ```
const MIGRATIONS: &[Migration] = &[];

/// Migrate `doc` in place to [`CURRENT_SCHEMA_VERSION`].
///
/// Returns the version the document had before migration.
pub fn migrate_to_current(doc: &mut Value) -> Result<u32, MigrateError> {
    let found = doc
        .get("schemaVersion")
        .and_then(Value::as_u64)
        .and_then(|v| u32::try_from(v).ok())
        .ok_or(MigrateError::InvalidFile)?;

    if found == 0 || found > CURRENT_SCHEMA_VERSION {
        return Err(MigrateError::UnsupportedVersion {
            found,
            current: CURRENT_SCHEMA_VERSION,
        });
    }

    let mut version = found;
    while version < CURRENT_SCHEMA_VERSION {
        let migration = MIGRATIONS
            .iter()
            .find(|m| m.from == version)
            .ok_or(MigrateError::MissingMigration { from: version })?;
        (migration.apply)(doc)?;
        version += 1;
        doc["schemaVersion"] = Value::from(version);
    }
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn current_version_documents_pass_through_unchanged() {
        let mut doc = json!({ "schemaVersion": CURRENT_SCHEMA_VERSION, "name": "x" });
        let before = doc.clone();
        let found = migrate_to_current(&mut doc).expect("migrates");
        assert_eq!(found, CURRENT_SCHEMA_VERSION);
        assert_eq!(doc, before);
    }

    #[test]
    fn newer_documents_are_rejected_not_mangled() {
        let mut doc = json!({ "schemaVersion": CURRENT_SCHEMA_VERSION + 1 });
        assert!(matches!(
            migrate_to_current(&mut doc),
            Err(MigrateError::UnsupportedVersion { .. })
        ));
    }

    #[test]
    fn missing_schema_version_is_invalid() {
        let mut doc = json!({ "name": "x" });
        assert!(matches!(
            migrate_to_current(&mut doc),
            Err(MigrateError::InvalidFile)
        ));
    }

    #[test]
    fn zero_is_rejected() {
        let mut doc = json!({ "schemaVersion": 0 });
        assert!(matches!(
            migrate_to_current(&mut doc),
            Err(MigrateError::UnsupportedVersion { .. })
        ));
    }
}
