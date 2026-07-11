// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The action registry: id → implementation, plus the runtime gate that
//! intersects static support with the capability probe and enforces
//! permission classes before every execute (§6.5, §6.6, §14).

use std::collections::BTreeMap;
use std::sync::Arc;

use lightning_core::{ActionError, ActionInvoker, Content, RunContext};
use lightning_platform::{CapabilityFix, CapabilityStatus, SupportLevel};

use crate::{Action, ActionDef};

/// All registered actions.
#[derive(Default)]
pub struct Registry {
    actions: BTreeMap<&'static str, Arc<dyn Action>>,
}

impl Registry {
    /// An empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A registry with every built-in action registered.
    #[must_use]
    pub fn with_builtins() -> Self {
        let mut registry = Self::new();
        crate::control::register(&mut registry);
        crate::text::register(&mut registry);
        crate::math::register(&mut registry);
        crate::date::register(&mut registry);
        registry
    }

    /// Register one action. Panics on duplicate ids — that is a programming
    /// error caught by the registry unit tests, never at user runtime.
    pub fn register(&mut self, action: Arc<dyn Action>) {
        let id = action.def().id;
        let previous = self.actions.insert(id, action);
        assert!(previous.is_none(), "duplicate action id '{id}'");
    }

    /// Look up an action.
    #[must_use]
    pub fn get(&self, action_id: &str) -> Option<&Arc<dyn Action>> {
        self.actions.get(action_id)
    }

    /// Every registered def, ordered by id — the payload of `list_actions`.
    #[must_use]
    pub fn defs(&self) -> Vec<ActionDef> {
        self.actions.values().map(|a| a.def()).collect()
    }

    /// Number of registered actions.
    #[must_use]
    pub fn len(&self) -> usize {
        self.actions.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.actions.is_empty()
    }
}

#[async_trait::async_trait]
impl ActionInvoker for Registry {
    fn contains(&self, action_id: &str) -> bool {
        self.actions.contains_key(action_id)
    }

    async fn invoke(
        &self,
        action_id: &str,
        ctx: &mut RunContext,
        input: Content,
    ) -> Result<Content, ActionError> {
        let action = self
            .get(action_id)
            .ok_or_else(|| ActionError::UnknownAction(action_id.to_owned()))?;
        let def = action.def();

        // Effective status = static support ∩ runtime probe (§6.6).
        if matches!(def.supports.for_os(ctx.caps.os), SupportLevel::None) {
            return Err(ActionError::Unsupported {
                os: ctx.caps.os_label(),
                reason: format!("'{action_id}' is not implemented for this OS"),
            });
        }
        if let Some(capability) = def.requires_capability
            && let CapabilityStatus::Unavailable { reason, fix } = ctx.caps.status(capability)
        {
            return Err(match fix {
                Some(CapabilityFix::InstallTool { tool, hint }) => {
                    ActionError::MissingTool { tool, hint }
                }
                Some(CapabilityFix::GrantPermission { permission }) => {
                    ActionError::NeedsOsPermission {
                        permission,
                        hint: "grant it in the system settings".to_owned(),
                    }
                }
                None => ActionError::Unsupported { os: ctx.caps.os_label(), reason },
            });
        }
        if let Some(class) = def.permission {
            ctx.require_permission(class)?;
        }

        action.execute(ctx, input).await
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use crate::{Category, test_util};
    use lightning_core::{ContentKind, PermissionClass};
    use lightning_platform::{
        Capability, CapabilitySnapshot, Os, PlatformSupport,
    };

    struct WindowsOnly;

    #[async_trait::async_trait]
    impl Action for WindowsOnly {
        fn def(&self) -> ActionDef {
            ActionDef::pure("test.windows_only", Category::System, "gear", ContentKind::Nothing)
                .with_supports(PlatformSupport::only(Os::Windows))
        }
        async fn execute(
            &self,
            _ctx: &mut RunContext,
            _input: Content,
        ) -> Result<Content, ActionError> {
            Ok(Content::Nothing)
        }
    }

    struct NeedsInput;

    #[async_trait::async_trait]
    impl Action for NeedsInput {
        fn def(&self) -> ActionDef {
            ActionDef::pure("test.needs_input", Category::Input, "keyboard", ContentKind::Nothing)
                .with_capability(Capability::InputInjection)
                .with_permission(PermissionClass::Input)
        }
        async fn execute(
            &self,
            _ctx: &mut RunContext,
            _input: Content,
        ) -> Result<Content, ActionError> {
            Ok(Content::Nothing)
        }
    }

    #[test]
    fn builtins_register_without_duplicates() {
        let registry = Registry::with_builtins();
        assert!(registry.len() >= 25, "expected a substantial catalog, got {}", registry.len());
        for def in registry.defs() {
            let category_prefix = def.id.split('.').next().unwrap();
            assert!(!category_prefix.is_empty());
            assert!(def.id.contains('.'), "id '{}' must be <category>.<name>", def.id);
        }
    }

    #[tokio::test]
    async fn unsupported_os_yields_the_badge_error() {
        let mut registry = Registry::new();
        registry.register(Arc::new(WindowsOnly));
        // Linux ctx (from test_util) running a Windows-only action.
        let mut ctx = test_util::ctx();
        let err = registry.invoke("test.windows_only", &mut ctx, Content::Nothing).await;
        assert!(matches!(err, Err(ActionError::Unsupported { .. })));
    }

    #[tokio::test]
    async fn wayland_without_ydotool_maps_to_missing_tool() {
        // Fake-environment test required by CLAUDE.md §11.
        let mut registry = Registry::new();
        registry.register(Arc::new(NeedsInput));
        let snapshot = CapabilitySnapshot {
            os: Os::Linux,
            environment: Some("Wayland".into()),
            ..CapabilitySnapshot::unconstrained(Os::Linux)
        }
        .with(
            Capability::InputInjection,
            CapabilityStatus::Unavailable {
                reason: "Wayland session without ydotool".into(),
                fix: Some(CapabilityFix::InstallTool {
                    tool: "ydotool".into(),
                    hint: "install ydotool".into(),
                }),
            },
        );
        let mut ctx = lightning_core::RunContext::new(snapshot)
            .with_permissions([PermissionClass::Input]);
        let err = registry.invoke("test.needs_input", &mut ctx, Content::Nothing).await;
        assert!(matches!(err, Err(ActionError::MissingTool { tool, .. }) if tool == "ydotool"));
    }

    #[tokio::test]
    async fn permission_class_is_enforced() {
        let mut registry = Registry::new();
        registry.register(Arc::new(NeedsInput));
        // Capability available but permission not granted.
        let mut ctx = test_util::ctx();
        let err = registry.invoke("test.needs_input", &mut ctx, Content::Nothing).await;
        assert!(matches!(err, Err(ActionError::PermissionNotGranted { .. })));
    }

    #[tokio::test]
    async fn unknown_action_is_reported() {
        let registry = Registry::with_builtins();
        let mut ctx = test_util::ctx();
        let err = registry.invoke("no.such_action", &mut ctx, Content::Nothing).await;
        assert!(matches!(err, Err(ActionError::UnknownAction(_))));
    }
}
