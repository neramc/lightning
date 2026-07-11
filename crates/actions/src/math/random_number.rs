// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Random Number — uniform in [min, max].

use lightning_core::{ActionError, Content, ContentKind, RunContext};
use rand::Rng;

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `math.random`
pub struct RandomNumber;

#[async_trait::async_trait]
impl Action for RandomNumber {
    fn def(&self) -> ActionDef {
        ActionDef::pure("math.random", Category::Math, "dice", ContentKind::Number)
            .with_param(ParamDef::required("min", ParamKind::Number))
            .with_param(ParamDef::required("max", ParamKind::Number))
            .with_param(ParamDef::optional("integer", ParamKind::Boolean))
    }

    async fn execute(
        &self,
        ctx: &mut RunContext,
        _input: Content,
    ) -> Result<Content, ActionError> {
        let min = ctx.param_number("min")?;
        let max = ctx.param_number("max")?;
        let integer = ctx.param_bool_or("integer", true)?;
        if min > max {
            return Err(ActionError::InvalidParam {
                param: "min".into(),
                message: "min must not exceed max".into(),
            });
        }
        let value = if integer {
            rand::rng().random_range(min.ceil() as i64..=max.floor() as i64) as f64
        } else {
            rand::rng().random_range(min..=max)
        };
        Ok(Content::Number(value))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn stays_in_range_and_is_integer_by_default() {
        for _ in 0..50 {
            let mut ctx = test_util::ctx_with(&[
                ("min", Content::Number(1.0)),
                ("max", Content::Number(6.0)),
            ]);
            let out = RandomNumber.execute(&mut ctx, Content::Nothing).await.unwrap();
            let n = out.as_number().unwrap();
            assert!((1.0..=6.0).contains(&n));
            assert_eq!(n.fract(), 0.0);
        }
    }

    #[tokio::test]
    async fn inverted_range_is_rejected() {
        let mut ctx = test_util::ctx_with(&[
            ("min", Content::Number(9.0)),
            ("max", Content::Number(1.0)),
        ]);
        assert!(RandomNumber.execute(&mut ctx, Content::Nothing).await.is_err());
    }
}
