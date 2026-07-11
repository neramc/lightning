// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! FreeBSD implementation of [`lightning_platform::PlatformOps`] (Tier 3).
//!
//! Reuses the Linux D-Bus/X11 approaches where ports exist; systemd-only
//! actions surface the unsupported badge with `os = FreeBSD` (CLAUDE.md §8.4).
//! On non-FreeBSD targets the crate compiles to nothing.

#[cfg(target_os = "freebsd")]
mod imp;

#[cfg(target_os = "freebsd")]
pub use imp::BsdPlatform;
