// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Show Result — surfaces a value in the run UI (via the run log and the
//! step's progress preview).

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `control.show_result`
pub struct ShowResult;

#[async_trait::async_trait]
impl Action for ShowResult {
    fn def(&self) -> ActionDef {
        ActionDef::pure("control.show_result", Category::ControlFlow, "eye", ContentKind::Text)
            .with_param(ParamDef::optional("text", ParamKind::Text))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let text = match ctx.param("text") {
            Some(value) => value.as_text()?,
            None => input.as_text()?,
        };
        ctx.log.info(None, text.clone());
        Ok(Content::Text(text))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn logs_and_returns_the_text() {
        let mut ctx = test_util::ctx();
        let out = ShowResult.execute(&mut ctx, Content::Number(3.0)).await.unwrap();
        assert_eq!(out, Content::Text("3".into()));
        assert_eq!(ctx.log.entries().len(), 1);
    }
}
