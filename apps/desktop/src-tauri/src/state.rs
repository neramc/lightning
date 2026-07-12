// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Shared app state managed by Tauri. Assembly only — all real logic lives
//! in the crates (§6.1).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use lightning_actions::Registry;
use lightning_core::Engine;
use lightning_platform::PlatformOps;
use lightning_store::SqliteIndex;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

/// The one place that selects the host platform implementation (§6.1: thin
/// glue in the shell; everything else takes `Arc<dyn PlatformOps>`).
fn host_platform() -> Arc<dyn PlatformOps> {
    #[cfg(target_os = "windows")]
    {
        Arc::new(lightning_platform_windows::WindowsPlatform::new())
    }
    #[cfg(target_os = "macos")]
    {
        Arc::new(lightning_platform_macos::MacPlatform::new())
    }
    #[cfg(target_os = "linux")]
    {
        Arc::new(lightning_platform_linux::LinuxPlatform::new())
    }
    #[cfg(target_os = "freebsd")]
    {
        Arc::new(lightning_platform_bsd::BsdPlatform::new())
    }
}

pub struct AppState {
    pub registry: Arc<Registry>,
    pub engine: Arc<Engine>,
    pub platform: Arc<dyn PlatformOps>,
    pub index: Mutex<SqliteIndex>,
    pub shortcuts_dir: PathBuf,
    /// Cancellation tokens for in-flight runs, keyed by run id.
    pub running: Mutex<HashMap<Uuid, CancellationToken>>,
}

impl AppState {
    pub fn initialize() -> anyhow::Result<Self> {
        let mut registry = Registry::with_builtins();
        registry.register(Arc::new(lightning_scripting::RunJavaScript));
        let registry = Arc::new(registry);
        let engine = Arc::new(Engine::new(registry.clone()));

        let shortcuts_dir = lightning_store::shortcuts_dir()?;
        std::fs::create_dir_all(&shortcuts_dir)?;
        let index = SqliteIndex::open(&lightning_store::index_path()?)?;
        // Files are truth; rebuild the cache on startup (§6.3).
        let indexed = index.reindex(&shortcuts_dir)?;
        tracing::info!(indexed, "store reindexed");

        Ok(Self {
            registry,
            engine,
            platform: host_platform(),
            index: Mutex::new(index),
            shortcuts_dir,
            running: Mutex::new(HashMap::new()),
        })
    }
}
