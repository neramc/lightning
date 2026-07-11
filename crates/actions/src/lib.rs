// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! # lightning-actions
//!
//! The [`Action`] trait, the [`Registry`], and the built-in cross-platform
//! action catalog (CLAUDE.md §6.5, §8.3).
//!
//! Actions self-register into the registry; the frontend fetches the whole
//! catalog (defs only) through one `list_actions` command and renders
//! everything from data — it never hardcodes an action list. Param schemas
//! declared here flow to TypeScript via `lightning-ipc-types`.
//!
//! Adding an action? Follow the recipe in CLAUDE.md §8.6 **exactly** —
//! spec first in `docs/actions/`, honest support matrix, i18n en+ko, tests.

#![deny(missing_docs)]

pub mod control;
pub mod date;
pub mod math;
mod registry;
pub mod text;

pub use registry::Registry;

use lightning_core::{ActionError, Content, ContentKind, PermissionClass, RunContext};
use lightning_platform::{Capability, PlatformSupport};

/// Static definition of an action: identity, parameter schema, support
/// matrix, and permission class. Everything the editor needs to render the
/// block comes from here — as data.
#[derive(Debug, Clone)]
pub struct ActionDef {
    /// Stable id, `<category>.<name>` (e.g. `text.change_case`). i18n keys
    /// derive from it: `actions.<id>.name` / `.description`.
    pub id: &'static str,
    /// Category (drives the gradient token and library grouping).
    pub category: Category,
    /// Icon token from the design system.
    pub icon: &'static str,
    /// Parameter schema; the editor auto-renders the param UI from this.
    pub params: Vec<ParamDef>,
    /// The output content kind; `None` when it depends on the script/input
    /// (e.g. Run JavaScript).
    pub output: Option<ContentKind>,
    /// Per-OS static support (§8.1). A wrong `Full` is worse than an honest
    /// `None`.
    pub supports: PlatformSupport,
    /// Permission class required at run time, if any (§14).
    pub permission: Option<PermissionClass>,
    /// Runtime capability this action needs (intersected with the probe).
    pub requires_capability: Option<Capability>,
    /// If set, the named param holds script text that the permission review
    /// UI must display in full (§14).
    pub script_param: Option<&'static str>,
}

impl ActionDef {
    /// A def with full cross-platform support and no permissions — the
    /// baseline for pure, engine-level actions.
    #[must_use]
    pub fn pure(
        id: &'static str,
        category: Category,
        icon: &'static str,
        output: ContentKind,
    ) -> Self {
        Self {
            id,
            category,
            icon,
            params: Vec::new(),
            output: Some(output),
            supports: PlatformSupport::all_full(),
            permission: None,
            requires_capability: None,
            script_param: None,
        }
    }

    /// Add a parameter, builder-style.
    #[must_use]
    pub fn with_param(mut self, param: ParamDef) -> Self {
        self.params.push(param);
        self
    }

    /// Set the support matrix, builder-style.
    #[must_use]
    pub fn with_supports(mut self, supports: PlatformSupport) -> Self {
        self.supports = supports;
        self
    }

    /// Set the permission class, builder-style.
    #[must_use]
    pub fn with_permission(mut self, permission: PermissionClass) -> Self {
        self.permission = Some(permission);
        self
    }

    /// Set the required runtime capability, builder-style.
    #[must_use]
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.requires_capability = Some(capability);
        self
    }
}

/// One parameter in an action's schema.
#[derive(Debug, Clone)]
pub struct ParamDef {
    /// Stable key used in step params and i18n (`actions.<id>.params.<key>`).
    pub key: &'static str,
    /// Widget/value kind.
    pub kind: ParamKind,
    /// Whether the editor requires a value before the shortcut can run.
    pub required: bool,
}

impl ParamDef {
    /// A required parameter.
    #[must_use]
    pub fn required(key: &'static str, kind: ParamKind) -> Self {
        Self {
            key,
            kind,
            required: true,
        }
    }

    /// An optional parameter.
    #[must_use]
    pub fn optional(key: &'static str, kind: ParamKind) -> Self {
        Self {
            key,
            kind,
            required: false,
        }
    }
}

/// The value/widget kind of a parameter. Every kind can alternatively be
/// filled with a variable or magic output — that is editor-level, not
/// schema-level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParamKind {
    /// Free text (rendered as a template field with variable chips).
    Text,
    /// A number field.
    Number,
    /// A toggle.
    Boolean,
    /// A date/time picker.
    Date,
    /// A file picker.
    File,
    /// A fixed choice list; options are stable ids localized via i18n.
    Enum(&'static [&'static str]),
}

/// Action categories (§8.3). Ids double as gradient-token names in
/// `packages/ui` and as i18n key segments.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    /// Control flow & scripting basics.
    ControlFlow,
    /// Text handling.
    Text,
    /// Math & numbers.
    Math,
    /// Date & time.
    Date,
    /// Web & network.
    Web,
    /// Files & folders.
    Files,
    /// Clipboard.
    Clipboard,
    /// Images & graphics.
    Images,
    /// PDF & documents.
    Documents,
    /// Audio, video & speech.
    Media,
    /// Apps & windows.
    AppsWindows,
    /// System & device.
    System,
    /// Input & UI automation.
    Input,
    /// Scripting bridges.
    Scripting,
    /// Communication & sharing.
    Communication,
    /// Productivity.
    Productivity,
    /// Location & weather.
    Location,
}

impl Category {
    /// Stable camelCase id (gradient token + i18n segment).
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Category::ControlFlow => "controlFlow",
            Category::Text => "text",
            Category::Math => "math",
            Category::Date => "date",
            Category::Web => "web",
            Category::Files => "files",
            Category::Clipboard => "clipboard",
            Category::Images => "images",
            Category::Documents => "documents",
            Category::Media => "media",
            Category::AppsWindows => "appsWindows",
            Category::System => "system",
            Category::Input => "input",
            Category::Scripting => "scripting",
            Category::Communication => "communication",
            Category::Productivity => "productivity",
            Category::Location => "location",
        }
    }
}

/// One executable action (CLAUDE.md §6.5).
#[async_trait::async_trait]
pub trait Action: Send + Sync {
    /// The static definition.
    fn def(&self) -> ActionDef;
    /// Execute against the previous step's output. Step params are available
    /// via [`RunContext::param`] and friends.
    async fn execute(&self, ctx: &mut RunContext, input: Content) -> Result<Content, ActionError>;
}

#[cfg(test)]
pub(crate) mod test_util {
    //! Shared helpers for action unit tests.

    use std::collections::BTreeMap;

    use lightning_core::{Content, RunContext};
    use lightning_platform::{CapabilitySnapshot, Os};

    /// A context with the given resolved step params.
    pub fn ctx_with(params: &[(&str, Content)]) -> RunContext {
        let mut ctx = RunContext::new(CapabilitySnapshot::unconstrained(Os::Linux));
        ctx.set_step_params(
            params
                .iter()
                .map(|(k, v)| ((*k).to_owned(), v.clone()))
                .collect::<BTreeMap<_, _>>(),
        );
        ctx
    }

    /// A context with no params.
    pub fn ctx() -> RunContext {
        ctx_with(&[])
    }
}
