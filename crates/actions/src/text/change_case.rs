// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Change Case — lowercase, UPPERCASE, Capitalize Every Word, Sentence case.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `text.change_case`
pub struct ChangeCase;

fn capitalize_word(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

#[async_trait::async_trait]
impl Action for ChangeCase {
    fn def(&self) -> ActionDef {
        ActionDef::pure("text.change_case", Category::Text, "text-case", ContentKind::Text)
            .with_param(ParamDef::required(
                "case",
                ParamKind::Enum(&["lowercase", "uppercase", "capitalize", "sentence"]),
            ))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let mode = ctx.param_text("case")?;
        let text = input.as_text()?;
        let out = match mode.as_str() {
            "lowercase" => text.to_lowercase(),
            "uppercase" => text.to_uppercase(),
            "capitalize" => text
                .split_inclusive(char::is_whitespace)
                .map(capitalize_word)
                .collect::<String>(),
            "sentence" => {
                let lower = text.to_lowercase();
                capitalize_word(&lower)
            }
            other => {
                return Err(ActionError::InvalidParam {
                    param: "case".into(),
                    message: format!("unknown case '{other}'"),
                });
            }
        };
        Ok(Content::Text(out))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    async fn run(mode: &str, text: &str) -> String {
        let mut ctx = test_util::ctx_with(&[("case", Content::Text(mode.into()))]);
        ChangeCase
            .execute(&mut ctx, Content::Text(text.into()))
            .await
            .unwrap()
            .as_text()
            .unwrap()
    }

    #[tokio::test]
    async fn all_modes() {
        assert_eq!(run("lowercase", "Hello World").await, "hello world");
        assert_eq!(run("uppercase", "Hello World").await, "HELLO WORLD");
        assert_eq!(run("capitalize", "hello brave world").await, "Hello Brave World");
        assert_eq!(run("sentence", "HELLO WORLD").await, "Hello world");
    }
}
