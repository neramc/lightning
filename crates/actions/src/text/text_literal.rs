// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Text — a literal (usually templated) text value.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `text.text`
pub struct TextLiteral;

#[async_trait::async_trait]
impl Action for TextLiteral {
    fn def(&self) -> ActionDef {
        ActionDef::pure("text.text", Category::Text, "text", ContentKind::Text)
            .with_param(ParamDef::required("text", ParamKind::Text))
    }

    async fn execute(
        &self,
        ctx: &mut RunContext,
        _input: Content,
    ) -> Result<Content, ActionError> {
        Ok(Content::Text(ctx.param_text("text")?))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn emits_the_param() {
        let mut ctx = test_util::ctx_with(&[("text", Content::Text("hi".into()))]);
        assert_eq!(
            TextLiteral.execute(&mut ctx, Content::Nothing).await.unwrap(),
            Content::Text("hi".into())
        );
    }
}
