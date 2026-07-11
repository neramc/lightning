// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Wait — pauses the flow, cancellation-aware (never blocks the runtime).

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `control.wait`
pub struct Wait;

#[async_trait::async_trait]
impl Action for Wait {
    fn def(&self) -> ActionDef {
        ActionDef::pure(
            "control.wait",
            Category::ControlFlow,
            "hourglass",
            ContentKind::Nothing,
        )
        .with_param(ParamDef::required("seconds", ParamKind::Number))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let seconds = ctx.param_number("seconds")?;
        if !(0.0..=86_400.0).contains(&seconds) {
            return Err(ActionError::InvalidParam {
                param: "seconds".into(),
                message: "must be between 0 and 86400".into(),
            });
        }
        let cancel = ctx.cancellation();
        tokio::select! {
            () = tokio::time::sleep(std::time::Duration::from_secs_f64(seconds)) => Ok(input),
            () = cancel.cancelled() => Err(ActionError::Cancelled),
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn waits_and_passes_input_through() {
        let mut ctx = test_util::ctx_with(&[("seconds", Content::Number(0.01))]);
        let out = Wait
            .execute(&mut ctx, Content::Text("x".into()))
            .await
            .unwrap();
        assert_eq!(out, Content::Text("x".into()));
    }

    #[tokio::test]
    async fn rejects_negative_durations() {
        let mut ctx = test_util::ctx_with(&[("seconds", Content::Number(-1.0))]);
        assert!(matches!(
            Wait.execute(&mut ctx, Content::Nothing).await,
            Err(ActionError::InvalidParam { .. })
        ));
    }
}
