// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Adjust Date — add/subtract seconds…weeks from a date.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `date.adjust`
pub struct AdjustDate;

#[async_trait::async_trait]
impl Action for AdjustDate {
    fn def(&self) -> ActionDef {
        ActionDef::pure("date.adjust", Category::Date, "calendar-plus", ContentKind::Date)
            .with_param(ParamDef::required("amount", ParamKind::Number))
            .with_param(ParamDef::required(
                "unit",
                ParamKind::Enum(&["seconds", "minutes", "hours", "days", "weeks"]),
            ))
            .with_param(ParamDef::optional("direction", ParamKind::Enum(&["add", "subtract"])))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let amount = ctx.param_number("amount")?;
        let unit = ctx.param_text("unit")?;
        let direction = ctx.param_text_or("direction", "add")?;
        let Content::Date(date) = input.coerce_to(ContentKind::Date)? else {
            return Err(ActionError::Failed("input did not coerce to a date".into()));
        };

        let seconds_per_unit: f64 = match unit.as_str() {
            "seconds" => 1.0,
            "minutes" => 60.0,
            "hours" => 3600.0,
            "days" => 86_400.0,
            "weeks" => 604_800.0,
            other => {
                return Err(ActionError::InvalidParam {
                    param: "unit".into(),
                    message: format!("unknown unit '{other}'"),
                });
            }
        };
        let signed = match direction.as_str() {
            "add" => amount,
            "subtract" => -amount,
            other => {
                return Err(ActionError::InvalidParam {
                    param: "direction".into(),
                    message: format!("unknown direction '{other}'"),
                });
            }
        };
        let delta = chrono::Duration::milliseconds((signed * seconds_per_unit * 1000.0) as i64);
        date.checked_add_signed(delta)
            .map(Content::Date)
            .ok_or_else(|| ActionError::Failed("adjusted date is out of range".into()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;
    use chrono::{TimeZone, Utc};

    #[tokio::test]
    async fn adds_and_subtracts() {
        let base = Utc.with_ymd_and_hms(2026, 7, 11, 12, 0, 0).unwrap();
        let mut ctx = test_util::ctx_with(&[
            ("amount", Content::Number(90.0)),
            ("unit", Content::Text("minutes".into())),
        ]);
        let out = AdjustDate.execute(&mut ctx, Content::Date(base)).await.unwrap();
        assert_eq!(out, Content::Date(base + chrono::Duration::minutes(90)));

        let mut ctx = test_util::ctx_with(&[
            ("amount", Content::Number(1.0)),
            ("unit", Content::Text("weeks".into())),
            ("direction", Content::Text("subtract".into())),
        ]);
        let out = AdjustDate.execute(&mut ctx, Content::Date(base)).await.unwrap();
        assert_eq!(out, Content::Date(base - chrono::Duration::weeks(1)));
    }
}
