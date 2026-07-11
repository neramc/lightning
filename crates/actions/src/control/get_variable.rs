// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Get Variable — reads a named variable into the flow.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `control.get_variable`
pub struct GetVariable;

#[async_trait::async_trait]
impl Action for GetVariable {
    fn def(&self) -> ActionDef {
        ActionDef::pure(
            "control.get_variable",
            Category::ControlFlow,
            "bookmark-open",
            ContentKind::Nothing,
        )
        .with_param(ParamDef::required("name", ParamKind::Text))
    }

    async fn execute(&self, ctx: &mut RunContext, _input: Content) -> Result<Content, ActionError> {
        let name = ctx.param_text("name")?;
        ctx.var(&name)
            .cloned()
            .ok_or(ActionError::UnknownVariable(name))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn reads_a_variable_or_errors() {
        let mut ctx = test_util::ctx_with(&[("name", Content::Text("x".into()))]);
        ctx.set_var("x", Content::Boolean(true));
        assert_eq!(
            GetVariable
                .execute(&mut ctx, Content::Nothing)
                .await
                .unwrap(),
            Content::Boolean(true)
        );

        let mut ctx = test_util::ctx_with(&[("name", Content::Text("missing".into()))]);
        assert!(matches!(
            GetVariable.execute(&mut ctx, Content::Nothing).await,
            Err(ActionError::UnknownVariable(_))
        ));
    }
}
