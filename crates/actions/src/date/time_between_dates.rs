// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Time Between Dates — absolute difference in the chosen unit.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `date.time_between`
pub struct TimeBetweenDates;

#[async_trait::async_trait]
impl Action for TimeBetweenDates {
    fn def(&self) -> ActionDef {
        ActionDef::pure(
            "date.time_between",
            Category::Date,
            "calendar-clock",
            ContentKind::Number,
        )
        .with_param(ParamDef::required("other", ParamKind::Date))
        .with_param(ParamDef::optional(
            "unit",
            ParamKind::Enum(&["seconds", "minutes", "hours", "days"]),
        ))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let Content::Date(other) = ctx.required_param("other")?.coerce_to(ContentKind::Date)?
        else {
            return Err(ActionError::InvalidParam {
                param: "other".into(),
                message: "did not coerce to a date".into(),
            });
        };
        let unit = ctx.param_text_or("unit", "seconds")?;
        let Content::Date(date) = input.coerce_to(ContentKind::Date)? else {
            return Err(ActionError::Failed("input did not coerce to a date".into()));
        };
        let seconds = (date - other).num_milliseconds().abs() as f64 / 1000.0;
        let value = match unit.as_str() {
            "seconds" => seconds,
            "minutes" => seconds / 60.0,
            "hours" => seconds / 3600.0,
            "days" => seconds / 86_400.0,
            other => {
                return Err(ActionError::InvalidParam {
                    param: "unit".into(),
                    message: format!("unknown unit '{other}'"),
                });
            }
        };
        Ok(Content::Number(value))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;
    use chrono::{TimeZone, Utc};

    #[tokio::test]
    async fn difference_is_absolute_and_unit_aware() {
        let a = Utc.with_ymd_and_hms(2026, 7, 11, 0, 0, 0).unwrap();
        let b = Utc.with_ymd_and_hms(2026, 7, 12, 6, 0, 0).unwrap();
        let mut ctx = test_util::ctx_with(&[
            ("other", Content::Date(b)),
            ("unit", Content::Text("hours".into())),
        ]);
        let out = TimeBetweenDates
            .execute(&mut ctx, Content::Date(a))
            .await
            .unwrap();
        assert_eq!(out, Content::Number(30.0));
    }
}
