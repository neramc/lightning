// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Text Statistics — characters / words / lines as a dictionary.

use std::collections::BTreeMap;

use lightning_core::{ActionError, Content, ContentKind, RunContext};

use crate::{Action, ActionDef, Category};

/// `text.statistics`
pub struct TextStatistics;

#[async_trait::async_trait]
impl Action for TextStatistics {
    fn def(&self) -> ActionDef {
        ActionDef::pure("text.statistics", Category::Text, "chart-bar", ContentKind::Dictionary)
    }

    async fn execute(
        &self,
        _ctx: &mut RunContext,
        input: Content,
    ) -> Result<Content, ActionError> {
        let text = input.as_text()?;
        let mut stats = BTreeMap::new();
        stats.insert("characters".to_owned(), Content::Number(text.chars().count() as f64));
        stats.insert(
            "words".to_owned(),
            Content::Number(text.split_whitespace().count() as f64),
        );
        stats.insert("lines".to_owned(), Content::Number(text.lines().count() as f64));
        Ok(Content::Dictionary(stats))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::test_util;

    #[tokio::test]
    async fn counts_are_correct() {
        let mut ctx = test_util::ctx();
        let out = TextStatistics
            .execute(&mut ctx, Content::Text("one two\nthree".into()))
            .await
            .unwrap();
        let Content::Dictionary(stats) = out else { panic!("expected dictionary") };
        assert_eq!(stats["words"], Content::Number(3.0));
        assert_eq!(stats["lines"], Content::Number(2.0));
        assert_eq!(stats["characters"], Content::Number(13.0));
    }
}
