// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The tree-walking async execution engine (CLAUDE.md §6.4).
//!
//! The engine walks a shortcut's steps on tokio, awaiting each action's
//! `execute`. Control-flow steps (`control.if`, `control.repeat`,
//! `control.repeat_each`, `control.while`, `control.exit`,
//! `control.stop_output`, `control.run_shortcut`) are interpreted here; every
//! other step is dispatched through the [`ActionInvoker`] (the action
//! registry lives in `lightning-actions`).

mod condition;
mod context;
mod progress;

pub use condition::{CondOp, evaluate as evaluate_condition};
pub use context::{RunContext, RunLimits};
pub use progress::{RunProgress, StepPhase};

use std::collections::BTreeMap;
use std::sync::Arc;

use futures::future::BoxFuture;

use crate::content::Content;
use crate::error::ActionError;
use crate::shortcut::{ErrorPolicy, ParamValue, Shortcut, Step};

/// Dispatches non-control-flow steps to their action implementations.
#[async_trait::async_trait]
pub trait ActionInvoker: Send + Sync {
    /// Whether an action id is registered.
    fn contains(&self, action_id: &str) -> bool;
    /// Execute the action. Step params are available via
    /// [`RunContext::param`] and friends.
    async fn invoke(
        &self,
        action_id: &str,
        ctx: &mut RunContext,
        input: Content,
    ) -> Result<Content, ActionError>;
}

/// Resolves `Run Shortcut` references to their definitions.
#[async_trait::async_trait]
pub trait ShortcutResolver: Send + Sync {
    /// Look up a shortcut by name or id string.
    async fn resolve(&self, name_or_id: &str) -> Option<Shortcut>;
}

/// Outcome of running a step sequence.
#[derive(Debug, Clone, PartialEq)]
pub enum Flow {
    /// Continue with this value as the next step's input.
    Continue(Content),
    /// `Exit Shortcut` / `Stop and Output` unwound the run with this value.
    Exit(Content),
}

impl Flow {
    /// The carried value, whichever way the flow ended.
    #[must_use]
    pub fn into_value(self) -> Content {
        match self {
            Flow::Continue(v) | Flow::Exit(v) => v,
        }
    }
}

/// The execution engine. Cheap to clone per run via `Arc`s.
pub struct Engine {
    invoker: Arc<dyn ActionInvoker>,
    resolver: Option<Arc<dyn ShortcutResolver>>,
}

impl Engine {
    /// An engine dispatching to `invoker`.
    #[must_use]
    pub fn new(invoker: Arc<dyn ActionInvoker>) -> Self {
        Self {
            invoker,
            resolver: None,
        }
    }

    /// Attach a `Run Shortcut` resolver, builder-style.
    #[must_use]
    pub fn with_resolver(mut self, resolver: Arc<dyn ShortcutResolver>) -> Self {
        self.resolver = Some(resolver);
        self
    }

    /// Run a shortcut to completion, returning its final output.
    pub async fn run(
        &self,
        shortcut: &Shortcut,
        ctx: &mut RunContext,
    ) -> Result<Content, ActionError> {
        ctx.arm_deadline();
        tracing::info!(shortcut = %shortcut.name, run_id = %ctx.run_id, "run started");
        let flow = self
            .run_steps(&shortcut.steps, ctx, Content::Nothing)
            .await?;
        tracing::info!(run_id = %ctx.run_id, "run finished");
        Ok(flow.into_value())
    }

    /// Run a step slice. Boxed for control-flow recursion.
    fn run_steps<'a>(
        &'a self,
        steps: &'a [Step],
        ctx: &'a mut RunContext,
        input: Content,
    ) -> BoxFuture<'a, Result<Flow, ActionError>> {
        Box::pin(async move {
            let mut current = input;
            for step in steps {
                ctx.check_cancelled()?;
                match self.run_step(step, ctx, current).await? {
                    Flow::Continue(value) => current = value,
                    exit @ Flow::Exit(_) => return Ok(exit),
                }
            }
            Ok(Flow::Continue(current))
        })
    }

    /// Run one step, applying its error policy and emitting progress.
    async fn run_step(
        &self,
        step: &Step,
        ctx: &mut RunContext,
        input: Content,
    ) -> Result<Flow, ActionError> {
        ctx.emit(RunProgress {
            run_id: ctx.run_id,
            step: step.uuid,
            action_id: step.action_id.clone(),
            phase: StepPhase::Started,
        });

        let mut attempt: u32 = 0;
        loop {
            match self.execute_step(step, ctx, input.clone()).await {
                Ok(flow) => {
                    if let Flow::Continue(value) = &flow {
                        ctx.set_magic_output(step.uuid, value.clone());
                        ctx.emit(RunProgress {
                            run_id: ctx.run_id,
                            step: step.uuid,
                            action_id: step.action_id.clone(),
                            phase: StepPhase::Finished {
                                preview: value.preview(120),
                            },
                        });
                    }
                    return Ok(flow);
                }
                // Cancellation and timeouts are never retried or skipped.
                Err(err @ (ActionError::Cancelled | ActionError::Timeout)) => return Err(err),
                Err(err) => match &step.error_policy {
                    ErrorPolicy::Stop => {
                        ctx.log.error(Some(step.uuid), err.to_string());
                        ctx.emit(RunProgress {
                            run_id: ctx.run_id,
                            step: step.uuid,
                            action_id: step.action_id.clone(),
                            phase: StepPhase::Failed {
                                message: err.to_string(),
                            },
                        });
                        return Err(err);
                    }
                    ErrorPolicy::Continue => {
                        ctx.log.warn(
                            Some(step.uuid),
                            format!("step skipped ({err}) — Continue policy"),
                        );
                        ctx.emit(RunProgress {
                            run_id: ctx.run_id,
                            step: step.uuid,
                            action_id: step.action_id.clone(),
                            phase: StepPhase::Skipped {
                                message: err.to_string(),
                            },
                        });
                        // Pass the previous output through so the chain survives.
                        return Ok(Flow::Continue(input));
                    }
                    ErrorPolicy::Retry {
                        attempts,
                        backoff_ms,
                    } => {
                        if attempt >= *attempts {
                            ctx.log.error(
                                Some(step.uuid),
                                format!("failed after {} attempts: {err}", attempt + 1),
                            );
                            ctx.emit(RunProgress {
                                run_id: ctx.run_id,
                                step: step.uuid,
                                action_id: step.action_id.clone(),
                                phase: StepPhase::Failed {
                                    message: err.to_string(),
                                },
                            });
                            return Err(err);
                        }
                        attempt += 1;
                        ctx.log.warn(
                            Some(step.uuid),
                            format!("attempt {attempt} failed ({err}); retrying"),
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(*backoff_ms)).await;
                    }
                },
            }
        }
    }

    /// Execute a step's semantics (control flow here, actions via the invoker).
    async fn execute_step(
        &self,
        step: &Step,
        ctx: &mut RunContext,
        input: Content,
    ) -> Result<Flow, ActionError> {
        match step.action_id.as_str() {
            "control.if" => self.exec_if(step, ctx, input).await,
            "control.repeat" => self.exec_repeat(step, ctx, input).await,
            "control.repeat_each" => self.exec_repeat_each(step, ctx, input).await,
            "control.while" => self.exec_while(step, ctx, input).await,
            "control.exit" => Ok(Flow::Exit(input)),
            "control.stop_output" => {
                let params = self.resolve_params(step, ctx)?;
                let value = params.get("value").cloned().unwrap_or(input);
                Ok(Flow::Exit(value))
            }
            "control.run_shortcut" => self.exec_run_shortcut(step, ctx, input).await,
            _ => {
                let params = self.resolve_params(step, ctx)?;
                ctx.set_step_params(params);
                let result = self.invoker.invoke(&step.action_id, ctx, input).await;
                ctx.clear_step_params();
                Ok(Flow::Continue(result?))
            }
        }
    }

    fn resolve_params(
        &self,
        step: &Step,
        ctx: &mut RunContext,
    ) -> Result<BTreeMap<String, Content>, ActionError> {
        let mut resolved = BTreeMap::new();
        for (key, value) in &step.params {
            let content = match value {
                ParamValue::Literal { value } => value.clone(),
                ParamValue::Variable { name } => ctx
                    .var(name)
                    .cloned()
                    .ok_or_else(|| ActionError::UnknownVariable(name.clone()))?,
                ParamValue::MagicOutput { step: source } => ctx
                    .magic_output(*source)
                    .cloned()
                    .ok_or_else(|| ActionError::InvalidParam {
                        param: key.clone(),
                        message: format!("step {source} has not produced output"),
                    })?,
                ParamValue::Template { template } => Content::Text(ctx.interpolate(template)),
            };
            resolved.insert(key.clone(), content);
        }
        Ok(resolved)
    }

    async fn exec_if(
        &self,
        step: &Step,
        ctx: &mut RunContext,
        input: Content,
    ) -> Result<Flow, ActionError> {
        let params = self.resolve_params(step, ctx)?;
        let op_id = params
            .get("op")
            .ok_or_else(|| ActionError::InvalidParam {
                param: "op".into(),
                message: "required parameter missing".into(),
            })?
            .as_text()?;
        let op = CondOp::parse(&op_id).ok_or_else(|| ActionError::InvalidParam {
            param: "op".into(),
            message: format!("unknown operator '{op_id}'"),
        })?;
        let matched = condition::evaluate(op, &input, params.get("value"))?;
        let label = if matched { "then" } else { "otherwise" };
        match step.branch(label) {
            Some(branch) => self.run_steps(&branch.steps, ctx, input).await,
            None => Ok(Flow::Continue(input)),
        }
    }

    async fn exec_repeat(
        &self,
        step: &Step,
        ctx: &mut RunContext,
        input: Content,
    ) -> Result<Flow, ActionError> {
        let params = self.resolve_params(step, ctx)?;
        let count = params
            .get("count")
            .ok_or_else(|| ActionError::InvalidParam {
                param: "count".into(),
                message: "required parameter missing".into(),
            })?
            .as_number()?;
        if !count.is_finite() || count < 0.0 {
            return Err(ActionError::InvalidParam {
                param: "count".into(),
                message: "count must be a non-negative number".into(),
            });
        }
        let count = count as u64;
        if count > ctx.limits.loop_cap {
            return Err(ActionError::LoopCapExceeded {
                cap: ctx.limits.loop_cap,
            });
        }
        let body = step.branch("body").cloned().unwrap_or_default_branch();
        let mut results = Vec::new();
        for index in 1..=count {
            ctx.check_cancelled()?;
            ctx.set_var("Repeat Index", Content::Number(index as f64));
            match self.run_steps(&body.steps, ctx, input.clone()).await? {
                Flow::Continue(value) => {
                    if value != Content::Nothing {
                        results.push(value);
                    }
                }
                exit @ Flow::Exit(_) => return Ok(exit),
            }
        }
        Ok(Flow::Continue(Content::List(results)))
    }

    async fn exec_repeat_each(
        &self,
        step: &Step,
        ctx: &mut RunContext,
        input: Content,
    ) -> Result<Flow, ActionError> {
        let params = self.resolve_params(step, ctx)?;
        let items = params.get("items").cloned().unwrap_or(input).into_items();
        if items.len() as u64 > ctx.limits.loop_cap {
            return Err(ActionError::LoopCapExceeded {
                cap: ctx.limits.loop_cap,
            });
        }
        let body = step.branch("body").cloned().unwrap_or_default_branch();
        let mut results = Vec::new();
        for (index, item) in items.into_iter().enumerate() {
            ctx.check_cancelled()?;
            ctx.set_var("Repeat Index", Content::Number((index + 1) as f64));
            ctx.set_var("Repeat Item", item.clone());
            match self.run_steps(&body.steps, ctx, item).await? {
                Flow::Continue(value) => {
                    if value != Content::Nothing {
                        results.push(value);
                    }
                }
                exit @ Flow::Exit(_) => return Ok(exit),
            }
        }
        Ok(Flow::Continue(Content::List(results)))
    }

    async fn exec_while(
        &self,
        step: &Step,
        ctx: &mut RunContext,
        input: Content,
    ) -> Result<Flow, ActionError> {
        let body = step.branch("body").cloned().unwrap_or_default_branch();
        let mut current = input;
        let mut iterations: u64 = 0;
        loop {
            ctx.check_cancelled()?;
            let params = self.resolve_params(step, ctx)?;
            let op_id = params
                .get("op")
                .ok_or_else(|| ActionError::InvalidParam {
                    param: "op".into(),
                    message: "required parameter missing".into(),
                })?
                .as_text()?;
            let op = CondOp::parse(&op_id).ok_or_else(|| ActionError::InvalidParam {
                param: "op".into(),
                message: format!("unknown operator '{op_id}'"),
            })?;
            if !condition::evaluate(op, &current, params.get("value"))? {
                return Ok(Flow::Continue(current));
            }
            iterations += 1;
            if iterations > ctx.limits.loop_cap {
                return Err(ActionError::LoopCapExceeded {
                    cap: ctx.limits.loop_cap,
                });
            }
            match self.run_steps(&body.steps, ctx, current).await? {
                Flow::Continue(value) => current = value,
                exit @ Flow::Exit(_) => return Ok(exit),
            }
        }
    }

    async fn exec_run_shortcut(
        &self,
        step: &Step,
        ctx: &mut RunContext,
        input: Content,
    ) -> Result<Flow, ActionError> {
        let params = self.resolve_params(step, ctx)?;
        let reference = params
            .get("shortcut")
            .ok_or_else(|| ActionError::InvalidParam {
                param: "shortcut".into(),
                message: "required parameter missing".into(),
            })?
            .as_text()?;
        let depth = ctx.depth + 1;
        if depth > ctx.limits.max_recursion {
            return Err(ActionError::RecursionLimit {
                depth,
                max: ctx.limits.max_recursion,
            });
        }
        let resolver = self.resolver.as_ref().ok_or_else(|| {
            ActionError::Failed("Run Shortcut is not available in this context".into())
        })?;
        let child = resolver
            .resolve(&reference)
            .await
            .ok_or_else(|| ActionError::Failed(format!("shortcut '{reference}' not found")))?;
        ctx.depth = depth;
        let result = self.run_steps(&child.steps, ctx, input).await;
        ctx.depth -= 1;
        // A sub-shortcut's Exit ends the child, not the parent.
        result.map(|flow| Flow::Continue(flow.into_value()))
    }
}

trait BranchExt {
    fn unwrap_or_default_branch(self) -> crate::shortcut::Branch;
}

impl BranchExt for Option<crate::shortcut::Branch> {
    fn unwrap_or_default_branch(self) -> crate::shortcut::Branch {
        self.unwrap_or(crate::shortcut::Branch {
            label: "body".to_owned(),
            steps: Vec::new(),
        })
    }
}
