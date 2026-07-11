// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! [`RunContext`] — everything one run carries with it (CLAUDE.md §6.4):
//! variable scope, coercer access, cancellation, granted permissions, the
//! capability snapshot, and the structured run logger.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::Duration;

use lightning_platform::CapabilitySnapshot;
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use super::progress::RunProgress;
use crate::content::Content;
use crate::error::ActionError;
use crate::permission::PermissionClass;
use crate::run_log::RunLog;

/// Hard limits that make runaway automations impossible (§6.4).
#[derive(Debug, Clone, Copy)]
pub struct RunLimits {
    /// Maximum `Run Shortcut` nesting depth.
    pub max_recursion: u32,
    /// Loop iteration cap (user-raisable per shortcut).
    pub loop_cap: u64,
    /// Wall-clock budget; automations always set one.
    pub timeout: Option<Duration>,
}

impl Default for RunLimits {
    fn default() -> Self {
        Self { max_recursion: 16, loop_cap: 10_000, timeout: None }
    }
}

/// Mutable state threaded through one run.
pub struct RunContext {
    /// Identity of this run (correlates progress events and history rows).
    pub run_id: Uuid,
    /// Capability snapshot the run executes under.
    pub caps: CapabilitySnapshot,
    /// Hard limits.
    pub limits: RunLimits,
    /// Current `Run Shortcut` nesting depth.
    pub depth: u32,
    /// The user-visible run log.
    pub log: RunLog,
    vars: HashMap<String, Content>,
    magic: HashMap<Uuid, Content>,
    params: BTreeMap<String, Content>,
    granted: BTreeSet<PermissionClass>,
    cancel: CancellationToken,
    deadline: Option<tokio::time::Instant>,
    progress: Option<UnboundedSender<RunProgress>>,
}

impl RunContext {
    /// A context with default limits and no grants.
    #[must_use]
    pub fn new(caps: CapabilitySnapshot) -> Self {
        Self {
            run_id: Uuid::new_v4(),
            caps,
            limits: RunLimits::default(),
            depth: 0,
            log: RunLog::default(),
            vars: HashMap::new(),
            magic: HashMap::new(),
            params: BTreeMap::new(),
            granted: BTreeSet::new(),
            cancel: CancellationToken::new(),
            deadline: None,
            progress: None,
        }
    }

    /// Grant permission classes, builder-style.
    #[must_use]
    pub fn with_permissions(
        mut self,
        classes: impl IntoIterator<Item = PermissionClass>,
    ) -> Self {
        self.granted.extend(classes);
        self
    }

    /// Attach a progress event sender, builder-style.
    #[must_use]
    pub fn with_progress(mut self, sender: UnboundedSender<RunProgress>) -> Self {
        self.progress = Some(sender);
        self
    }

    /// Override the limits, builder-style. The timeout starts counting when
    /// the engine begins executing.
    #[must_use]
    pub fn with_limits(mut self, limits: RunLimits) -> Self {
        self.limits = limits;
        self
    }

    /// Use an external cancellation token, builder-style.
    #[must_use]
    pub fn with_cancellation(mut self, token: CancellationToken) -> Self {
        self.cancel = token;
        self
    }

    /// The cancellation token (for actions that sleep or poll).
    #[must_use]
    pub fn cancellation(&self) -> CancellationToken {
        self.cancel.clone()
    }

    /// Arm the wall-clock deadline from `limits.timeout`. Called by the
    /// engine at run start.
    pub(crate) fn arm_deadline(&mut self) {
        if let Some(timeout) = self.limits.timeout {
            self.deadline = Some(tokio::time::Instant::now() + timeout);
        }
    }

    /// Error if the run was cancelled or ran past its deadline.
    pub fn check_cancelled(&self) -> Result<(), ActionError> {
        if self.cancel.is_cancelled() {
            return Err(ActionError::Cancelled);
        }
        if let Some(deadline) = self.deadline
            && tokio::time::Instant::now() >= deadline
        {
            return Err(ActionError::Timeout);
        }
        Ok(())
    }

    // ── variables ──────────────────────────────────────────────────────────

    /// Read a named variable.
    #[must_use]
    pub fn var(&self, name: &str) -> Option<&Content> {
        self.vars.get(name)
    }

    /// Set a named variable.
    pub fn set_var(&mut self, name: impl Into<String>, value: Content) {
        self.vars.insert(name.into(), value);
    }

    /// Read a magic output by producing step.
    #[must_use]
    pub fn magic_output(&self, step: Uuid) -> Option<&Content> {
        self.magic.get(&step)
    }

    /// Record a step's magic output.
    pub fn set_magic_output(&mut self, step: Uuid, value: Content) {
        self.magic.insert(step, value);
    }

    /// Interpolate `{{variable}}` references. Unknown variables render empty
    /// and log a warning — templates never hard-fail a run.
    pub fn interpolate(&mut self, template: &str) -> String {
        let mut out = String::with_capacity(template.len());
        let mut rest = template;
        while let Some(start) = rest.find("{{") {
            out.push_str(&rest[..start]);
            let after = &rest[start + 2..];
            match after.find("}}") {
                Some(end) => {
                    let name = after[..end].trim();
                    match self.vars.get(name).map(|v| v.as_text()) {
                        Some(Ok(text)) => out.push_str(&text),
                        Some(Err(_)) | None => {
                            self.log.warn(
                                None,
                                format!("template references unknown variable '{name}'"),
                            );
                        }
                    }
                    rest = &after[end + 2..];
                }
                None => {
                    out.push_str(&rest[start..]);
                    rest = "";
                }
            }
        }
        out.push_str(rest);
        out
    }

    // ── step params (set by the engine around each invoke) ─────────────────

    /// Replace the current step's resolved params. Called by the engine
    /// around each invoke, and by action unit tests.
    pub fn set_step_params(&mut self, params: BTreeMap<String, Content>) {
        self.params = params;
    }

    /// Clear the current step's params.
    pub fn clear_step_params(&mut self) {
        self.params.clear();
    }

    /// The current step's resolved parameter, if present.
    #[must_use]
    pub fn param(&self, key: &str) -> Option<&Content> {
        self.params.get(key)
    }

    /// The current step's resolved parameter, or `InvalidParam`.
    pub fn required_param(&self, key: &str) -> Result<&Content, ActionError> {
        self.param(key).ok_or_else(|| ActionError::InvalidParam {
            param: key.to_owned(),
            message: "required parameter missing".into(),
        })
    }

    /// Required parameter coerced to text.
    pub fn param_text(&self, key: &str) -> Result<String, ActionError> {
        Ok(self.required_param(key)?.as_text()?)
    }

    /// Required parameter coerced to a number.
    pub fn param_number(&self, key: &str) -> Result<f64, ActionError> {
        Ok(self.required_param(key)?.as_number()?)
    }

    /// Optional parameter coerced to text, with a default.
    pub fn param_text_or(&self, key: &str, default: &str) -> Result<String, ActionError> {
        match self.param(key) {
            Some(value) => Ok(value.as_text()?),
            None => Ok(default.to_owned()),
        }
    }

    /// Optional boolean parameter, with a default.
    pub fn param_bool_or(&self, key: &str, default: bool) -> Result<bool, ActionError> {
        match self.param(key) {
            Some(value) => Ok(value.as_boolean()?),
            None => Ok(default),
        }
    }

    // ── permissions & progress ──────────────────────────────────────────────

    /// Error unless `class` was granted to this shortcut (§14).
    pub fn require_permission(&self, class: PermissionClass) -> Result<(), ActionError> {
        if self.granted.contains(&class) {
            Ok(())
        } else {
            Err(ActionError::PermissionNotGranted { class })
        }
    }

    /// Emit a progress event (drops silently if no listener is attached).
    pub fn emit(&self, event: RunProgress) {
        if let Some(sender) = &self.progress {
            let _ = sender.send(event);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lightning_platform::{CapabilitySnapshot, Os};

    fn ctx() -> RunContext {
        RunContext::new(CapabilitySnapshot::unconstrained(Os::Linux))
    }

    #[test]
    fn interpolate_replaces_known_variables() {
        let mut c = ctx();
        c.set_var("name", Content::Text("World".into()));
        assert_eq!(c.interpolate("Hello {{name}}!"), "Hello World!");
    }

    #[test]
    fn interpolate_renders_unknown_as_empty_and_warns() {
        let mut c = ctx();
        assert_eq!(c.interpolate("Hi {{missing}}!"), "Hi !");
        assert_eq!(c.log.entries().len(), 1);
    }

    #[test]
    fn interpolate_keeps_unclosed_braces_literal() {
        let mut c = ctx();
        assert_eq!(c.interpolate("brace {{open"), "brace {{open");
    }

    #[test]
    fn permissions_gate() {
        let c = ctx().with_permissions([PermissionClass::Network]);
        assert!(c.require_permission(PermissionClass::Network).is_ok());
        assert!(matches!(
            c.require_permission(PermissionClass::Shell),
            Err(ActionError::PermissionNotGranted { .. })
        ));
    }
}
