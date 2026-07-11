// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The shortcut model and `.lightning` file format (CLAUDE.md §6.3).
//!
//! A shortcut is an ordered tree of [`Step`]s; control-flow steps own child
//! steps through [`Branch`]es. Files are pretty-printed JSON with a top-level
//! `schemaVersion`; any breaking change requires a version bump plus a
//! migration in [`crate::migrate`] — old files must always open.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::content::Content;
use crate::error::MigrateError;
use crate::migrate;

/// Icon of a shortcut tile: a glyph plus a category gradient token.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Icon {
    /// Emoji or named glyph rendered on the tile.
    pub glyph: String,
    /// Gradient token from the design system (never a raw hex value).
    pub gradient: String,
}

impl Default for Icon {
    fn default() -> Self {
        Self {
            glyph: "⚡".to_owned(),
            gradient: "system".to_owned(),
        }
    }
}

/// How a step's parameter value is produced at run time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum ParamValue {
    /// A literal content value.
    Literal {
        /// The value.
        value: Content,
    },
    /// A named variable set earlier in the run.
    Variable {
        /// Variable name.
        name: String,
    },
    /// The magic output of a previous step.
    MagicOutput {
        /// UUID of the producing step.
        step: Uuid,
    },
    /// A text template with `{{variable}}` interpolation.
    Template {
        /// The template source.
        template: String,
    },
}

/// Per-step error policy (CLAUDE.md §6.4). Default is `Stop`.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(
    tag = "policy",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum ErrorPolicy {
    /// Stop the run and surface the error.
    #[default]
    Stop,
    /// Log a warning, skip the step, pass the previous output through.
    Continue,
    /// Retry with backoff before giving up.
    Retry {
        /// Additional attempts after the first failure.
        attempts: u32,
        /// Delay between attempts, in milliseconds.
        backoff_ms: u64,
    },
}

/// A named group of child steps owned by a control-flow step
/// (`then` / `otherwise` for If, `body` for loops, option labels for Menu).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Branch {
    /// Branch label.
    pub label: String,
    /// The child steps.
    #[serde(default)]
    pub steps: Vec<Step>,
}

/// One block in the flow.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Step {
    /// Stable identity — magic-variable references point at this.
    pub uuid: Uuid,
    /// The action id (e.g. `text.change_case`, `control.if`).
    pub action_id: String,
    /// Parameter values keyed by the action's param schema keys.
    #[serde(default)]
    pub params: BTreeMap<String, ParamValue>,
    /// What to do when this step fails.
    #[serde(default)]
    pub error_policy: ErrorPolicy,
    /// Child branches for control-flow steps; empty otherwise.
    #[serde(default)]
    pub branches: Vec<Branch>,
}

impl Step {
    /// A new step for `action_id` with a fresh UUID and no params.
    #[must_use]
    pub fn new(action_id: impl Into<String>) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            action_id: action_id.into(),
            params: BTreeMap::new(),
            error_policy: ErrorPolicy::default(),
            branches: Vec::new(),
        }
    }

    /// Set a literal parameter, builder-style.
    #[must_use]
    pub fn with_param(mut self, key: impl Into<String>, value: Content) -> Self {
        self.params
            .insert(key.into(), ParamValue::Literal { value });
        self
    }

    /// Set any parameter value, builder-style.
    #[must_use]
    pub fn with_param_value(mut self, key: impl Into<String>, value: ParamValue) -> Self {
        self.params.insert(key.into(), value);
        self
    }

    /// Add a branch, builder-style.
    #[must_use]
    pub fn with_branch(mut self, label: impl Into<String>, steps: Vec<Step>) -> Self {
        self.branches.push(Branch {
            label: label.into(),
            steps,
        });
        self
    }

    /// Find a branch by label.
    #[must_use]
    pub fn branch(&self, label: &str) -> Option<&Branch> {
        self.branches.iter().find(|b| b.label == label)
    }
}

/// The trigger block that turns a shortcut into an automation (§6.7).
/// The trigger id and config are interpreted by `lightning-triggers`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TriggerConfig {
    /// Trigger id (e.g. `trigger.interval`, `trigger.file_changed`).
    pub trigger_id: String,
    /// Trigger-specific configuration.
    #[serde(default)]
    pub config: serde_json::Value,
    /// Whether the automation is armed.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Per-automation cooldown in milliseconds (default 2000 — §6.7).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cooldown_ms: Option<u64>,
}

fn default_true() -> bool {
    true
}

/// A shortcut (or automation) — the unit stored as one `.lightning` file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Shortcut {
    /// File format version; see [`crate::migrate`].
    pub schema_version: u32,
    /// Stable identity.
    pub id: Uuid,
    /// Display name.
    pub name: String,
    /// Optional description shown in the gallery and editor.
    #[serde(default)]
    pub description: String,
    /// Tile icon.
    #[serde(default)]
    pub icon: Icon,
    /// Global hotkey (e.g. `Ctrl+Shift+L`), if assigned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hotkey: Option<String>,
    /// The flow.
    #[serde(default)]
    pub steps: Vec<Step>,
    /// Present iff this is an automation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger: Option<TriggerConfig>,
}

impl Shortcut {
    /// A new, empty shortcut at the current schema version.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            schema_version: migrate::CURRENT_SCHEMA_VERSION,
            id: Uuid::new_v4(),
            name: name.into(),
            description: String::new(),
            icon: Icon::default(),
            hotkey: None,
            steps: Vec::new(),
            trigger: None,
        }
    }

    /// Whether this shortcut is an automation (has an armed trigger block).
    #[must_use]
    pub fn is_automation(&self) -> bool {
        self.trigger.is_some()
    }

    /// Serialize as the pretty-printed `.lightning` JSON document.
    pub fn to_pretty_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Parse a `.lightning` document, migrating older schema versions first.
    /// Old files must always open (CLAUDE.md §2.6).
    pub fn from_json_str(source: &str) -> Result<Self, MigrateError> {
        let mut doc: serde_json::Value =
            serde_json::from_str(source).map_err(|_| MigrateError::InvalidFile)?;
        migrate::migrate_to_current(&mut doc)?;
        serde_json::from_value(doc).map_err(|err| MigrateError::StepFailed {
            from: migrate::CURRENT_SCHEMA_VERSION,
            message: format!("deserialize after migration: {err}"),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_preserves_the_document() {
        let mut shortcut = Shortcut::new("Greet");
        shortcut
            .steps
            .push(Step::new("text.text").with_param("text", Content::Text("hello".into())));
        let json = shortcut.to_pretty_json().expect("serialize");
        let parsed = Shortcut::from_json_str(&json).expect("parse");
        assert_eq!(parsed, shortcut);
    }

    #[test]
    fn top_level_schema_version_is_camel_case() {
        let shortcut = Shortcut::new("X");
        let value = serde_json::to_value(&shortcut).expect("serialize");
        assert!(value.get("schemaVersion").is_some());
    }

    #[test]
    fn branch_lookup_by_label() {
        let step = Step::new("control.if")
            .with_branch("then", vec![Step::new("control.nothing")])
            .with_branch("otherwise", vec![]);
        assert_eq!(step.branch("then").map(|b| b.steps.len()), Some(1));
        assert!(step.branch("missing").is_none());
    }
}
