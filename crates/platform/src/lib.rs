// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! # lightning-platform
//!
//! OS abstraction layer for Lightning (CLAUDE.md §6.6).
//!
//! This crate owns three things:
//!
//! - the **static support matrix** vocabulary ([`Os`], [`SupportLevel`],
//!   [`PlatformSupport`]) that every action declares,
//! - the **runtime capability probe** result ([`CapabilitySnapshot`]) — because
//!   compile-target support is necessary but not sufficient (X11 vs Wayland,
//!   missing permissions, missing tools),
//! - the [`PlatformOps`] trait that per-OS crates (`lightning-platform-windows`,
//!   `-macos`, `-linux`, `-bsd`) implement.
//!
//! Effective status of an action = static support ∩ runtime probe. When either
//! side says no, the engine returns `Unsupported { os, reason }` and the UI
//! renders the localized badge — capabilities are *data*, never per-OS UI hacks.

#![deny(missing_docs)]

mod capability;
mod support;
pub mod testing;
mod traits;

pub use capability::{Capability, CapabilityFix, CapabilitySnapshot, CapabilityStatus};
pub use support::{Os, PlatformSupport, SupportLevel};
pub use traits::{PlatformError, PlatformOps};
