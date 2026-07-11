// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Combine Text — joins list items with a separator.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `text.combine`
pub struct CombineText;

#[async_trait::async_trait]
impl Action for CombineText {
    fn def(&self) -> ActionDef {
        ActionDef::pure(
            "text.combine",
            Category::Text,
            "rows-merge",
            ContentKind::Text,
        )
        .with_param(ParamDef::optional("separator", ParamKind::Text))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let separator = ctx.param_text_or("separator", "\n")?;
        let mut parts = Vec::new();
        for item in input.into_items() {
            parts.push(item.as_text()?);
        }
        Ok(Content::Text(parts.join(&separator)))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn joins_with_custom_separator() {
        let mut ctx = test_util::ctx_with(&[("separator", Content::Text(", ".into()))]);
        let list = Content::List(vec![Content::Text("a".into()), Content::Number(2.0)]);
        assert_eq!(
            CombineText.execute(&mut ctx, list).await.unwrap(),
            Content::Text("a, 2".into())
        );
    }
}
