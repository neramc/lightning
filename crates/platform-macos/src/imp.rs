// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The actual macOS platform implementation.

use std::path::Path;

use lightning_platform::{
    Capability, CapabilitySnapshot, CapabilityStatus, Os, PlatformError, PlatformOps,
};
use tokio::process::Command;

/// macOS [`PlatformOps`] implementation.
#[derive(Default)]
pub struct MacPlatform;

impl MacPlatform {
    /// Create the platform handle.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

async fn run_checked(program: &str, args: &[&str]) -> Result<(), PlatformError> {
    let output = Command::new(program).args(args).output().await?;
    if output.status.success() {
        Ok(())
    } else {
        Err(PlatformError::CommandFailed(format!(
            "{program} exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

fn applescript_quote(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

#[async_trait::async_trait]
impl PlatformOps for MacPlatform {
    fn os(&self) -> Os {
        Os::MacOs
    }

    fn probe(&self) -> CapabilitySnapshot {
        // TCC permissions (Accessibility, Screen Recording, per-app
        // Automation) cannot be read without triggering prompts, so the base
        // probe reports them as degraded with the grant path; the native
        // AXIsProcessTrusted/CGPreflightScreenCaptureAccess probes replace
        // this when the FFI module lands.
        let grant = |permission: &str| CapabilityStatus::Degraded {
            reason: format!(
                "requires the {permission} permission (System Settings → Privacy & Security)"
            ),
        };
        CapabilitySnapshot::unconstrained(Os::MacOs)
            .with(Capability::InputInjection, grant("Accessibility"))
            .with(Capability::WindowManagement, grant("Accessibility"))
            .with(Capability::Screenshot, grant("Screen Recording"))
            .with(Capability::Notifications, CapabilityStatus::Available)
            .with(Capability::MediaControl, CapabilityStatus::Available)
            .with(Capability::SpeechSynthesis, CapabilityStatus::Available)
    }

    async fn reveal_in_file_manager(&self, path: &Path) -> Result<(), PlatformError> {
        run_checked("open", &["-R", &path.display().to_string()]).await
    }

    async fn send_notification(&self, title: &str, body: &str) -> Result<(), PlatformError> {
        let script = format!(
            "display notification {} with title {}",
            applescript_quote(body),
            applescript_quote(title),
        );
        run_checked("osascript", &["-e", &script]).await
    }

    async fn open_url(&self, url: &str) -> Result<(), PlatformError> {
        run_checked("open", &[url]).await
    }

    async fn clipboard_read_text(&self) -> Result<String, PlatformError> {
        let output = Command::new("pbpaste").output().await?;
        if !output.status.success() {
            return Err(PlatformError::CommandFailed("pbpaste failed".into()));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    async fn clipboard_write_text(&self, text: &str) -> Result<(), PlatformError> {
        use tokio::io::AsyncWriteExt;
        let mut child = Command::new("pbcopy")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).await?;
        }
        let status = child.wait().await?;
        if status.success() {
            Ok(())
        } else {
            Err(PlatformError::CommandFailed(format!(
                "pbcopy exited with {status}"
            )))
        }
    }
}
