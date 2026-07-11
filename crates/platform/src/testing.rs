// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Fake platform for engine and capability tests (CLAUDE.md §11: capability
//! probes get fake-environment tests, e.g. "Wayland without ydotool" must
//! yield `Unsupported` with a reason).

use std::path::Path;
use std::sync::Mutex;

use crate::capability::CapabilitySnapshot;
use crate::support::Os;
use crate::traits::{PlatformError, PlatformOps};

/// An in-memory [`PlatformOps`] with a caller-supplied snapshot.
pub struct FakePlatform {
    snapshot: CapabilitySnapshot,
    clipboard: Mutex<String>,
    /// Notifications sent through this fake, for assertions.
    pub notifications: Mutex<Vec<(String, String)>>,
}

impl FakePlatform {
    /// A fake reporting the given snapshot.
    #[must_use]
    pub fn new(snapshot: CapabilitySnapshot) -> Self {
        Self {
            snapshot,
            clipboard: Mutex::new(String::new()),
            notifications: Mutex::new(Vec::new()),
        }
    }

    /// A fake for `os` with no capability constraints.
    #[must_use]
    pub fn unconstrained(os: Os) -> Self {
        Self::new(CapabilitySnapshot::unconstrained(os))
    }
}

#[async_trait::async_trait]
impl PlatformOps for FakePlatform {
    fn os(&self) -> Os {
        self.snapshot.os
    }

    fn probe(&self) -> CapabilitySnapshot {
        self.snapshot.clone()
    }

    async fn reveal_in_file_manager(&self, _path: &Path) -> Result<(), PlatformError> {
        Ok(())
    }

    async fn send_notification(&self, title: &str, body: &str) -> Result<(), PlatformError> {
        self.notifications
            .lock()
            .map_err(|_| PlatformError::CommandFailed("poisoned lock".into()))?
            .push((title.to_owned(), body.to_owned()));
        Ok(())
    }

    async fn open_url(&self, _url: &str) -> Result<(), PlatformError> {
        Ok(())
    }

    async fn clipboard_read_text(&self) -> Result<String, PlatformError> {
        Ok(self
            .clipboard
            .lock()
            .map_err(|_| PlatformError::CommandFailed("poisoned lock".into()))?
            .clone())
    }

    async fn clipboard_write_text(&self, text: &str) -> Result<(), PlatformError> {
        *self
            .clipboard
            .lock()
            .map_err(|_| PlatformError::CommandFailed("poisoned lock".into()))? = text.to_owned();
        Ok(())
    }
}
