// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! List Statistics ◆ — min / max / average / sum / median / count over a
//! list of numbers.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `math.list_statistics`
pub struct ListStatistics;

#[async_trait::async_trait]
impl Action for ListStatistics {
    fn def(&self) -> ActionDef {
        ActionDef::pure("math.list_statistics", Category::Math, "sigma", ContentKind::Number)
            .with_param(ParamDef::required(
                "operation",
                ParamKind::Enum(&["minimum", "maximum", "average", "sum", "median", "count"]),
            ))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let operation = ctx.param_text("operation")?;
        let mut numbers = Vec::new();
        for item in input.into_items() {
            numbers.push(item.as_number()?);
        }
        if numbers.is_empty() && operation != "count" && operation != "sum" {
            return Err(ActionError::Failed("the list is empty".into()));
        }
        let result = match operation.as_str() {
            "minimum" => numbers.iter().copied().fold(f64::INFINITY, f64::min),
            "maximum" => numbers.iter().copied().fold(f64::NEG_INFINITY, f64::max),
            "sum" => numbers.iter().sum(),
            "average" => numbers.iter().sum::<f64>() / numbers.len() as f64,
            "count" => numbers.len() as f64,
            "median" => {
                let mut sorted = numbers.clone();
                sorted.sort_by(f64::total_cmp);
                let mid = sorted.len() / 2;
                if sorted.len() % 2 == 0 {
                    (sorted[mid - 1] + sorted[mid]) / 2.0
                } else {
                    sorted[mid]
                }
            }
            other => {
                return Err(ActionError::InvalidParam {
                    param: "operation".into(),
                    message: format!("unknown operation '{other}'"),
                });
            }
        };
        Ok(Content::Number(result))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    fn list(values: &[f64]) -> Content {
        Content::List(values.iter().map(|n| Content::Number(*n)).collect())
    }

    async fn run(op: &str, values: &[f64]) -> f64 {
        let mut ctx = test_util::ctx_with(&[("operation", Content::Text(op.into()))]);
        ListStatistics.execute(&mut ctx, list(values)).await.unwrap().as_number().unwrap()
    }

    #[tokio::test]
    async fn statistics() {
        let values = [4.0, 1.0, 3.0, 2.0];
        assert_eq!(run("minimum", &values).await, 1.0);
        assert_eq!(run("maximum", &values).await, 4.0);
        assert_eq!(run("sum", &values).await, 10.0);
        assert_eq!(run("average", &values).await, 2.5);
        assert_eq!(run("median", &values).await, 2.5);
        assert_eq!(run("count", &values).await, 4.0);
        assert_eq!(run("median", &[3.0, 1.0, 2.0]).await, 2.0);
    }

    #[tokio::test]
    async fn empty_list_min_fails() {
        let mut ctx = test_util::ctx_with(&[("operation", Content::Text("minimum".into()))]);
        assert!(ListStatistics.execute(&mut ctx, list(&[])).await.is_err());
    }
}
