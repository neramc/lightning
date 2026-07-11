// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! # lightning-core
//!
//! The heart of Lightning (CLAUDE.md §6): the typed **content model** with its
//! explicit coercion table, the **shortcut model** serialized as versioned
//! `.lightning` JSON, the tree-walking async **execution engine**, and the
//! **schema migration** framework that guarantees old files always open.
//!
//! This crate is OS-agnostic. Anything platform-specific flows through the
//! [`lightning_platform`] capability types carried in the [`RunContext`].

#![deny(missing_docs)]

pub mod content;
pub mod engine;
mod error;
pub mod migrate;
mod permission;
mod run_log;
mod shortcut;

pub use content::{Content, ContentKind};
pub use engine::{
    ActionInvoker, CondOp, Engine, Flow, RunContext, RunLimits, RunProgress, ShortcutResolver,
    StepPhase,
};
pub use error::{ActionError, CoerceError, MigrateError};
pub use migrate::CURRENT_SCHEMA_VERSION;
pub use permission::PermissionClass;
pub use run_log::{LogLevel, RunLog, RunLogEntry};
pub use shortcut::{Branch, ErrorPolicy, Icon, ParamValue, Shortcut, Step, TriggerConfig};
