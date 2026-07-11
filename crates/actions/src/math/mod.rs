// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Math & numbers (§8.3 C) — pure and fully cross-platform.

mod calculate;
mod list_statistics;
mod random_number;
mod round_number;

pub use calculate::Calculate;
pub use list_statistics::ListStatistics;
pub use random_number::RandomNumber;
pub use round_number::RoundNumber;

use std::sync::Arc;

use crate::Registry;

/// Register every action in this category.
pub fn register(registry: &mut Registry) {
    registry.register(Arc::new(Calculate));
    registry.register(Arc::new(ListStatistics));
    registry.register(Arc::new(RandomNumber));
    registry.register(Arc::new(RoundNumber));
}
