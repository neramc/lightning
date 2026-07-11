// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Interval trigger — fires every N seconds (§8.5, ✅ everywhere).

use lightning_platform::PlatformSupport;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::{EventBus, Trigger, TriggerError, TriggerEvent};

/// `trigger.interval`
pub struct IntervalTrigger;

#[derive(Debug, Deserialize)]
struct Config {
    /// Seconds between firings; floor 1 s so a bad config cannot spin.
    seconds: f64,
}

#[async_trait::async_trait]
impl Trigger for IntervalTrigger {
    fn id(&self) -> &'static str {
        "trigger.interval"
    }

    fn supports(&self) -> PlatformSupport {
        PlatformSupport::all_full()
    }

    async fn run(
        &self,
        config: serde_json::Value,
        bus: EventBus,
        cancel: CancellationToken,
    ) -> Result<(), TriggerError> {
        let config: Config = serde_json::from_value(config)
            .map_err(|err| TriggerError::InvalidConfig(err.to_string()))?;
        if !config.seconds.is_finite() || config.seconds < 1.0 {
            return Err(TriggerError::InvalidConfig("seconds must be ≥ 1".into()));
        }
        let period = std::time::Duration::from_secs_f64(config.seconds);
        let mut interval = tokio::time::interval(period);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        interval.tick().await; // the first tick is immediate — skip it
        loop {
            tokio::select! {
                () = cancel.cancelled() => return Ok(()),
                _ = interval.tick() => {
                    bus.publish(TriggerEvent::now(self.id(), serde_json::json!({})));
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn rejects_sub_second_intervals() {
        let cancel = CancellationToken::new();
        let err = IntervalTrigger
            .run(
                serde_json::json!({ "seconds": 0.01 }),
                EventBus::default(),
                cancel,
            )
            .await
            .unwrap_err();
        assert!(matches!(err, TriggerError::InvalidConfig(_)));
    }

    #[tokio::test]
    async fn fires_until_cancelled() {
        let bus = EventBus::default();
        let mut rx = bus.subscribe();
        let cancel = CancellationToken::new();
        let handle = tokio::spawn({
            let bus = bus.clone();
            let cancel = cancel.clone();
            async move {
                IntervalTrigger
                    .run(serde_json::json!({ "seconds": 1 }), bus, cancel)
                    .await
            }
        });
        // One real tick (~1s) keeps the test honest without test-util clocks.
        let event = tokio::time::timeout(std::time::Duration::from_secs(3), rx.recv())
            .await
            .unwrap();
        assert_eq!(event.unwrap().trigger_id, "trigger.interval");
        cancel.cancel();
        handle.await.unwrap().unwrap();
    }
}
