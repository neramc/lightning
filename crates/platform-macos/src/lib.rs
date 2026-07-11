// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! macOS implementation of [`lightning_platform::PlatformOps`].
//!
//! Uses the system's first-party command surfaces (`open`, `osascript`,
//! `pbcopy`/`pbpaste`) — native AppKit/CoreFoundation bindings replace these
//! incrementally. Accessibility and Screen Recording are per-app TCC
//! permissions the probe reports as degraded until granted. On non-macOS
//! targets the crate compiles to nothing.

#[cfg(target_os = "macos")]
mod imp;

#[cfg(target_os = "macos")]
pub use imp::MacPlatform;
