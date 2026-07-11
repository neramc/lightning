// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Schedule trigger — fires daily at a wall-clock time (§8.5; RRULE and cron
//! expressions layer on top of this same next-occurrence loop).

use chrono::{DateTime, Duration as ChronoDuration, Local, NaiveTime, TimeZone};
use lightning_platform::PlatformSupport;
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::{EventBus, Trigger, TriggerError, TriggerEvent};

/// `trigger.schedule`
pub struct ScheduleTrigger;

#[derive(Debug, Deserialize)]
struct Config {
    /// Local wall-clock time, `"HH:MM"`.
    time: String,
}

/// Next occurrence of `HH:MM` local time strictly after `now`.
fn next_occurrence(now: DateTime<Local>, hour: u32, minute: u32) -> Option<DateTime<Local>> {
    let time = NaiveTime::from_hms_opt(hour, minute, 0)?;
    let today = now.date_naive().and_time(time);
    let candidate = Local.from_local_datetime(&today).earliest()?;
    if candidate > now {
        Some(candidate)
    } else {
        let tomorrow = (now.date_naive() + ChronoDuration::days(1)).and_time(time);
        Local.from_local_datetime(&tomorrow).earliest()
    }
}

fn parse_hh_mm(value: &str) -> Result<(u32, u32), TriggerError> {
    let (h, m) = value
        .split_once(':')
        .ok_or_else(|| TriggerError::InvalidConfig("time must be HH:MM".into()))?;
    let hour: u32 = h
        .parse()
        .map_err(|_| TriggerError::InvalidConfig("invalid hour".into()))?;
    let minute: u32 = m
        .parse()
        .map_err(|_| TriggerError::InvalidConfig("invalid minute".into()))?;
    if hour > 23 || minute > 59 {
        return Err(TriggerError::InvalidConfig("time out of range".into()));
    }
    Ok((hour, minute))
}

#[async_trait::async_trait]
impl Trigger for ScheduleTrigger {
    fn id(&self) -> &'static str {
        "trigger.schedule"
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
        let (hour, minute) = parse_hh_mm(&config.time)?;
        loop {
            let now = Local::now();
            let Some(next) = next_occurrence(now, hour, minute) else {
                return Err(TriggerError::InvalidConfig(
                    "unrepresentable local time".into(),
                ));
            };
            let wait = (next - now).to_std().unwrap_or_default();
            tokio::select! {
                () = cancel.cancelled() => return Ok(()),
                () = tokio::time::sleep(wait) => {
                    bus.publish(TriggerEvent::now(
                        self.id(),
                        serde_json::json!({ "scheduled": config.time }),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn next_occurrence_is_today_when_still_ahead() {
        let now = Local.with_ymd_and_hms(2026, 7, 11, 8, 0, 0).unwrap();
        let next = next_occurrence(now, 9, 30).unwrap();
        assert_eq!(next, Local.with_ymd_and_hms(2026, 7, 11, 9, 30, 0).unwrap());
    }

    #[test]
    fn next_occurrence_rolls_to_tomorrow() {
        let now = Local.with_ymd_and_hms(2026, 7, 11, 10, 0, 0).unwrap();
        let next = next_occurrence(now, 9, 30).unwrap();
        assert_eq!(next, Local.with_ymd_and_hms(2026, 7, 12, 9, 30, 0).unwrap());
    }

    #[test]
    fn config_validation() {
        assert!(parse_hh_mm("09:30").is_ok());
        assert!(parse_hh_mm("24:00").is_err());
        assert!(parse_hh_mm("nope").is_err());
    }
}
