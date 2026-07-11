// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! `run://progress` event payloads (CLAUDE.md §6.4, §9.3). These drive the
//! editor's run animation: blocks light up top-to-bottom, success ticks,
//! error shakes.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lifecycle phase of one step within a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "camelCase")]
pub enum StepPhase {
    /// The step began executing.
    Started,
    /// The step finished; `preview` is a short single-line output preview.
    Finished {
        /// Truncated output preview.
        preview: String,
    },
    /// The step failed and the run stops (Stop policy).
    Failed {
        /// Error message.
        message: String,
    },
    /// The step failed but was skipped (Continue policy).
    Skipped {
        /// Why it was skipped.
        message: String,
    },
}

/// One `run://progress` event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunProgress {
    /// The run this event belongs to.
    pub run_id: Uuid,
    /// The step concerned.
    pub step: Uuid,
    /// The step's action id (lets the UI animate without a lookup).
    pub action_id: String,
    /// What happened.
    #[serde(flatten)]
    pub phase: StepPhase,
}
