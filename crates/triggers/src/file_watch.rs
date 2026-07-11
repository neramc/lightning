// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! File / Folder Changed trigger — `notify`-based, debounced (§8.5).

use std::path::PathBuf;
use std::time::Duration;

use lightning_platform::PlatformSupport;
use notify::{RecursiveMode, Watcher};
use serde::Deserialize;
use tokio_util::sync::CancellationToken;

use crate::{EventBus, Trigger, TriggerError, TriggerEvent};

/// Default debounce for file events.
pub const DEFAULT_DEBOUNCE_MS: u64 = 500;

/// `trigger.file_changed`
pub struct FileWatchTrigger;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Config {
    /// File or directory to watch.
    path: PathBuf,
    /// Watch subdirectories too.
    #[serde(default)]
    recursive: bool,
    /// Debounce window in milliseconds.
    #[serde(default = "default_debounce")]
    debounce_ms: u64,
}

fn default_debounce() -> u64 {
    DEFAULT_DEBOUNCE_MS
}

#[async_trait::async_trait]
impl Trigger for FileWatchTrigger {
    fn id(&self) -> &'static str {
        "trigger.file_changed"
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
        if !config.path.exists() {
            return Err(TriggerError::InvalidConfig(format!(
                "path does not exist: {}",
                config.path.display()
            )));
        }

        // notify's watcher is callback-based and synchronous; bridge into
        // tokio through an unbounded channel.
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Vec<PathBuf>>();
        let mut watcher =
            notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
                if let Ok(event) = result
                    && matches!(
                        event.kind,
                        notify::EventKind::Create(_)
                            | notify::EventKind::Modify(_)
                            | notify::EventKind::Remove(_)
                    )
                {
                    let _ = tx.send(event.paths);
                }
            })
            .map_err(|err| TriggerError::InvalidConfig(err.to_string()))?;

        let mode = if config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        watcher
            .watch(&config.path, mode)
            .map_err(|err| TriggerError::InvalidConfig(err.to_string()))?;

        let debounce = Duration::from_millis(config.debounce_ms.max(50));
        loop {
            // Wait for the first event of a burst…
            let first = tokio::select! {
                () = cancel.cancelled() => return Ok(()),
                paths = rx.recv() => match paths {
                    Some(paths) => paths,
                    None => return Ok(()),
                },
            };
            // …then swallow the rest of the burst inside the debounce window.
            let mut paths = first;
            loop {
                tokio::select! {
                    () = cancel.cancelled() => return Ok(()),
                    () = tokio::time::sleep(debounce) => break,
                    more = rx.recv() => match more {
                        Some(mut extra) => paths.append(&mut extra),
                        None => break,
                    },
                }
            }
            paths.sort();
            paths.dedup();
            bus.publish(TriggerEvent::now(
                self.id(),
                serde_json::json!({
                    "paths": paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>(),
                }),
            ));
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn missing_path_is_invalid_config() {
        let err = FileWatchTrigger
            .run(
                serde_json::json!({ "path": "/definitely/not/here" }),
                EventBus::default(),
                CancellationToken::new(),
            )
            .await
            .unwrap_err();
        assert!(matches!(err, TriggerError::InvalidConfig(_)));
    }

    #[tokio::test]
    async fn fires_debounced_on_writes() {
        let dir = std::env::temp_dir().join(format!("lightning-watch-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&dir).unwrap();
        let bus = EventBus::default();
        let mut rx = bus.subscribe();
        let cancel = CancellationToken::new();
        let handle = tokio::spawn({
            let bus = bus.clone();
            let cancel = cancel.clone();
            let dir = dir.clone();
            async move {
                FileWatchTrigger
                    .run(
                        serde_json::json!({ "path": dir, "debounceMs": 100 }),
                        bus,
                        cancel,
                    )
                    .await
            }
        });
        // Give the watcher a moment to arm, then write twice quickly.
        tokio::time::sleep(Duration::from_millis(300)).await;
        std::fs::write(dir.join("a.txt"), "1").unwrap();
        std::fs::write(dir.join("a.txt"), "2").unwrap();

        let event = tokio::time::timeout(Duration::from_secs(5), rx.recv())
            .await
            .expect("debounced event within 5s")
            .unwrap();
        assert_eq!(event.trigger_id, "trigger.file_changed");
        cancel.cancel();
        handle.await.unwrap().unwrap();
        std::fs::remove_dir_all(&dir).ok();
    }
}
