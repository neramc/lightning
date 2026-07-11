// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Run JavaScript ◆ — the cross-platform scripting action (§8.3 N).

use lightning_actions::{Action, ActionDef, Category, ParamDef, ParamKind};
use lightning_core::content::coerce::{content_to_json_value, json_to_content};
use lightning_core::{ActionError, Content, RunContext};

use crate::{JsLimits, ScriptError, eval};

/// `scripting.run_javascript`
pub struct RunJavaScript;

#[async_trait::async_trait]
impl Action for RunJavaScript {
    fn def(&self) -> ActionDef {
        let mut def = ActionDef::pure(
            "scripting.run_javascript",
            Category::Scripting,
            "code",
            lightning_core::ContentKind::Nothing,
        )
        .with_param(ParamDef::required("script", ParamKind::Text))
        .with_param(ParamDef::optional("timeoutMs", ParamKind::Number));
        // Output depends on the script; no permission class — the sandbox
        // has no fs/net — but the review UI shows the full script text (§14).
        def.output = None;
        def.script_param = Some("script");
        def
    }

    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError> {
        let script = ctx.param_text("script")?;
        let mut limits = JsLimits::default();
        if let Some(timeout) = ctx.param("timeoutMs") {
            let ms = timeout.as_number()?;
            if !(1.0..=60_000.0).contains(&ms) {
                return Err(ActionError::InvalidParam {
                    param: "timeoutMs".into(),
                    message: "must be between 1 and 60000".into(),
                });
            }
            limits.timeout = std::time::Duration::from_millis(ms as u64);
        }

        let input_json = content_to_json_value(&input);
        let outcome = eval(script, input_json, limits)
            .await
            .map_err(|err| match err {
                ScriptError::Timeout(_) => ActionError::Timeout,
                other => ActionError::Failed(other.to_string()),
            })?;

        for line in outcome.console {
            ctx.log.info(None, line);
        }
        Ok(json_to_content(&outcome.value))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use lightning_platform::{CapabilitySnapshot, Os};
    use std::collections::BTreeMap;

    fn ctx_with_script(script: &str) -> RunContext {
        let mut ctx = RunContext::new(CapabilitySnapshot::unconstrained(Os::Linux));
        let mut params = BTreeMap::new();
        params.insert("script".to_owned(), Content::Text(script.to_owned()));
        ctx.set_step_params(params);
        ctx
    }

    #[tokio::test]
    async fn transforms_the_input() {
        let mut ctx = ctx_with_script("input.map((n) => n * 2)");
        let input = Content::List(vec![Content::Number(1.0), Content::Number(2.0)]);
        let out = RunJavaScript.execute(&mut ctx, input).await.unwrap();
        assert_eq!(
            out,
            Content::List(vec![Content::Number(2.0), Content::Number(4.0)])
        );
    }

    #[tokio::test]
    async fn console_goes_to_the_run_log() {
        let mut ctx = ctx_with_script("console.log('from js'); null");
        RunJavaScript
            .execute(&mut ctx, Content::Nothing)
            .await
            .unwrap();
        assert!(ctx.log.entries().iter().any(|e| e.message == "from js"));
    }

    #[tokio::test]
    async fn script_errors_fail_the_step() {
        let mut ctx = ctx_with_script("undefinedFunction()");
        assert!(matches!(
            RunJavaScript.execute(&mut ctx, Content::Nothing).await,
            Err(ActionError::Failed(_))
        ));
    }
}
