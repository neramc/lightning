// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The user-visible run log (distinct from the developer `tracing` log —
//! CLAUDE.md §4). Collected per run, shown in the run inspector, persisted
//! into run history by `lightning-store`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Severity of a run log entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LogLevel {
    /// Informational (step outputs, Show Result).
    Info,
    /// Something was skipped or degraded.
    Warn,
    /// A step failed.
    Error,
}

/// One line in the run log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunLogEntry {
    /// When the entry was recorded.
    pub at: DateTime<Utc>,
    /// The step it concerns, if any.
    pub step: Option<Uuid>,
    /// Severity.
    pub level: LogLevel,
    /// Human-readable message.
    pub message: String,
}

/// The collected log of one run.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RunLog {
    entries: Vec<RunLogEntry>,
}

impl RunLog {
    /// All entries, oldest first.
    #[must_use]
    pub fn entries(&self) -> &[RunLogEntry] {
        &self.entries
    }

    /// Record an entry.
    pub fn push(&mut self, level: LogLevel, step: Option<Uuid>, message: impl Into<String>) {
        self.entries.push(RunLogEntry {
            at: Utc::now(),
            step,
            level,
            message: message.into(),
        });
    }

    /// Record an informational entry.
    pub fn info(&mut self, step: Option<Uuid>, message: impl Into<String>) {
        self.push(LogLevel::Info, step, message);
    }

    /// Record a warning.
    pub fn warn(&mut self, step: Option<Uuid>, message: impl Into<String>) {
        self.push(LogLevel::Warn, step, message);
    }

    /// Record an error.
    pub fn error(&mut self, step: Option<Uuid>, message: impl Into<String>) {
        self.push(LogLevel::Error, step, message);
    }
}
