// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Control-flow & scripting basics (§8.3 A). The structural steps
//! (If/Repeat/While/Exit/Run Shortcut) live in the engine; the actions here
//! are the value-level ones.

mod add_to_variable;
mod comment;
mod count;
mod get_item_from_list;
mod get_type;
mod get_variable;
mod nothing;
mod set_variable;
mod show_result;
mod wait;

pub use add_to_variable::AddToVariable;
pub use comment::Comment;
pub use count::Count;
pub use get_item_from_list::GetItemFromList;
pub use get_type::GetType;
pub use get_variable::GetVariable;
pub use nothing::Nothing;
pub use set_variable::SetVariable;
pub use show_result::ShowResult;
pub use wait::Wait;

use std::sync::Arc;

use crate::Registry;

/// Register every action in this category.
pub fn register(registry: &mut Registry) {
    registry.register(Arc::new(AddToVariable));
    registry.register(Arc::new(Comment));
    registry.register(Arc::new(Count));
    registry.register(Arc::new(GetItemFromList));
    registry.register(Arc::new(GetType));
    registry.register(Arc::new(GetVariable));
    registry.register(Arc::new(Nothing));
    registry.register(Arc::new(SetVariable));
    registry.register(Arc::new(ShowResult));
    registry.register(Arc::new(Wait));
}
