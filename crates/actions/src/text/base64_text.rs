// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Base64 Encode / Decode (text payloads).

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `text.base64`
pub struct Base64Text;

#[async_trait::async_trait]
impl Action for Base64Text {
    fn def(&self) -> ActionDef {
        ActionDef::pure("text.base64", Category::Text, "binary", ContentKind::Text).with_param(
            ParamDef::required("mode", ParamKind::Enum(&["encode", "decode"])),
        )
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let mode = ctx.param_text("mode")?;
        let text = input.as_text()?;
        let out = match mode.as_str() {
            "encode" => STANDARD.encode(text.as_bytes()),
            "decode" => {
                let bytes = STANDARD
                    .decode(text.trim())
                    .map_err(|err| ActionError::Failed(format!("invalid base64: {err}")))?;
                String::from_utf8(bytes)
                    .map_err(|_| ActionError::Failed("decoded bytes are not UTF-8 text".into()))?
            }
            other => {
                return Err(ActionError::InvalidParam {
                    param: "mode".into(),
                    message: format!("unknown mode '{other}'"),
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

    #[tokio::test]
    async fn round_trips() {
        let mut ctx = test_util::ctx_with(&[("mode", Content::Text("encode".into()))]);
        let encoded = Base64Text
            .execute(&mut ctx, Content::Text("빛 lightning".into()))
            .await
            .unwrap();
        let mut ctx = test_util::ctx_with(&[("mode", Content::Text("decode".into()))]);
        let decoded = Base64Text.execute(&mut ctx, encoded).await.unwrap();
        assert_eq!(decoded, Content::Text("빛 lightning".into()));
    }

    #[tokio::test]
    async fn invalid_base64_fails_cleanly() {
        let mut ctx = test_util::ctx_with(&[("mode", Content::Text("decode".into()))]);
        assert!(matches!(
            Base64Text
                .execute(&mut ctx, Content::Text("!!not base64!!".into()))
                .await,
            Err(ActionError::Failed(_))
        ));
    }
}
