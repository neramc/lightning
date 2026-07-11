// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The [`PlatformOps`] trait implemented by every `lightning-platform-*` crate.
//!
//! Adding OS-specific behavior for an action means adding a method here and
//! implementing it in **every** per-OS crate — unsupported OSes return
//! [`PlatformError::Unsupported`] with a reason, never `todo!()` and never a
//! silent no-op (CLAUDE.md §8.6).

use std::path::Path;

use crate::capability::CapabilitySnapshot;
use crate::support::Os;

/// Errors from platform operations. The engine maps these onto `ActionError`
/// (`Unsupported` / `needsTool` / `needsPermission` — §8.1).
#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    /// The operation cannot work on this OS/environment at all.
    #[error("not supported on {os}: {reason}")]
    Unsupported {
        /// The OS (with environment refinement) that lacks support.
        os: String,
        /// Technical explanation.
        reason: String,
    },
    /// A required external tool is missing — solvable, so never blamed on the OS.
    #[error("required tool missing: {tool}")]
    MissingTool {
        /// Binary name.
        tool: String,
        /// Install hint shown behind the "Fix it" button.
        hint: String,
    },
    /// A required OS permission has not been granted — also solvable.
    #[error("missing permission: {permission}")]
    MissingPermission {
        /// Permission name (i18n key suffix).
        permission: String,
        /// How to grant it.
        hint: String,
    },
    /// An external command ran but failed.
    #[error("command failed: {0}")]
    CommandFailed(String),
    /// Underlying IO failure.
    #[error(transparent)]
    Io(#[from] std::io::Error),
}

/// Operations Lightning needs from the host OS.
///
/// Implementations must be cheap to construct, thread-safe, and must never
/// block the async runtime: shell-outs use `tokio::process`, and any heavy
/// probing happens inside `probe()` which callers run off the hot path.
#[async_trait::async_trait]
pub trait PlatformOps: Send + Sync {
    /// The OS this implementation targets.
    fn os(&self) -> Os;

    /// Probe the runtime environment (session type, permissions, optional
    /// tools). Called on startup and on relevant system changes; the result
    /// is broadcast as `capability://changed`.
    fn probe(&self) -> CapabilitySnapshot;

    /// Reveal a path in the OS file manager (Explorer / Finder /
    /// `org.freedesktop.FileManager1` with `xdg-open` fallback).
    async fn reveal_in_file_manager(&self, path: &Path) -> Result<(), PlatformError>;

    /// Show a plain desktop notification (rich actions are a separate,
    /// per-OS action).
    async fn send_notification(&self, title: &str, body: &str) -> Result<(), PlatformError>;

    /// Open a URL with the default handler.
    async fn open_url(&self, url: &str) -> Result<(), PlatformError>;

    /// Read text from the system clipboard.
    async fn clipboard_read_text(&self) -> Result<String, PlatformError>;

    /// Write text to the system clipboard.
    async fn clipboard_write_text(&self, text: &str) -> Result<(), PlatformError>;
}
