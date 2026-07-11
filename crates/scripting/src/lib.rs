// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! # lightning-scripting
//!
//! The sandboxed QuickJS runtime powering the cross-platform **Run
//! JavaScript** action (CLAUDE.md §4, §8.3 N).
//!
//! Sandbox model: the runtime exposes **no host APIs at all** — no
//! filesystem, no network, no process access. The script sees exactly two
//! things: a global `input` (the previous step's output as JSON) and a
//! `console.log` that collects lines for the run log. Wall-clock and memory
//! limits are enforced by the QuickJS interrupt handler and allocator cap.

mod action;

pub use action::RunJavaScript;

use std::time::{Duration, Instant};

/// Limits applied to one evaluation.
#[derive(Debug, Clone, Copy)]
pub struct JsLimits {
    /// Wall-clock budget.
    pub timeout: Duration,
    /// Heap cap in bytes.
    pub memory_limit: usize,
}

impl Default for JsLimits {
    fn default() -> Self {
        Self {
            timeout: Duration::from_millis(5_000),
            memory_limit: 32 * 1024 * 1024,
        }
    }
}

/// Result of one evaluation.
#[derive(Debug, Clone, PartialEq)]
pub struct JsOutcome {
    /// The script's completion value as JSON (`null` for `undefined`).
    pub value: serde_json::Value,
    /// Captured `console.log` lines.
    pub console: Vec<String>,
}

/// Errors from the JS runtime.
#[derive(Debug, thiserror::Error)]
pub enum ScriptError {
    /// Could not construct the runtime.
    #[error("failed to initialize the JavaScript runtime: {0}")]
    Init(String),
    /// The script threw or failed to parse.
    #[error("JavaScript error: {0}")]
    Eval(String),
    /// The wall-clock budget was exceeded.
    #[error("script exceeded its {0:?} time budget")]
    Timeout(Duration),
    /// The completion value could not be serialized to JSON.
    #[error("script result is not JSON-serializable")]
    NonSerializable,
}

/// Bootstrap that defines the sandbox surface (`console` capture). Runs
/// before the user script; keep it dependency-free JS.
const SANDBOX_PRELUDE: &str = r#"
globalThis.__logs = [];
globalThis.console = {
  log: (...args) => {
    globalThis.__logs.push(
      args
        .map((a) => (typeof a === 'string' ? a : JSON.stringify(a)))
        .join(' '),
    );
  },
};
"#;

/// Evaluate `source` synchronously. Prefer [`eval`] from async contexts.
pub fn eval_blocking(
    source: &str,
    input: &serde_json::Value,
    limits: JsLimits,
) -> Result<JsOutcome, ScriptError> {
    let runtime = rquickjs::Runtime::new().map_err(|e| ScriptError::Init(e.to_string()))?;
    runtime.set_memory_limit(limits.memory_limit);
    let deadline = Instant::now() + limits.timeout;
    runtime.set_interrupt_handler(Some(Box::new(move || Instant::now() >= deadline)));

    let context =
        rquickjs::Context::full(&runtime).map_err(|e| ScriptError::Init(e.to_string()))?;

    context.with(|ctx| {
        let describe = |ctx: &rquickjs::Ctx<'_>, err: rquickjs::Error| -> ScriptError {
            if Instant::now() >= deadline {
                return ScriptError::Timeout(limits.timeout);
            }
            if err.is_exception() {
                let caught = ctx.catch();
                let message = ctx
                    .json_stringify(caught)
                    .ok()
                    .flatten()
                    .and_then(|s| s.to_string().ok())
                    .unwrap_or_else(|| "uncaught exception".to_owned());
                ScriptError::Eval(message)
            } else {
                ScriptError::Eval(err.to_string())
            }
        };

        ctx.eval::<(), _>(SANDBOX_PRELUDE)
            .map_err(|e| describe(&ctx, e))?;

        // Inject the input as parsed JSON.
        let input_json = serde_json::to_string(input).map_err(|_| ScriptError::NonSerializable)?;
        ctx.globals()
            .set("__INPUT_JSON__", input_json)
            .map_err(|e| describe(&ctx, e))?;
        ctx.eval::<(), _>("globalThis.input = JSON.parse(globalThis.__INPUT_JSON__);")
            .map_err(|e| describe(&ctx, e))?;

        // Run the user script and serialize its completion value.
        let value: rquickjs::Value = ctx.eval(source).map_err(|e| describe(&ctx, e))?;
        let value_json = if value.is_undefined() {
            serde_json::Value::Null
        } else {
            let text = ctx
                .json_stringify(value)
                .map_err(|e| describe(&ctx, e))?
                .and_then(|s| s.to_string().ok())
                .ok_or(ScriptError::NonSerializable)?;
            serde_json::from_str(&text).map_err(|_| ScriptError::NonSerializable)?
        };

        // Collect console output.
        let logs: rquickjs::Value = ctx
            .eval("globalThis.__logs")
            .map_err(|e| describe(&ctx, e))?;
        let console: Vec<String> = ctx
            .json_stringify(logs)
            .ok()
            .flatten()
            .and_then(|s| s.to_string().ok())
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();

        Ok(JsOutcome {
            value: value_json,
            console,
        })
    })
}

/// Evaluate `source` off the async runtime via `spawn_blocking`.
pub async fn eval(
    source: String,
    input: serde_json::Value,
    limits: JsLimits,
) -> Result<JsOutcome, ScriptError> {
    tokio::task::spawn_blocking(move || eval_blocking(&source, &input, limits))
        .await
        .map_err(|e| ScriptError::Init(format!("worker panicked: {e}")))?
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn evaluates_pure_expressions() {
        let out = eval_blocking("1 + 2", &serde_json::Value::Null, JsLimits::default()).unwrap();
        assert_eq!(out.value, serde_json::json!(3));
    }

    #[test]
    fn input_is_visible_as_parsed_json() {
        let input = serde_json::json!({ "a": 21 });
        let out = eval_blocking("input.a * 2", &input, JsLimits::default()).unwrap();
        assert_eq!(out.value, serde_json::json!(42));
    }

    #[test]
    fn console_log_is_captured() {
        let out = eval_blocking(
            "console.log('hello', { x: 1 }); 'done'",
            &serde_json::Value::Null,
            JsLimits::default(),
        )
        .unwrap();
        assert_eq!(out.console, vec!["hello {\"x\":1}".to_owned()]);
        assert_eq!(out.value, serde_json::json!("done"));
    }

    #[test]
    fn runaway_scripts_hit_the_time_budget() {
        let limits = JsLimits {
            timeout: Duration::from_millis(100),
            ..JsLimits::default()
        };
        let err = eval_blocking("while (true) {}", &serde_json::Value::Null, limits).unwrap_err();
        assert!(matches!(err, ScriptError::Timeout(_)), "got {err:?}");
    }

    #[test]
    fn exceptions_surface_as_eval_errors() {
        let err = eval_blocking(
            "throw new Error('nope')",
            &serde_json::Value::Null,
            JsLimits::default(),
        )
        .unwrap_err();
        assert!(matches!(err, ScriptError::Eval(_)));
    }

    #[test]
    fn no_host_apis_exist_in_the_sandbox() {
        for probe in ["typeof require", "typeof process", "typeof fetch"] {
            let out = eval_blocking(probe, &serde_json::Value::Null, JsLimits::default()).unwrap();
            assert_eq!(out.value, serde_json::json!("undefined"), "{probe}");
        }
    }
}
