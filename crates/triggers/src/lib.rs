// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! # lightning-triggers
//!
//! The automation runtime (CLAUDE.md §6.7): each [`Trigger`] runs as a tokio
//! task publishing [`TriggerEvent`]s to a broadcast [`EventBus`]; the
//! [`Scheduler`](scheduler::Scheduler) matches events to enabled automations
//! and enqueues runs with per-automation cooldown (default 2 s).

pub mod file_watch;
pub mod interval;
pub mod schedule;
pub mod scheduler;

use chrono::{DateTime, Utc};
use lightning_platform::PlatformSupport;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

/// Default per-automation cooldown (§6.7).
pub const DEFAULT_COOLDOWN_MS: u64 = 2_000;

/// One firing of a trigger.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerEvent {
    /// The trigger id that fired (e.g. `trigger.interval`).
    pub trigger_id: String,
    /// When it fired.
    pub fired_at: DateTime<Utc>,
    /// Trigger-specific payload (e.g. the changed path).
    #[serde(default)]
    pub payload: serde_json::Value,
}

impl TriggerEvent {
    /// An event for `trigger_id` firing now.
    #[must_use]
    pub fn now(trigger_id: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            trigger_id: trigger_id.into(),
            fired_at: Utc::now(),
            payload,
        }
    }
}

/// Errors from trigger startup or runtime.
#[derive(Debug, thiserror::Error)]
pub enum TriggerError {
    /// The trigger's config block is malformed.
    #[error("invalid trigger config: {0}")]
    InvalidConfig(String),
    /// The trigger cannot run in this environment.
    #[error("trigger unsupported: {0}")]
    Unsupported(String),
    /// Underlying IO failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// The broadcast bus every trigger publishes into.
#[derive(Clone)]
pub struct EventBus {
    tx: tokio::sync::broadcast::Sender<TriggerEvent>,
}

impl EventBus {
    /// A bus with the given buffered capacity.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = tokio::sync::broadcast::channel(capacity);
        Self { tx }
    }

    /// Publish an event (lagging subscribers drop oldest events).
    pub fn publish(&self, event: TriggerEvent) {
        tracing::debug!(trigger = %event.trigger_id, "trigger fired");
        let _ = self.tx.send(event);
    }

    /// Subscribe to all trigger events.
    #[must_use]
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<TriggerEvent> {
        self.tx.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

/// A source of automation events (CLAUDE.md §8.5). Implementations follow
/// the §8.7 recipe: honest support matrix, sensible debounce defaults,
/// persistence round-trip test, i18n en+ko.
#[async_trait::async_trait]
pub trait Trigger: Send + Sync {
    /// Stable id (e.g. `trigger.file_changed`).
    fn id(&self) -> &'static str;

    /// Per-OS support (§8.5 table).
    fn supports(&self) -> PlatformSupport;

    /// Run until cancelled, publishing events to `bus`. `config` is the
    /// automation's trigger config block.
    async fn run(
        &self,
        config: serde_json::Value,
        bus: EventBus,
        cancel: CancellationToken,
    ) -> Result<(), TriggerError>;
}
