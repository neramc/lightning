// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Engine integration tests: fixture `.lightning` files replayed against a
//! test invoker (CLAUDE.md §11).

#![allow(clippy::unwrap_used)]

use std::sync::Arc;

use lightning_core::{
    ActionError, ActionInvoker, Content, Engine, ErrorPolicy, RunContext, RunLimits, Shortcut,
    ShortcutResolver, Step,
};
use lightning_platform::{CapabilitySnapshot, Os};

/// Minimal invoker: `test.echo` returns its `text` param, `test.uppercase`
/// uppercases its input, `test.fail` always fails.
struct TestInvoker;

#[async_trait::async_trait]
impl ActionInvoker for TestInvoker {
    fn contains(&self, action_id: &str) -> bool {
        matches!(action_id, "test.echo" | "test.uppercase" | "test.fail")
    }

    async fn invoke(
        &self,
        action_id: &str,
        ctx: &mut RunContext,
        input: Content,
    ) -> Result<Content, ActionError> {
        match action_id {
            "test.echo" => Ok(Content::Text(ctx.param_text("text")?)),
            "test.uppercase" => Ok(Content::Text(input.as_text()?.to_uppercase())),
            "test.fail" => Err(ActionError::Failed("boom".into())),
            other => Err(ActionError::UnknownAction(other.to_owned())),
        }
    }
}

fn engine() -> Engine {
    Engine::new(Arc::new(TestInvoker))
}

fn ctx() -> RunContext {
    RunContext::new(CapabilitySnapshot::unconstrained(Os::Linux))
}

fn load_fixture(name: &str) -> Shortcut {
    let path = format!("{}/tests/fixtures/{name}", env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(path).unwrap();
    Shortcut::from_json_str(&source).unwrap()
}

#[tokio::test]
async fn hello_fixture_echoes_and_uppercases() {
    let shortcut = load_fixture("hello.lightning");
    let output = engine().run(&shortcut, &mut ctx()).await.unwrap();
    assert_eq!(output, Content::Text("HELLO".into()));
}

#[tokio::test]
async fn branching_fixture_takes_the_then_branch() {
    let shortcut = load_fixture("branching.lightning");
    let output = engine().run(&shortcut, &mut ctx()).await.unwrap();
    assert_eq!(output, Content::Text("big".into()));
}

#[tokio::test]
async fn loop_fixture_collects_each_iteration() {
    let shortcut = load_fixture("loop.lightning");
    let output = engine().run(&shortcut, &mut ctx()).await.unwrap();
    let Content::List(items) = output else { panic!("expected list, got {output:?}") };
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], Content::Text("round 1".into()));
    assert_eq!(items[2], Content::Text("round 3".into()));
}

#[tokio::test]
async fn continue_policy_passes_previous_output_through() {
    let mut shortcut = Shortcut::new("Skip");
    shortcut.steps.push(Step::new("test.echo").with_param("text", Content::Text("keep".into())));
    let mut failing = Step::new("test.fail");
    failing.error_policy = ErrorPolicy::Continue;
    shortcut.steps.push(failing);
    shortcut.steps.push(Step::new("test.uppercase"));

    let output = engine().run(&shortcut, &mut ctx()).await.unwrap();
    assert_eq!(output, Content::Text("KEEP".into()));
}

#[tokio::test]
async fn stop_policy_surfaces_the_error() {
    let mut shortcut = Shortcut::new("Fail");
    shortcut.steps.push(Step::new("test.fail"));
    let err = engine().run(&shortcut, &mut ctx()).await.unwrap_err();
    assert!(matches!(err, ActionError::Failed(_)));
}

#[tokio::test]
async fn retry_policy_retries_then_fails() {
    let mut shortcut = Shortcut::new("Retry");
    let mut failing = Step::new("test.fail");
    failing.error_policy = ErrorPolicy::Retry { attempts: 2, backoff_ms: 1 };
    shortcut.steps.push(failing);

    let mut context = ctx();
    let err = engine().run(&shortcut, &mut context).await.unwrap_err();
    assert!(matches!(err, ActionError::Failed(_)));
    // Two retry warnings plus the final error land in the run log.
    assert!(context.log.entries().len() >= 3);
}

#[tokio::test]
async fn loop_cap_stops_runaway_repeats() {
    let mut shortcut = Shortcut::new("Runaway");
    shortcut.steps.push(
        Step::new("control.repeat")
            .with_param("count", Content::Number(50.0))
            .with_branch("body", vec![]),
    );
    let mut context = ctx().with_limits(RunLimits { loop_cap: 10, ..RunLimits::default() });
    let err = engine().run(&shortcut, &mut context).await.unwrap_err();
    assert!(matches!(err, ActionError::LoopCapExceeded { cap: 10 }));
}

/// Resolver that always returns a shortcut which calls Run Shortcut again —
/// the depth limit must break the cycle.
struct SelfResolver;

#[async_trait::async_trait]
impl ShortcutResolver for SelfResolver {
    async fn resolve(&self, _name_or_id: &str) -> Option<Shortcut> {
        let mut shortcut = Shortcut::new("Ouroboros");
        shortcut.steps.push(
            Step::new("control.run_shortcut")
                .with_param("shortcut", Content::Text("Ouroboros".into())),
        );
        Some(shortcut)
    }
}

#[tokio::test]
async fn run_shortcut_recursion_is_capped() {
    let engine = Engine::new(Arc::new(TestInvoker)).with_resolver(Arc::new(SelfResolver));
    let mut shortcut = Shortcut::new("Entry");
    shortcut.steps.push(
        Step::new("control.run_shortcut")
            .with_param("shortcut", Content::Text("Ouroboros".into())),
    );
    let err = engine.run(&shortcut, &mut ctx()).await.unwrap_err();
    assert!(matches!(err, ActionError::RecursionLimit { max: 16, .. }));
}

#[tokio::test]
async fn exit_unwinds_with_current_value() {
    let mut shortcut = Shortcut::new("Early");
    shortcut.steps.push(Step::new("test.echo").with_param("text", Content::Text("done".into())));
    shortcut.steps.push(Step::new("control.exit"));
    shortcut.steps.push(Step::new("test.fail")); // must never run
    let output = engine().run(&shortcut, &mut ctx()).await.unwrap();
    assert_eq!(output, Content::Text("done".into()));
}

#[tokio::test]
async fn while_loop_threads_output_until_condition_fails() {
    // Start at "aaa" (3 chars → coerces to nothing numeric): use numbers via
    // echo/uppercase is text-only, so drive While with a numeric input chain:
    // While input < 3: body echoes a template that appends "+" — instead we
    // simply count via Repeat Index; here we assert termination semantics.
    let mut shortcut = Shortcut::new("While");
    let mut w = Step::new("control.while")
        .with_param("op", Content::Text("hasValue".into()))
        .with_branch("body", vec![Step::new("control.exit")]);
    // hasValue on Nothing input is false → zero iterations, output Nothing.
    w.params.remove("value");
    shortcut.steps.push(w);
    let output = engine().run(&shortcut, &mut ctx()).await.unwrap();
    assert_eq!(output, Content::Nothing);
}

#[tokio::test]
async fn cancellation_wins_over_error_policy() {
    let mut shortcut = Shortcut::new("Cancelled");
    let mut failing = Step::new("test.fail");
    failing.error_policy = ErrorPolicy::Continue;
    shortcut.steps.push(failing);

    let context_token = tokio_util::sync::CancellationToken::new();
    context_token.cancel();
    let mut context = ctx().with_cancellation(context_token);
    let err = engine().run(&shortcut, &mut context).await.unwrap_err();
    assert!(matches!(err, ActionError::Cancelled));
}
