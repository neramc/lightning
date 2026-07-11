// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Matches trigger events to enabled automations and enqueues runs with
//! per-automation cooldown (§6.7). Runaway automations must be impossible:
//! the cooldown floor is enforced here, and the engine adds loop caps and a
//! wall-clock timeout per run.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{EventBus, TriggerEvent};

/// A registered automation the scheduler can match.
#[derive(Debug, Clone)]
pub struct Automation {
    /// The automation's shortcut id.
    pub shortcut_id: Uuid,
    /// The trigger id its trigger block names.
    pub trigger_id: String,
    /// Whether it is armed.
    pub enabled: bool,
    /// Cooldown between runs; `None` uses [`crate::DEFAULT_COOLDOWN_MS`].
    pub cooldown: Option<Duration>,
}

/// Callback invoked for each matched run (wired to the engine by the shell).
pub type RunHandler = Arc<dyn Fn(Uuid, TriggerEvent) + Send + Sync>;

/// The event → automation matcher.
pub struct Scheduler {
    automations: Vec<Automation>,
    last_run: HashMap<Uuid, Instant>,
    handler: RunHandler,
}

impl Scheduler {
    /// A scheduler dispatching matched runs to `handler`.
    #[must_use]
    pub fn new(handler: RunHandler) -> Self {
        Self {
            automations: Vec::new(),
            last_run: HashMap::new(),
            handler,
        }
    }

    /// Replace the automation set (called when the store changes).
    pub fn set_automations(&mut self, automations: Vec<Automation>) {
        self.automations = automations;
    }

    /// Offer one event; returns the shortcut ids that were dispatched.
    pub fn offer(&mut self, event: &TriggerEvent) -> Vec<Uuid> {
        let now = Instant::now();
        let mut dispatched = Vec::new();
        for automation in &self.automations {
            if !automation.enabled || automation.trigger_id != event.trigger_id {
                continue;
            }
            let cooldown = automation
                .cooldown
                .unwrap_or(Duration::from_millis(crate::DEFAULT_COOLDOWN_MS));
            if let Some(last) = self.last_run.get(&automation.shortcut_id)
                && now.duration_since(*last) < cooldown
            {
                tracing::debug!(
                    shortcut = %automation.shortcut_id,
                    "automation suppressed by cooldown"
                );
                continue;
            }
            self.last_run.insert(automation.shortcut_id, now);
            (self.handler)(automation.shortcut_id, event.clone());
            dispatched.push(automation.shortcut_id);
        }
        dispatched
    }

    /// Consume a bus until cancelled. The shell spawns this on tokio.
    pub async fn run(mut self, bus: EventBus, cancel: CancellationToken) {
        let mut rx = bus.subscribe();
        loop {
            tokio::select! {
                () = cancel.cancelled() => return,
                event = rx.recv() => match event {
                    Ok(event) => {
                        let _ = self.offer(&event);
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(skipped, "scheduler lagged behind the event bus");
                    }
                    Err(tokio::sync::broadcast::error::RecvError::Closed) => return,
                },
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    fn scheduler_with_log() -> (Scheduler, Arc<Mutex<Vec<Uuid>>>) {
        let log = Arc::new(Mutex::new(Vec::new()));
        let log_clone = Arc::clone(&log);
        let handler: RunHandler = Arc::new(move |id, _event| {
            log_clone.lock().unwrap().push(id);
        });
        (Scheduler::new(handler), log)
    }

    #[test]
    fn matches_only_enabled_automations_with_the_same_trigger() {
        let (mut scheduler, log) = scheduler_with_log();
        let armed = Uuid::new_v4();
        let disarmed = Uuid::new_v4();
        scheduler.set_automations(vec![
            Automation {
                shortcut_id: armed,
                trigger_id: "trigger.interval".into(),
                enabled: true,
                cooldown: None,
            },
            Automation {
                shortcut_id: disarmed,
                trigger_id: "trigger.interval".into(),
                enabled: false,
                cooldown: None,
            },
            Automation {
                shortcut_id: Uuid::new_v4(),
                trigger_id: "trigger.file_changed".into(),
                enabled: true,
                cooldown: None,
            },
        ]);
        let hits = scheduler.offer(&TriggerEvent::now(
            "trigger.interval",
            serde_json::json!({}),
        ));
        assert_eq!(hits, vec![armed]);
        assert_eq!(log.lock().unwrap().as_slice(), &[armed]);
    }

    #[test]
    fn cooldown_suppresses_rapid_refires() {
        let (mut scheduler, log) = scheduler_with_log();
        let id = Uuid::new_v4();
        scheduler.set_automations(vec![Automation {
            shortcut_id: id,
            trigger_id: "trigger.interval".into(),
            enabled: true,
            cooldown: Some(Duration::from_secs(60)),
        }]);
        let event = TriggerEvent::now("trigger.interval", serde_json::json!({}));
        assert_eq!(scheduler.offer(&event).len(), 1);
        assert_eq!(
            scheduler.offer(&event).len(),
            0,
            "second fire within cooldown"
        );
        assert_eq!(log.lock().unwrap().len(), 1);
    }
}
