// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Hash Text — MD5 / SHA-1 / SHA-256 hex digests.

use lightning_core::{ActionError, Content, ContentKind, RunContext};
use md5::Digest as _;

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `text.hash`
pub struct HashText;

#[async_trait::async_trait]
impl Action for HashText {
    fn def(&self) -> ActionDef {
        ActionDef::pure("text.hash", Category::Text, "hash", ContentKind::Text).with_param(
            ParamDef::required("algorithm", ParamKind::Enum(&["md5", "sha1", "sha256"])),
        )
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let algorithm = ctx.param_text("algorithm")?;
        let text = input.as_text()?;
        let digest = match algorithm.as_str() {
            "md5" => hex::encode(md5::Md5::digest(text.as_bytes())),
            "sha1" => hex::encode(sha1::Sha1::digest(text.as_bytes())),
            "sha256" => hex::encode(sha2::Sha256::digest(text.as_bytes())),
            other => {
                return Err(ActionError::InvalidParam {
                    param: "algorithm".into(),
                    message: format!("unknown algorithm '{other}'"),
                });
            }
        };
        Ok(Content::Text(digest))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn sha256_matches_known_vector() {
        let mut ctx = test_util::ctx_with(&[("algorithm", Content::Text("sha256".into()))]);
        let out = HashText
            .execute(&mut ctx, Content::Text("abc".into()))
            .await
            .unwrap();
        assert_eq!(
            out,
            Content::Text(
                "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad".into()
            )
        );
    }

    #[tokio::test]
    async fn md5_matches_known_vector() {
        let mut ctx = test_util::ctx_with(&[("algorithm", Content::Text("md5".into()))]);
        let out = HashText
            .execute(&mut ctx, Content::Text("abc".into()))
            .await
            .unwrap();
        assert_eq!(
            out,
            Content::Text("900150983cd24fb0d6963f7d28e17f72".into())
        );
    }
}
