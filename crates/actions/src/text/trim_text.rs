// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Trim — strips whitespace from both ends, the start, or the end.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `text.trim`
pub struct TrimText;

#[async_trait::async_trait]
impl Action for TrimText {
    fn def(&self) -> ActionDef {
        ActionDef::pure("text.trim", Category::Text, "scissors", ContentKind::Text).with_param(
            ParamDef::optional("mode", ParamKind::Enum(&["both", "start", "end"])),
        )
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let mode = ctx.param_text_or("mode", "both")?;
        let text = input.as_text()?;
        let out = match mode.as_str() {
            "both" => text.trim(),
            "start" => text.trim_start(),
            "end" => text.trim_end(),
            other => {
                return Err(ActionError::InvalidParam {
                    param: "mode".into(),
                    message: format!("unknown mode '{other}'"),
                });
            }
        };
        Ok(Content::Text(out.to_owned()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn trims_both_ends_by_default() {
        let mut ctx = test_util::ctx();
        let out = TrimText
            .execute(&mut ctx, Content::Text("  x  ".into()))
            .await
            .unwrap();
        assert_eq!(out, Content::Text("x".into()));
    }
}
