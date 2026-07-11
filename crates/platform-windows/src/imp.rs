// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The actual Windows platform implementation.

use std::path::Path;

use lightning_platform::{
    Capability, CapabilitySnapshot, CapabilityStatus, Os, PlatformError, PlatformOps,
};
use tokio::process::Command;

/// Windows [`PlatformOps`] implementation.
#[derive(Default)]
pub struct WindowsPlatform;

impl WindowsPlatform {
    /// Create the platform handle.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

async fn powershell(script: &str) -> Result<std::process::Output, PlatformError> {
    // Arguments are argv arrays; the script is a fixed literal assembled by
    // us, with user data passed via -EncodedCommand-safe single quoting only.
    let output = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .await?;
    Ok(output)
}

fn ps_quote(value: &str) -> String {
    // PowerShell single-quoted literal: only `'` needs doubling.
    format!("'{}'", value.replace('\'', "''"))
}

#[async_trait::async_trait]
impl PlatformOps for WindowsPlatform {
    fn os(&self) -> Os {
        Os::Windows
    }

    fn probe(&self) -> CapabilitySnapshot {
        // Elevation state and PowerShell version are queried lazily by the
        // actions that need them; the base snapshot records what is known
        // synchronously. SendInput, SMTC, and toasts are OS-native.
        CapabilitySnapshot::unconstrained(Os::Windows)
            .with(Capability::InputInjection, CapabilityStatus::Available)
            .with(Capability::Screenshot, CapabilityStatus::Available)
            .with(Capability::Notifications, CapabilityStatus::Available)
            .with(Capability::MediaControl, CapabilityStatus::Available)
            .with(Capability::GlobalHotkeys, CapabilityStatus::Available)
            .with(Capability::WindowManagement, CapabilityStatus::Available)
    }

    async fn reveal_in_file_manager(&self, path: &Path) -> Result<(), PlatformError> {
        // `explorer /select,<path>` — documented Explorer switch. Explorer
        // returns a nonzero exit code even on success, so only spawn errors
        // are surfaced.
        Command::new("explorer")
            .arg(format!("/select,{}", path.display()))
            .spawn()?
            .wait()
            .await?;
        Ok(())
    }

    async fn send_notification(&self, title: &str, body: &str) -> Result<(), PlatformError> {
        // WinRT toast via PowerShell projection — replaced by a native WinRT
        // call when the windows crate lands. Falls back to a message box-free
        // silent failure never: errors are surfaced.
        let script = format!(
            "[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null; \
             $t = [Windows.UI.Notifications.ToastNotificationManager]::GetTemplateContent([Windows.UI.Notifications.ToastTemplateType]::ToastText02); \
             $n = $t.GetElementsByTagName('text'); \
             $n.Item(0).AppendChild($t.CreateTextNode({title})) | Out-Null; \
             $n.Item(1).AppendChild($t.CreateTextNode({body})) | Out-Null; \
             [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier('Lightning').Show([Windows.UI.Notifications.ToastNotification]::new($t))",
            title = ps_quote(title),
            body = ps_quote(body),
        );
        let output = powershell(&script).await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(PlatformError::CommandFailed(
                String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            ))
        }
    }

    async fn open_url(&self, url: &str) -> Result<(), PlatformError> {
        let output = Command::new("cmd")
            .args(["/C", "start", "", url])
            .output()
            .await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(PlatformError::CommandFailed("start failed".into()))
        }
    }

    async fn clipboard_read_text(&self) -> Result<String, PlatformError> {
        let output = powershell("Get-Clipboard -Raw").await?;
        if !output.status.success() {
            return Err(PlatformError::CommandFailed("Get-Clipboard failed".into()));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    async fn clipboard_write_text(&self, text: &str) -> Result<(), PlatformError> {
        let script = format!("Set-Clipboard -Value {}", ps_quote(text));
        let output = powershell(&script).await?;
        if output.status.success() {
            Ok(())
        } else {
            Err(PlatformError::CommandFailed("Set-Clipboard failed".into()))
        }
    }
}
