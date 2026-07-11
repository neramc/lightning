// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! URL Encode / Decode.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `text.url_encode`
pub struct UrlEncode;

#[async_trait::async_trait]
impl Action for UrlEncode {
    fn def(&self) -> ActionDef {
        ActionDef::pure("text.url_encode", Category::Text, "link", ContentKind::Text)
            .with_param(ParamDef::required("mode", ParamKind::Enum(&["encode", "decode"])))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let mode = ctx.param_text("mode")?;
        let text = input.as_text()?;
        let out = match mode.as_str() {
            "encode" => urlencoding::encode(&text).into_owned(),
            "decode" => urlencoding::decode(&text)
                .map_err(|err| ActionError::Failed(format!("invalid percent-encoding: {err}")))?
                .into_owned(),
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
        let original = "a b/î?";
        let mut ctx = test_util::ctx_with(&[("mode", Content::Text("encode".into()))]);
        let encoded = UrlEncode
            .execute(&mut ctx, Content::Text(original.into()))
            .await
            .unwrap()
            .as_text()
            .unwrap();
        assert!(!encoded.contains(' '));
        let mut ctx = test_util::ctx_with(&[("mode", Content::Text("decode".into()))]);
        let decoded = UrlEncode.execute(&mut ctx, Content::Text(encoded)).await.unwrap();
        assert_eq!(decoded, Content::Text(original.into()));
    }
}
