// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Get Item from List — first / last / by index / random.

use lightning_core::{ActionError, Content, ContentKind, RunContext};
use rand::Rng;

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `control.get_item_from_list`
pub struct GetItemFromList;

#[async_trait::async_trait]
impl Action for GetItemFromList {
    fn def(&self) -> ActionDef {
        ActionDef::pure("control.get_item_from_list", Category::ControlFlow, "list-magnify", ContentKind::Nothing)
            .with_param(ParamDef::required(
                "which",
                ParamKind::Enum(&["first", "last", "index", "random"]),
            ))
            .with_param(ParamDef::optional("index", ParamKind::Number))
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let which = ctx.param_text("which")?;
        let items = input.into_items();
        if items.is_empty() {
            return Ok(Content::Nothing);
        }
        let item = match which.as_str() {
            "first" => items.first(),
            "last" => items.last(),
            "index" => {
                let index = ctx.param_number("index")?;
                if index < 1.0 || index.fract() != 0.0 {
                    return Err(ActionError::InvalidParam {
                        param: "index".into(),
                        message: "index is 1-based and must be a whole number".into(),
                    });
                }
                items.get(index as usize - 1)
            }
            "random" => {
                let i = rand::rng().random_range(0..items.len());
                items.get(i)
            }
            other => {
                return Err(ActionError::InvalidParam {
                    param: "which".into(),
                    message: format!("unknown selector '{other}'"),
                });
            }
        };
        Ok(item.cloned().unwrap_or(Content::Nothing))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    fn list() -> Content {
        Content::List(vec![
            Content::Text("a".into()),
            Content::Text("b".into()),
            Content::Text("c".into()),
        ])
    }

    #[tokio::test]
    async fn first_last_and_index() {
        let mut ctx = test_util::ctx_with(&[("which", Content::Text("first".into()))]);
        assert_eq!(
            GetItemFromList.execute(&mut ctx, list()).await.unwrap(),
            Content::Text("a".into())
        );
        let mut ctx = test_util::ctx_with(&[("which", Content::Text("last".into()))]);
        assert_eq!(
            GetItemFromList.execute(&mut ctx, list()).await.unwrap(),
            Content::Text("c".into())
        );
        let mut ctx = test_util::ctx_with(&[
            ("which", Content::Text("index".into())),
            ("index", Content::Number(2.0)),
        ]);
        assert_eq!(
            GetItemFromList.execute(&mut ctx, list()).await.unwrap(),
            Content::Text("b".into())
        );
    }

    #[tokio::test]
    async fn empty_list_yields_nothing() {
        let mut ctx = test_util::ctx_with(&[("which", Content::Text("first".into()))]);
        assert_eq!(
            GetItemFromList.execute(&mut ctx, Content::List(vec![])).await.unwrap(),
            Content::Nothing
        );
    }
}
