// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Get Type — reports the content kind of the input.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category};

/// `control.get_type`
pub struct GetType;

#[async_trait::async_trait]
impl Action for GetType {
    fn def(&self) -> ActionDef {
        ActionDef::pure(
            "control.get_type",
            Category::ControlFlow,
            "tag",
            ContentKind::Text,
        )
    }

    async fn execute(&self, _ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        Ok(Content::Text(input.kind().to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn names_the_kind() {
        let mut ctx = test_util::ctx();
        let out = GetType
            .execute(&mut ctx, Content::Number(1.0))
            .await
            .unwrap();
        assert_eq!(out, Content::Text("Number".into()));
    }
}
