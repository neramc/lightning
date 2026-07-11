// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Error types shared by the engine, actions, and stores.

use crate::content::ContentKind;
use crate::permission::PermissionClass;

/// A coercion between content types that the explicit table does not allow,
/// or whose parse failed (CLAUDE.md §6.2 — never add an implicit lossy
/// coercion).
#[derive(Debug, thiserror::Error)]
pub enum CoerceError {
    /// The (from → to) pair is not in the coercion table.
    #[error("cannot coerce {from} to {to}")]
    Unsupported {
        /// Source kind.
        from: ContentKind,
        /// Requested target kind.
        to: ContentKind,
    },
    /// The pair is allowed but this particular value failed to parse.
    #[error("cannot parse {value:?} as {to}")]
    Parse {
        /// The offending value, abbreviated.
        value: String,
        /// Requested target kind.
        to: ContentKind,
    },
}

/// Errors an action (or the engine around it) can produce.
///
/// `Unsupported` carries the `{{os}}` label (`"Linux (Wayland)"` style) for
/// the localized `action.unsupportedOnOs` message; `MissingTool` /
/// `NeedsOsPermission` map to `action.needsTool` / `action.needsPermission`
/// with a "Fix it" affordance — never blame the OS for a solvable setup
/// issue (§8.1).
#[derive(Debug, thiserror::Error)]
pub enum ActionError {
    /// The action cannot work on this OS/environment.
    #[error("not supported on {os}")]
    Unsupported {
        /// OS display label including environment refinement.
        os: String,
        /// Technical reason (developer/log facing).
        reason: String,
    },
    /// A Lightning permission class has not been granted to this shortcut.
    #[error("permission not granted: {class}")]
    PermissionNotGranted {
        /// The missing permission class.
        class: PermissionClass,
    },
    /// An OS-level permission is missing (solvable — "Fix it" opens settings).
    #[error("missing OS permission: {permission}")]
    NeedsOsPermission {
        /// Permission name.
        permission: String,
        /// How to grant it.
        hint: String,
    },
    /// A required external tool is missing (solvable — install hint shown).
    #[error("required tool missing: {tool}")]
    MissingTool {
        /// Binary name.
        tool: String,
        /// Install hint.
        hint: String,
    },
    /// A step parameter is missing or malformed.
    #[error("invalid parameter '{param}': {message}")]
    InvalidParam {
        /// Parameter key.
        param: String,
        /// What is wrong with it.
        message: String,
    },
    /// A content coercion failed.
    #[error(transparent)]
    Coerce(#[from] CoerceError),
    /// The action id is not in the registry.
    #[error("unknown action '{0}'")]
    UnknownAction(String),
    /// A referenced variable does not exist.
    #[error("unknown variable '{0}'")]
    UnknownVariable(String),
    /// The run was cancelled by the user.
    #[error("run cancelled")]
    Cancelled,
    /// The run exceeded its wall-clock budget (automations always have one).
    #[error("run timed out")]
    Timeout,
    /// A loop exceeded the iteration cap — runaway automations must be
    /// impossible (§6.4).
    #[error("loop iteration cap of {cap} exceeded")]
    LoopCapExceeded {
        /// The configured cap.
        cap: u64,
    },
    /// `Run Shortcut` nesting exceeded the depth limit.
    #[error("Run Shortcut recursion depth {depth} exceeds the limit of {max}")]
    RecursionLimit {
        /// The depth that was attempted.
        depth: u32,
        /// The configured maximum.
        max: u32,
    },
    /// Any other action failure, with a human-readable message.
    #[error("{0}")]
    Failed(String),
    /// Underlying IO failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl From<lightning_platform::PlatformError> for ActionError {
    fn from(err: lightning_platform::PlatformError) -> Self {
        use lightning_platform::PlatformError as P;
        match err {
            P::Unsupported { os, reason } => ActionError::Unsupported { os, reason },
            P::MissingTool { tool, hint } => ActionError::MissingTool { tool, hint },
            P::MissingPermission { permission, hint } => {
                ActionError::NeedsOsPermission { permission, hint }
            }
            P::CommandFailed(message) => ActionError::Failed(message),
            P::Io(io) => ActionError::Io(io),
        }
    }
}

/// Errors while migrating a `.lightning` document to the current schema.
#[derive(Debug, thiserror::Error)]
pub enum MigrateError {
    /// The document has no readable `schemaVersion`.
    #[error("not a .lightning file: missing schemaVersion")]
    InvalidFile,
    /// The document is newer than this build (or zero).
    #[error("unsupported schemaVersion {found} (this build supports up to {current})")]
    UnsupportedVersion {
        /// Version found in the file.
        found: u32,
        /// Version this build writes.
        current: u32,
    },
    /// A migration step for this version is missing — a bug, caught by tests.
    #[error("no migration registered from schemaVersion {from}")]
    MissingMigration {
        /// The version lacking a migration.
        from: u32,
    },
    /// A migration step failed on this document.
    #[error("migration from {from} failed: {message}")]
    StepFailed {
        /// The version whose migration failed.
        from: u32,
        /// Failure detail.
        message: String,
    },
}
