// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The actual FreeBSD platform implementation (best-effort).

use std::path::Path;

use lightning_platform::{
    Capability, CapabilitySnapshot, CapabilityStatus, Os, PlatformError, PlatformOps,
};
use tokio::process::Command;

/// FreeBSD [`PlatformOps`] implementation.
#[derive(Default)]
pub struct BsdPlatform;

impl BsdPlatform {
    /// Create the platform handle.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

fn has_tool(name: &str) -> bool {
    which::which(name).is_ok()
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

#[async_trait::async_trait]
impl PlatformOps for BsdPlatform {
    fn os(&self) -> Os {
        Os::FreeBsd
    }

    fn probe(&self) -> CapabilitySnapshot {
        let x11 = std::env::var_os("DISPLAY").is_some();
        let environment = Some(if x11 { "X11" } else { "headless" }.to_owned());
        let mut snapshot = CapabilitySnapshot {
            os: Os::FreeBsd,
            environment,
            ..CapabilitySnapshot::unconstrained(Os::FreeBsd)
        };
        snapshot = snapshot.with(
            Capability::InputInjection,
            if x11 {
                CapabilityStatus::Available
            } else {
                CapabilityStatus::Unavailable { reason: "no X11 session".into(), fix: None }
            },
        );
        snapshot = snapshot.with(
            Capability::Notifications,
            if has_tool("notify-send") {
                CapabilityStatus::Available
            } else {
                CapabilityStatus::Unavailable {
                    reason: "libnotify not installed".into(),
                    fix: Some(lightning_platform::CapabilityFix::InstallTool {
                        tool: "notify-send".into(),
                        hint: "pkg install libnotify".into(),
                    }),
                }
            },
        );
        snapshot
    }

    async fn reveal_in_file_manager(&self, path: &Path) -> Result<(), PlatformError> {
        let parent = path.parent().unwrap_or(path);
        run_checked("xdg-open", &[&parent.display().to_string()]).await
    }

    async fn send_notification(&self, title: &str, body: &str) -> Result<(), PlatformError> {
        if !has_tool("notify-send") {
            return Err(PlatformError::MissingTool {
                tool: "notify-send".into(),
                hint: "pkg install libnotify".into(),
            });
        }
        run_checked("notify-send", &["--app-name", "Lightning", title, body]).await
    }

    async fn open_url(&self, url: &str) -> Result<(), PlatformError> {
        run_checked("xdg-open", &[url]).await
    }

    async fn clipboard_read_text(&self) -> Result<String, PlatformError> {
        if !has_tool("xclip") {
            return Err(PlatformError::MissingTool {
                tool: "xclip".into(),
                hint: "pkg install xclip".into(),
            });
        }
        let output = Command::new("xclip")
            .args(["-selection", "clipboard", "-o"])
            .output()
            .await?;
        if !output.status.success() {
            return Err(PlatformError::CommandFailed("xclip failed".into()));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    async fn clipboard_write_text(&self, text: &str) -> Result<(), PlatformError> {
        use tokio::io::AsyncWriteExt;
        if !has_tool("xclip") {
            return Err(PlatformError::MissingTool {
                tool: "xclip".into(),
                hint: "pkg install xclip".into(),
            });
        }
        let mut child = Command::new("xclip")
            .args(["-selection", "clipboard"])
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).await?;
        }
        let status = child.wait().await?;
        if status.success() {
            Ok(())
        } else {
            Err(PlatformError::CommandFailed(format!("xclip exited with {status}")))
        }
    }
}
