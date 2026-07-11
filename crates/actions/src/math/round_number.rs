// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Round Number — nearest / up / down, to N decimals.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `math.round`
pub struct RoundNumber;

#[async_trait::async_trait]
impl Action for RoundNumber {
    fn def(&self) -> ActionDef {
        ActionDef::pure(
            "math.round",
            Category::Math,
            "circle-half",
            ContentKind::Number,
        )
        .with_param(ParamDef::optional(
            "mode",
            ParamKind::Enum(&["nearest", "up", "down"]),
        ))
        .with_param(ParamDef::optional("decimals", ParamKind::Number))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let mode = ctx.param_text_or("mode", "nearest")?;
        let decimals = match ctx.param("decimals") {
            Some(value) => value.as_number()?,
            None => 0.0,
        };
        if !(0.0..=12.0).contains(&decimals) || decimals.fract() != 0.0 {
            return Err(ActionError::InvalidParam {
                param: "decimals".into(),
                message: "must be a whole number between 0 and 12".into(),
            });
        }
        let factor = 10f64.powi(decimals as i32);
        let value = input.as_number()? * factor;
        let rounded = match mode.as_str() {
            "nearest" => value.round(),
            "up" => value.ceil(),
            "down" => value.floor(),
            other => {
                return Err(ActionError::InvalidParam {
                    param: "mode".into(),
                    message: format!("unknown mode '{other}'"),
                });
            }
        };
        Ok(Content::Number(rounded / factor))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn modes_and_decimals() {
        let mut ctx = test_util::ctx();
        assert_eq!(
            RoundNumber
                .execute(&mut ctx, Content::Number(2.5))
                .await
                .unwrap(),
            Content::Number(3.0)
        );
        let mut ctx = test_util::ctx_with(&[
            ("mode", Content::Text("down".into())),
            ("decimals", Content::Number(1.0)),
        ]);
        assert_eq!(
            RoundNumber
                .execute(&mut ctx, Content::Number(2.79))
                .await
                .unwrap(),
            Content::Number(2.7)
        );
    }
}
