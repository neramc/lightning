// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Generate UUID — a fresh v4 UUID as text.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category};

/// `text.generate_uuid`
pub struct GenerateUuid;

#[async_trait::async_trait]
impl Action for GenerateUuid {
    fn def(&self) -> ActionDef {
        ActionDef::pure(
            "text.generate_uuid",
            Category::Text,
            "fingerprint",
            ContentKind::Text,
        )
    }

    async fn execute(
        &self,
        _ctx: &mut RunContext,
        _input: Content,
    ) -> Result<Content, ActionError> {
        Ok(Content::Text(uuid::Uuid::new_v4().to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn output_parses_as_uuid() {
        let mut ctx = test_util::ctx();
        let out = GenerateUuid
            .execute(&mut ctx, Content::Nothing)
            .await
            .unwrap();
        let text = out.as_text().unwrap();
        assert!(uuid::Uuid::parse_str(&text).is_ok());
    }
}
