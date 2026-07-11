// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Split Text — splits into a list by a separator.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `text.split`
pub struct SplitText;

#[async_trait::async_trait]
impl Action for SplitText {
    fn def(&self) -> ActionDef {
        ActionDef::pure(
            "text.split",
            Category::Text,
            "rows-split",
            ContentKind::List,
        )
        .with_param(ParamDef::optional("separator", ParamKind::Text))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let separator = ctx.param_text_or("separator", "\n")?;
        if separator.is_empty() {
            return Err(ActionError::InvalidParam {
                param: "separator".into(),
                message: "separator must not be empty".into(),
            });
        }
        let text = input.as_text()?;
        Ok(Content::List(
            text.split(&separator)
                .map(|part| Content::Text(part.to_owned()))
                .collect(),
        ))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn splits_on_newline_by_default() {
        let mut ctx = test_util::ctx();
        let out = SplitText
            .execute(&mut ctx, Content::Text("a\nb".into()))
            .await
            .unwrap();
        assert_eq!(
            out,
            Content::List(vec![Content::Text("a".into()), Content::Text("b".into())])
        );
    }
}
