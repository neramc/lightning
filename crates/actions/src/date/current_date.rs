// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Current Date — the moment of execution.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category};

/// `date.current`
pub struct CurrentDate;

#[async_trait::async_trait]
impl Action for CurrentDate {
    fn def(&self) -> ActionDef {
        ActionDef::pure(
            "date.current",
            Category::Date,
            "calendar",
            ContentKind::Date,
        )
    }

    async fn execute(
        &self,
        _ctx: &mut RunContext,
        _input: Content,
    ) -> Result<Content, ActionError> {
        Ok(Content::Date(chrono::Utc::now()))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn outputs_a_recent_date() {
        let mut ctx = test_util::ctx();
        let out = CurrentDate
            .execute(&mut ctx, Content::Nothing)
            .await
            .unwrap();
        let Content::Date(d) = out else {
            panic!("expected date")
        };
        assert!((chrono::Utc::now() - d).num_seconds().abs() < 5);
    }
}
