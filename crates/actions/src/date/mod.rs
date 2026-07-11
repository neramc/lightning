// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Date & time (§8.3 D). Engine-side formatting uses chrono strftime
//! patterns; user-locale rendering (ICU) happens in the UI layer.

mod adjust_date;
mod current_date;
mod format_date;
mod time_between_dates;

pub use adjust_date::AdjustDate;
pub use current_date::CurrentDate;
pub use format_date::FormatDate;
pub use time_between_dates::TimeBetweenDates;

use std::sync::Arc;

use crate::Registry;

/// Register every action in this category.
pub fn register(registry: &mut Registry) {
    registry.register(Arc::new(AdjustDate));
    registry.register(Arc::new(CurrentDate));
    registry.register(Arc::new(FormatDate));
    registry.register(Arc::new(TimeBetweenDates));
}
