// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Text actions (§8.3 B) — pure and fully cross-platform.

mod base64_text;
mod change_case;
mod combine_text;
mod generate_uuid;
mod hash_text;
mod replace_text;
mod split_text;
mod text_literal;
mod text_statistics;
mod trim_text;
mod url_encode;

pub use base64_text::Base64Text;
pub use change_case::ChangeCase;
pub use combine_text::CombineText;
pub use generate_uuid::GenerateUuid;
pub use hash_text::HashText;
pub use replace_text::ReplaceText;
pub use split_text::SplitText;
pub use text_literal::TextLiteral;
pub use text_statistics::TextStatistics;
pub use trim_text::TrimText;
pub use url_encode::UrlEncode;

use std::sync::Arc;

use crate::Registry;

/// Register every action in this category.
pub fn register(registry: &mut Registry) {
    registry.register(Arc::new(Base64Text));
    registry.register(Arc::new(ChangeCase));
    registry.register(Arc::new(CombineText));
    registry.register(Arc::new(GenerateUuid));
    registry.register(Arc::new(HashText));
    registry.register(Arc::new(ReplaceText));
    registry.register(Arc::new(SplitText));
    registry.register(Arc::new(TextLiteral));
    registry.register(Arc::new(TextStatistics));
    registry.register(Arc::new(TrimText));
    registry.register(Arc::new(UrlEncode));
}
