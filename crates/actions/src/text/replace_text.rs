// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Replace Text — literal or regex replacement, optional case-insensitivity.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `text.replace`
pub struct ReplaceText;

#[async_trait::async_trait]
impl Action for ReplaceText {
    fn def(&self) -> ActionDef {
        ActionDef::pure("text.replace", Category::Text, "find-replace", ContentKind::Text)
            .with_param(ParamDef::required("find", ParamKind::Text))
            .with_param(ParamDef::optional("replace", ParamKind::Text))
            .with_param(ParamDef::optional("regex", ParamKind::Boolean))
            .with_param(ParamDef::optional("caseSensitive", ParamKind::Boolean))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let find = ctx.param_text("find")?;
        let replacement = ctx.param_text_or("replace", "")?;
        let use_regex = ctx.param_bool_or("regex", false)?;
        let case_sensitive = ctx.param_bool_or("caseSensitive", true)?;
        let text = input.as_text()?;

        let pattern = if use_regex { find.clone() } else { regex::escape(&find) };
        let pattern = if case_sensitive { pattern } else { format!("(?i){pattern}") };
        let re = regex::Regex::new(&pattern).map_err(|err| ActionError::InvalidParam {
            param: "find".into(),
            message: format!("invalid pattern: {err}"),
        })?;
        // In literal mode the replacement is literal too — escape `$`.
        let replacement =
            if use_regex { replacement } else { replacement.replace('$', "$$") };
        Ok(Content::Text(re.replace_all(&text, replacement.as_str()).into_owned()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn literal_replacement_is_default() {
        let mut ctx = test_util::ctx_with(&[
            ("find", Content::Text("a.c".into())),
            ("replace", Content::Text("X".into())),
        ]);
        // Literal mode: the dot must not act as a regex wildcard.
        let out = ReplaceText.execute(&mut ctx, Content::Text("abc a.c".into())).await.unwrap();
        assert_eq!(out, Content::Text("abc X".into()));
    }

    #[tokio::test]
    async fn regex_mode_with_groups() {
        let mut ctx = test_util::ctx_with(&[
            ("find", Content::Text(r"(\d+)".into())),
            ("replace", Content::Text("[$1]".into())),
            ("regex", Content::Boolean(true)),
        ]);
        let out = ReplaceText.execute(&mut ctx, Content::Text("a 42 b".into())).await.unwrap();
        assert_eq!(out, Content::Text("a [42] b".into()));
    }

    #[tokio::test]
    async fn case_insensitive_literal() {
        let mut ctx = test_util::ctx_with(&[
            ("find", Content::Text("hello".into())),
            ("replace", Content::Text("bye".into())),
            ("caseSensitive", Content::Boolean(false)),
        ]);
        let out = ReplaceText.execute(&mut ctx, Content::Text("HELLO world".into())).await.unwrap();
        assert_eq!(out, Content::Text("bye world".into()));
    }

    #[tokio::test]
    async fn invalid_regex_is_an_invalid_param() {
        let mut ctx = test_util::ctx_with(&[
            ("find", Content::Text("(".into())),
            ("regex", Content::Boolean(true)),
        ]);
        assert!(matches!(
            ReplaceText.execute(&mut ctx, Content::Text("x".into())).await,
            Err(ActionError::InvalidParam { .. })
        ));
    }
}
