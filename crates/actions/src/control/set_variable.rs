// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Set Variable — stores the input under a name; passes it through.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `control.set_variable`
pub struct SetVariable;

#[async_trait::async_trait]
impl Action for SetVariable {
    fn def(&self) -> ActionDef {
        ActionDef::pure("control.set_variable", Category::ControlFlow, "bookmark", ContentKind::Nothing)
            .with_param(ParamDef::required("name", ParamKind::Text))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let name = ctx.param_text("name")?;
        if name.trim().is_empty() {
            return Err(ActionError::InvalidParam {
                param: "name".into(),
                message: "variable name must not be empty".into(),
            });
        }
        ctx.set_var(name, input.clone());
        Ok(input)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn stores_and_passes_through() {
        let mut ctx = test_util::ctx_with(&[("name", Content::Text("x".into()))]);
        let out = SetVariable.execute(&mut ctx, Content::Number(7.0)).await.unwrap();
        assert_eq!(out, Content::Number(7.0));
        assert_eq!(ctx.var("x"), Some(&Content::Number(7.0)));
    }
}
