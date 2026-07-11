// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Comment — annotates the flow; passes its input through untouched.

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category, ParamDef, ParamKind};

/// `control.comment`
pub struct Comment;

#[async_trait::async_trait]
impl Action for Comment {
    fn def(&self) -> ActionDef {
        ActionDef::pure("control.comment", Category::ControlFlow, "chat-bubble", ContentKind::Nothing)
            .with_param(ParamDef::optional("text", ParamKind::Text))
    }

    async fn execute(
        &self,
        _ctx: &mut RunContext,
        input: Content,
    ) -> Result<Content, ActionError> {
        Ok(input)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn passes_input_through() {
        let mut ctx = test_util::ctx();
        let input = Content::Text("keep me".into());
        let out = Comment.execute(&mut ctx, input.clone()).await.unwrap();
        assert_eq!(out, input);
    }
}
