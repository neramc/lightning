// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Calculate — input `<op>` operand.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `math.calculate`
pub struct Calculate;

#[async_trait::async_trait]
impl Action for Calculate {
    fn def(&self) -> ActionDef {
        ActionDef::pure("math.calculate", Category::Math, "calculator", ContentKind::Number)
            .with_param(ParamDef::required(
                "operation",
                ParamKind::Enum(&["add", "subtract", "multiply", "divide", "modulo", "power"]),
            ))
            .with_param(ParamDef::required("operand", ParamKind::Number))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let operation = ctx.param_text("operation")?;
        let operand = ctx.param_number("operand")?;
        let value = input.as_number()?;
        let result = match operation.as_str() {
            "add" => value + operand,
            "subtract" => value - operand,
            "multiply" => value * operand,
            "divide" => {
                if operand == 0.0 {
                    return Err(ActionError::Failed("division by zero".into()));
                }
                value / operand
            }
            "modulo" => {
                if operand == 0.0 {
                    return Err(ActionError::Failed("modulo by zero".into()));
                }
                value % operand
            }
            "power" => value.powf(operand),
            other => {
                return Err(ActionError::InvalidParam {
                    param: "operation".into(),
                    message: format!("unknown operation '{other}'"),
                });
            }
        };
        if !result.is_finite() {
            return Err(ActionError::Failed("result is not a finite number".into()));
        }
        Ok(Content::Number(result))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    async fn run(op: &str, input: f64, operand: f64) -> Result<Content, ActionError> {
        let mut ctx = test_util::ctx_with(&[
            ("operation", Content::Text(op.into())),
            ("operand", Content::Number(operand)),
        ]);
        Calculate.execute(&mut ctx, Content::Number(input)).await
    }

    #[tokio::test]
    async fn arithmetic() {
        assert_eq!(run("add", 2.0, 3.0).await.unwrap(), Content::Number(5.0));
        assert_eq!(run("divide", 9.0, 3.0).await.unwrap(), Content::Number(3.0));
        assert_eq!(run("power", 2.0, 10.0).await.unwrap(), Content::Number(1024.0));
        assert_eq!(run("modulo", 7.0, 3.0).await.unwrap(), Content::Number(1.0));
    }

    #[tokio::test]
    async fn division_by_zero_fails_cleanly() {
        assert!(matches!(run("divide", 1.0, 0.0).await, Err(ActionError::Failed(_))));
    }
}
