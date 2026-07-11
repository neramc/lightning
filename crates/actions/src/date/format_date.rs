// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Format Date — strftime-style formatting; invalid patterns are rejected
//! up front instead of panicking mid-run.

use chrono::format::{Item, StrftimeItems};
use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `date.format`
pub struct FormatDate;

#[async_trait::async_trait]
impl Action for FormatDate {
    fn def(&self) -> ActionDef {
        ActionDef::pure(
            "date.format",
            Category::Date,
            "calendar-text",
            ContentKind::Text,
        )
        .with_param(ParamDef::required("format", ParamKind::Text))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let format = ctx.param_text("format")?;
        if StrftimeItems::new(&format).any(|item| matches!(item, Item::Error)) {
            return Err(ActionError::InvalidParam {
                param: "format".into(),
                message: "invalid date format pattern".into(),
            });
        }
        let Content::Date(date) = input.coerce_to(ContentKind::Date)? else {
            return Err(ActionError::Failed("input did not coerce to a date".into()));
        };
        Ok(Content::Text(date.format(&format).to_string()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;
    use chrono::{TimeZone, Utc};

    #[tokio::test]
    async fn formats_with_a_pattern() {
        let date = Utc.with_ymd_and_hms(2026, 7, 11, 9, 5, 0).unwrap();
        let mut ctx = test_util::ctx_with(&[("format", Content::Text("%Y-%m-%d %H:%M".into()))]);
        let out = FormatDate
            .execute(&mut ctx, Content::Date(date))
            .await
            .unwrap();
        assert_eq!(out, Content::Text("2026-07-11 09:05".into()));
    }

    #[tokio::test]
    async fn invalid_pattern_is_rejected() {
        let mut ctx = test_util::ctx_with(&[("format", Content::Text("%Q!".into()))]);
        assert!(matches!(
            FormatDate
                .execute(&mut ctx, Content::Date(Utc::now()))
                .await,
            Err(ActionError::InvalidParam { .. })
        ));
    }
}
