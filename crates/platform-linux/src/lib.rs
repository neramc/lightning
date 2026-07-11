// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Linux implementation of [`lightning_platform::PlatformOps`].
//!
//! X11 vs Wayland is a **runtime capability** matter, never a build split
//! (CLAUDE.md §7): this crate compiles once and probes the session type,
//! ydotool/libei availability, NetworkManager, and the notification daemon at
//! runtime. On non-Linux targets the crate compiles to nothing so that
//! `cargo check --workspace` passes everywhere.

#[cfg(target_os = "linux")]
mod imp;

#[cfg(target_os = "linux")]
pub use imp::LinuxPlatform;
