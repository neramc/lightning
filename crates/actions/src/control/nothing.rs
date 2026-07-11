// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Nothing — clears the flow's current value.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category};

/// `control.nothing`
pub struct Nothing;

#[async_trait::async_trait]
impl Action for Nothing {
    fn def(&self) -> ActionDef {
        ActionDef::pure("control.nothing", Category::ControlFlow, "circle-dashed", ContentKind::Nothing)
    }

    async fn execute(
        &self,
        _ctx: &mut RunContext,
        _input: Content,
    ) -> Result<Content, ActionError> {
        Ok(Content::Nothing)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn outputs_nothing() {
        let mut ctx = test_util::ctx();
        let out = Nothing.execute(&mut ctx, Content::Text("x".into())).await.unwrap();
        assert_eq!(out, Content::Nothing);
    }
}
