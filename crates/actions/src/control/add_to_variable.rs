// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Add to Variable — appends the input to a named list variable, creating or
//! wrapping as needed.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `control.add_to_variable`
pub struct AddToVariable;

#[async_trait::async_trait]
impl Action for AddToVariable {
    fn def(&self) -> ActionDef {
        ActionDef::pure(
            "control.add_to_variable",
            Category::ControlFlow,
            "bookmark-plus",
            ContentKind::List,
        )
        .with_param(ParamDef::required("name", ParamKind::Text))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let name = ctx.param_text("name")?;
        let updated = match ctx.var(&name).cloned() {
            None | Some(Content::Nothing) => Content::List(vec![input]),
            Some(Content::List(mut items)) => {
                items.push(input);
                Content::List(items)
            }
            Some(existing) => Content::List(vec![existing, input]),
        };
        ctx.set_var(name, updated.clone());
        Ok(updated)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn creates_then_appends() {
        let mut ctx = test_util::ctx_with(&[("name", Content::Text("bag".into()))]);
        AddToVariable
            .execute(&mut ctx, Content::Number(1.0))
            .await
            .unwrap();
        let out = AddToVariable
            .execute(&mut ctx, Content::Number(2.0))
            .await
            .unwrap();
        assert_eq!(
            out,
            Content::List(vec![Content::Number(1.0), Content::Number(2.0)])
        );
    }

    #[tokio::test]
    async fn wraps_an_existing_scalar() {
        let mut ctx = test_util::ctx_with(&[("name", Content::Text("bag".into()))]);
        ctx.set_var("bag", Content::Text("first".into()));
        let out = AddToVariable
            .execute(&mut ctx, Content::Text("second".into()))
            .await
            .unwrap();
        assert_eq!(
            out,
            Content::List(vec![
                Content::Text("first".into()),
                Content::Text("second".into())
            ])
        );
    }
}
