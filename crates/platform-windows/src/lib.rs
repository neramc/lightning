// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Windows implementation of [`lightning_platform::PlatformOps`].
//!
//! The scaffold shells out to documented OS entry points (`explorer
//! /select,`, PowerShell `Get-Clipboard`/`Set-Clipboard`, `cmd /C start`);
//! native Win32/WinRT bindings replace these hot paths incrementally without
//! changing the trait surface. On non-Windows targets the crate compiles to
//! nothing so `cargo check --workspace` passes everywhere.

#[cfg(target_os = "windows")]
mod imp;

#[cfg(target_os = "windows")]
pub use imp::WindowsPlatform;
