// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Count — items, characters, words, or lines of the input.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `control.count`
pub struct Count;

#[async_trait::async_trait]
impl Action for Count {
    fn def(&self) -> ActionDef {
        ActionDef::pure("control.count", Category::ControlFlow, "list-numbers", ContentKind::Number)
            .with_param(ParamDef::optional(
                "unit",
                ParamKind::Enum(&["items", "characters", "words", "lines"]),
            ))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let unit = ctx.param_text_or("unit", "items")?;
        let count = match unit.as_str() {
            "items" => input.into_items().len(),
            "characters" => input.as_text()?.chars().count(),
            "words" => input.as_text()?.split_whitespace().count(),
            "lines" => input.as_text()?.lines().count(),
            other => {
                return Err(ActionError::InvalidParam {
                    param: "unit".into(),
                    message: format!("unknown unit '{other}'"),
                });
            }
        };
        Ok(Content::Number(count as f64))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn counts_items_by_default() {
        let mut ctx = test_util::ctx();
        let list = Content::List(vec![Content::Number(1.0), Content::Number(2.0)]);
        assert_eq!(Count.execute(&mut ctx, list).await.unwrap(), Content::Number(2.0));
    }

    #[tokio::test]
    async fn counts_words_and_lines() {
        let text = Content::Text("one two\nthree".into());
        let mut ctx = test_util::ctx_with(&[("unit", Content::Text("words".into()))]);
        assert_eq!(Count.execute(&mut ctx, text.clone()).await.unwrap(), Content::Number(3.0));
        let mut ctx = test_util::ctx_with(&[("unit", Content::Text("lines".into()))]);
        assert_eq!(Count.execute(&mut ctx, text).await.unwrap(), Content::Number(2.0));
    }
}
