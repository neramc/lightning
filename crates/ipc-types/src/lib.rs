// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! # lightning-ipc-types
//!
//! specta-annotated DTOs, command signatures, and event names — the **single
//! source of truth** for the IPC boundary (CLAUDE.md §6.8).
//!
//! `pnpm bindings` runs the `export-bindings` bin (feature `export`) which
//! writes `packages/bindings/src/index.ts`. Never hand-write invoke strings
//! or duplicate these types in TS; CI fails on stale bindings.
//!
//! Payloads stay backward-compatible within a minor version — updater flows
//! can briefly mismatch frontend/backend.

#![deny(missing_docs)]

use std::collections::BTreeMap;

use lightning_actions::{ActionDef, ParamDef, ParamKind};
use lightning_core::{
    Branch, Content, ErrorPolicy, Icon, LogLevel, ParamValue, RunLogEntry, Shortcut, Step,
    TriggerConfig,
};
use lightning_platform::{
    CapabilityFix, CapabilitySnapshot, CapabilityStatus, PlatformSupport, SupportLevel,
};
use serde::{Deserialize, Serialize};

// ── event names (§6.8) ─────────────────────────────────────────────────────

/// Step progress during a run; payload [`RunProgressDto`].
pub const EVENT_RUN_PROGRESS: &str = "run://progress";
/// A trigger fired; payload is the trigger event JSON.
pub const EVENT_TRIGGERS_FIRED: &str = "triggers://fired";
/// The capability snapshot changed; payload [`CapabilitySnapshotDto`].
pub const EVENT_CAPABILITY_CHANGED: &str = "capability://changed";
/// Shortcut files changed on disk; payload is the affected shortcut id.
pub const EVENT_STORE_CHANGED: &str = "store://changed";

// ── content & shortcut DTOs ────────────────────────────────────────────────

/// Typed content value crossing the IPC boundary (mirror of
/// `lightning_core::Content`; dates and paths travel as strings).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "type", content = "value", rename_all = "camelCase")]
#[allow(missing_docs)]
pub enum ContentDto {
    Nothing,
    Text(String),
    Number(f64),
    Boolean(bool),
    Date(String),
    List(Vec<ContentDto>),
    Dictionary(BTreeMap<String, ContentDto>),
    File(String),
    Image(String),
    Url(String),
    RichText { html: String, plain: String },
    Error { message: String },
}

impl From<&Content> for ContentDto {
    fn from(value: &Content) -> Self {
        match value {
            Content::Nothing => ContentDto::Nothing,
            Content::Text(s) => ContentDto::Text(s.clone()),
            Content::Number(n) => ContentDto::Number(*n),
            Content::Boolean(b) => ContentDto::Boolean(*b),
            Content::Date(d) => ContentDto::Date(d.to_rfc3339()),
            Content::List(items) => ContentDto::List(items.iter().map(Into::into).collect()),
            Content::Dictionary(map) => {
                ContentDto::Dictionary(map.iter().map(|(k, v)| (k.clone(), v.into())).collect())
            }
            Content::File(p) => ContentDto::File(p.display().to_string()),
            Content::Image(p) => ContentDto::Image(p.display().to_string()),
            Content::Url(u) => ContentDto::Url(u.clone()),
            Content::RichText { html, plain } => ContentDto::RichText {
                html: html.clone(),
                plain: plain.clone(),
            },
            Content::Error { message } => ContentDto::Error {
                message: message.clone(),
            },
        }
    }
}

impl TryFrom<ContentDto> for Content {
    type Error = String;

    fn try_from(value: ContentDto) -> Result<Self, String> {
        Ok(match value {
            ContentDto::Nothing => Content::Nothing,
            ContentDto::Text(s) => Content::Text(s),
            ContentDto::Number(n) => Content::Number(n),
            ContentDto::Boolean(b) => Content::Boolean(b),
            ContentDto::Date(s) => Content::Date(
                chrono::DateTime::parse_from_rfc3339(&s)
                    .map_err(|e| format!("invalid date: {e}"))?
                    .with_timezone(&chrono::Utc),
            ),
            ContentDto::List(items) => Content::List(
                items
                    .into_iter()
                    .map(Content::try_from)
                    .collect::<Result<_, _>>()?,
            ),
            ContentDto::Dictionary(map) => Content::Dictionary(
                map.into_iter()
                    .map(|(k, v)| Content::try_from(v).map(|v| (k, v)))
                    .collect::<Result<_, _>>()?,
            ),
            ContentDto::File(p) => Content::File(p.into()),
            ContentDto::Image(p) => Content::Image(p.into()),
            ContentDto::Url(u) => Content::Url(u),
            ContentDto::RichText { html, plain } => Content::RichText { html, plain },
            ContentDto::Error { message } => Content::Error { message },
        })
    }
}

/// Mirror of `lightning_core::ParamValue`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[allow(missing_docs)]
pub enum ParamValueDto {
    Literal { value: ContentDto },
    Variable { name: String },
    MagicOutput { step: String },
    Template { template: String },
}

/// Mirror of `lightning_core::ErrorPolicy`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(
    tag = "policy",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[allow(missing_docs)]
pub enum ErrorPolicyDto {
    Stop,
    Continue,
    Retry { attempts: u32, backoff_ms: f64 },
}

/// Mirror of `lightning_core::Branch`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct BranchDto {
    /// Branch label (`then` / `otherwise` / `body` / menu option).
    pub label: String,
    /// Child steps.
    pub steps: Vec<StepDto>,
}

/// Mirror of `lightning_core::Step`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct StepDto {
    /// Step uuid (magic-variable identity).
    pub uuid: String,
    /// Action id.
    pub action_id: String,
    /// Parameter values.
    pub params: BTreeMap<String, ParamValueDto>,
    /// Error policy.
    pub error_policy: ErrorPolicyDto,
    /// Control-flow children.
    pub branches: Vec<BranchDto>,
}

/// Mirror of `lightning_core::Icon`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct IconDto {
    /// Tile glyph.
    pub glyph: String,
    /// Gradient token (design-system id, never a hex value).
    pub gradient: String,
}

/// Mirror of `lightning_core::TriggerConfig`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct TriggerConfigDto {
    /// Trigger id.
    pub trigger_id: String,
    /// Trigger-specific config JSON.
    pub config: serde_json::Value,
    /// Whether the automation is armed.
    pub enabled: bool,
    /// Cooldown override in milliseconds.
    pub cooldown_ms: Option<f64>,
}

/// Full shortcut document crossing the boundary (editor load/save).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutDto {
    /// Schema version.
    pub schema_version: u32,
    /// Shortcut id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Tile icon.
    pub icon: IconDto,
    /// Global hotkey.
    pub hotkey: Option<String>,
    /// The flow.
    pub steps: Vec<StepDto>,
    /// Automation trigger block.
    pub trigger: Option<TriggerConfigDto>,
}

impl From<&Shortcut> for ShortcutDto {
    fn from(s: &Shortcut) -> Self {
        fn step(s: &Step) -> StepDto {
            StepDto {
                uuid: s.uuid.to_string(),
                action_id: s.action_id.clone(),
                params: s
                    .params
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            match v {
                                ParamValue::Literal { value } => ParamValueDto::Literal {
                                    value: value.into(),
                                },
                                ParamValue::Variable { name } => {
                                    ParamValueDto::Variable { name: name.clone() }
                                }
                                ParamValue::MagicOutput { step } => ParamValueDto::MagicOutput {
                                    step: step.to_string(),
                                },
                                ParamValue::Template { template } => ParamValueDto::Template {
                                    template: template.clone(),
                                },
                            },
                        )
                    })
                    .collect(),
                error_policy: match &s.error_policy {
                    ErrorPolicy::Stop => ErrorPolicyDto::Stop,
                    ErrorPolicy::Continue => ErrorPolicyDto::Continue,
                    ErrorPolicy::Retry {
                        attempts,
                        backoff_ms,
                    } => ErrorPolicyDto::Retry {
                        attempts: *attempts,
                        backoff_ms: *backoff_ms as f64,
                    },
                },
                branches: s
                    .branches
                    .iter()
                    .map(|b| BranchDto {
                        label: b.label.clone(),
                        steps: b.steps.iter().map(step).collect(),
                    })
                    .collect(),
            }
        }
        ShortcutDto {
            schema_version: s.schema_version,
            id: s.id.to_string(),
            name: s.name.clone(),
            description: s.description.clone(),
            icon: IconDto {
                glyph: s.icon.glyph.clone(),
                gradient: s.icon.gradient.clone(),
            },
            hotkey: s.hotkey.clone(),
            steps: s.steps.iter().map(step).collect(),
            trigger: s.trigger.as_ref().map(|t| TriggerConfigDto {
                trigger_id: t.trigger_id.clone(),
                config: t.config.clone(),
                enabled: t.enabled,
                cooldown_ms: t.cooldown_ms.map(|v| v as f64),
            }),
        }
    }
}

impl TryFrom<ShortcutDto> for Shortcut {
    type Error = String;

    fn try_from(dto: ShortcutDto) -> Result<Self, String> {
        fn step(dto: StepDto) -> Result<Step, String> {
            Ok(Step {
                uuid: dto
                    .uuid
                    .parse()
                    .map_err(|e| format!("invalid step uuid: {e}"))?,
                action_id: dto.action_id,
                params: dto
                    .params
                    .into_iter()
                    .map(|(k, v)| {
                        Ok((
                            k,
                            match v {
                                ParamValueDto::Literal { value } => ParamValue::Literal {
                                    value: value.try_into()?,
                                },
                                ParamValueDto::Variable { name } => ParamValue::Variable { name },
                                ParamValueDto::MagicOutput { step } => ParamValue::MagicOutput {
                                    step: step
                                        .parse()
                                        .map_err(|e| format!("invalid magic ref: {e}"))?,
                                },
                                ParamValueDto::Template { template } => {
                                    ParamValue::Template { template }
                                }
                            },
                        ))
                    })
                    .collect::<Result<_, String>>()?,
                error_policy: match dto.error_policy {
                    ErrorPolicyDto::Stop => ErrorPolicy::Stop,
                    ErrorPolicyDto::Continue => ErrorPolicy::Continue,
                    ErrorPolicyDto::Retry {
                        attempts,
                        backoff_ms,
                    } => ErrorPolicy::Retry {
                        attempts,
                        backoff_ms: backoff_ms as u64,
                    },
                },
                branches: dto
                    .branches
                    .into_iter()
                    .map(|b| {
                        Ok(Branch {
                            label: b.label,
                            steps: b
                                .steps
                                .into_iter()
                                .map(step)
                                .collect::<Result<_, String>>()?,
                        })
                    })
                    .collect::<Result<_, String>>()?,
            })
        }
        Ok(Shortcut {
            schema_version: dto.schema_version,
            id: dto
                .id
                .parse()
                .map_err(|e| format!("invalid shortcut id: {e}"))?,
            name: dto.name,
            description: dto.description,
            icon: Icon {
                glyph: dto.icon.glyph,
                gradient: dto.icon.gradient,
            },
            hotkey: dto.hotkey,
            steps: dto
                .steps
                .into_iter()
                .map(step)
                .collect::<Result<_, String>>()?,
            trigger: dto.trigger.map(|t| TriggerConfig {
                trigger_id: t.trigger_id,
                config: t.config,
                enabled: t.enabled,
                cooldown_ms: t.cooldown_ms.map(|v| v as u64),
            }),
        })
    }
}

// ── catalog & capability DTOs ──────────────────────────────────────────────

/// One support level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct SupportLevelDto {
    /// `full` · `partial` · `none`.
    pub level: String,
    /// Note for `partial`.
    pub note: Option<String>,
}

impl From<&SupportLevel> for SupportLevelDto {
    fn from(level: &SupportLevel) -> Self {
        match level {
            SupportLevel::Full => Self {
                level: "full".into(),
                note: None,
            },
            SupportLevel::Partial { note } => Self {
                level: "partial".into(),
                note: Some(note.clone()),
            },
            SupportLevel::None => Self {
                level: "none".into(),
                note: None,
            },
        }
    }
}

/// Per-OS support matrix.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
#[allow(missing_docs)]
pub struct PlatformSupportDto {
    pub windows: SupportLevelDto,
    pub macos: SupportLevelDto,
    pub linux: SupportLevelDto,
    pub freebsd: SupportLevelDto,
}

impl From<&PlatformSupport> for PlatformSupportDto {
    fn from(s: &PlatformSupport) -> Self {
        Self {
            windows: (&s.windows).into(),
            macos: (&s.macos).into(),
            linux: (&s.linux).into(),
            freebsd: (&s.freebsd).into(),
        }
    }
}

/// One parameter schema entry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ParamDefDto {
    /// Param key (i18n: `actions.<id>.params.<key>`).
    pub key: String,
    /// `text` · `number` · `boolean` · `date` · `file` · `enum`.
    pub kind: String,
    /// Options when `kind == "enum"`.
    pub options: Option<Vec<String>>,
    /// Whether required.
    pub required: bool,
}

impl From<&ParamDef> for ParamDefDto {
    fn from(p: &ParamDef) -> Self {
        let (kind, options) = match &p.kind {
            ParamKind::Text => ("text", None),
            ParamKind::Number => ("number", None),
            ParamKind::Boolean => ("boolean", None),
            ParamKind::Date => ("date", None),
            ParamKind::File => ("file", None),
            ParamKind::Enum(options) => (
                "enum",
                Some(options.iter().map(|s| (*s).to_owned()).collect()),
            ),
        };
        Self {
            key: p.key.to_owned(),
            kind: kind.to_owned(),
            options,
            required: p.required,
        }
    }
}

/// One catalog entry — everything the editor renders, as data (§6.5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ActionDefDto {
    /// Action id.
    pub id: String,
    /// Category id (gradient token + i18n segment).
    pub category: String,
    /// Icon token.
    pub icon: String,
    /// Param schema.
    pub params: Vec<ParamDefDto>,
    /// Output kind id, `null` when dynamic.
    pub output: Option<String>,
    /// Support matrix.
    pub supports: PlatformSupportDto,
    /// Permission class id, if any.
    pub permission: Option<String>,
    /// Required runtime capability id, if any.
    pub requires_capability: Option<String>,
    /// Param holding reviewable script text, if any.
    pub script_param: Option<String>,
}

impl From<&ActionDef> for ActionDefDto {
    fn from(def: &ActionDef) -> Self {
        Self {
            id: def.id.to_owned(),
            category: def.category.id().to_owned(),
            icon: def.icon.to_owned(),
            params: def.params.iter().map(Into::into).collect(),
            output: def.output.map(|kind| kind.to_string()),
            supports: (&def.supports).into(),
            permission: def.permission.map(|p| p.id().to_owned()),
            requires_capability: def.requires_capability.map(|c| format!("{c:?}")),
            script_param: def.script_param.map(str::to_owned),
        }
    }
}

/// One probed capability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CapabilityStatusDto {
    /// Capability id.
    pub capability: String,
    /// `available` · `degraded` · `unavailable`.
    pub status: String,
    /// Technical reason for degraded/unavailable.
    pub reason: Option<String>,
    /// "Fix it": tool to install.
    pub fix_tool: Option<String>,
    /// "Fix it": install hint.
    pub fix_hint: Option<String>,
    /// "Fix it": OS permission to grant.
    pub fix_permission: Option<String>,
}

/// The runtime capability snapshot (payload of `capability://changed`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitySnapshotDto {
    /// OS id (`windows` · `macos` · `linux` · `freebsd`).
    pub os: String,
    /// Environment refinement (e.g. `Wayland`).
    pub environment: Option<String>,
    /// The `{{os}}` label for `action.unsupportedOnOs` (§8.1).
    pub os_label: String,
    /// Probed constraints.
    pub capabilities: Vec<CapabilityStatusDto>,
}

impl From<&CapabilitySnapshot> for CapabilitySnapshotDto {
    fn from(snapshot: &CapabilitySnapshot) -> Self {
        Self {
            os: format!("{:?}", snapshot.os).to_lowercase(),
            environment: snapshot.environment.clone(),
            os_label: snapshot.os_label(),
            capabilities: snapshot
                .capabilities
                .iter()
                .map(|(capability, status)| {
                    let (status_id, reason, fix) = match status {
                        CapabilityStatus::Available => ("available", None, None),
                        CapabilityStatus::Degraded { reason } => {
                            ("degraded", Some(reason.clone()), None)
                        }
                        CapabilityStatus::Unavailable { reason, fix } => {
                            ("unavailable", Some(reason.clone()), fix.clone())
                        }
                    };
                    let (fix_tool, fix_hint, fix_permission) = match fix {
                        Some(CapabilityFix::InstallTool { tool, hint }) => {
                            (Some(tool), Some(hint), None)
                        }
                        Some(CapabilityFix::GrantPermission { permission }) => {
                            (None, None, Some(permission))
                        }
                        None => (None, None, None),
                    };
                    CapabilityStatusDto {
                        capability: format!("{capability:?}"),
                        status: status_id.to_owned(),
                        reason,
                        fix_tool,
                        fix_hint,
                        fix_permission,
                    }
                })
                .collect(),
        }
    }
}

// ── run & history DTOs ─────────────────────────────────────────────────────

/// One `run://progress` event.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RunProgressDto {
    /// Run id.
    pub run_id: String,
    /// Step uuid.
    pub step: String,
    /// Action id (drives the block animation without a lookup).
    pub action_id: String,
    /// `started` · `finished` · `failed` · `skipped`.
    pub phase: String,
    /// Output preview for `finished`.
    pub preview: Option<String>,
    /// Error message for `failed` / `skipped`.
    pub message: Option<String>,
}

/// One user-visible run log line.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RunLogEntryDto {
    /// RFC3339 timestamp.
    pub at: String,
    /// Step uuid, if step-scoped.
    pub step: Option<String>,
    /// `info` · `warn` · `error`.
    pub level: String,
    /// Message.
    pub message: String,
}

impl From<&RunLogEntry> for RunLogEntryDto {
    fn from(entry: &RunLogEntry) -> Self {
        Self {
            at: entry.at.to_rfc3339(),
            step: entry.step.map(|s| s.to_string()),
            level: match entry.level {
                LogLevel::Info => "info".into(),
                LogLevel::Warn => "warn".into(),
                LogLevel::Error => "error".into(),
            },
            message: entry.message.clone(),
        }
    }
}

/// Result of a completed (or failed) run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RunResultDto {
    /// Run id.
    pub run_id: String,
    /// `success` · `error` · `cancelled`.
    pub status: String,
    /// Final output.
    pub output: ContentDto,
    /// Error message when failed.
    pub error: Option<String>,
    /// The collected run log.
    pub log: Vec<RunLogEntryDto>,
}

/// Indexed shortcut metadata (grid tiles, palette).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct ShortcutMetaDto {
    /// Shortcut id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Tile glyph.
    pub icon_glyph: String,
    /// Gradient token.
    pub gradient: String,
    /// Hotkey, if assigned.
    pub hotkey: Option<String>,
    /// Whether it is an automation.
    pub is_automation: bool,
}

/// One run-history row.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct RunRecordDto {
    /// Shortcut id.
    pub shortcut_id: String,
    /// RFC3339 start time.
    pub started_at: String,
    /// Duration in milliseconds.
    pub duration_ms: f64,
    /// `success` · `error` · `cancelled`.
    pub status: String,
    /// Error message when failed.
    pub error: Option<String>,
}

// ── command table (consumed by export-bindings) ────────────────────────────

/// One Tauri command signature (name + typed args + result TS type).
pub struct CommandSig {
    /// Command name as registered in the Tauri builder.
    pub name: &'static str,
    /// `(arg_name, ts_type)` pairs.
    pub args: &'static [(&'static str, &'static str)],
    /// TS result type.
    pub result: &'static str,
}

/// Every command the shell exposes. The desktop shell registers handlers for
/// exactly this set; the generated bindings wrap exactly this set.
pub const COMMANDS: &[CommandSig] = &[
    CommandSig {
        name: "list_actions",
        args: &[],
        result: "ActionDefDto[]",
    },
    CommandSig {
        name: "get_capabilities",
        args: &[],
        result: "CapabilitySnapshotDto",
    },
    CommandSig {
        name: "list_shortcuts",
        args: &[],
        result: "ShortcutMetaDto[]",
    },
    CommandSig {
        name: "load_shortcut",
        args: &[("id", "string")],
        result: "ShortcutDto",
    },
    CommandSig {
        name: "save_shortcut",
        args: &[("shortcut", "ShortcutDto")],
        result: "ShortcutMetaDto",
    },
    CommandSig {
        name: "delete_shortcut",
        args: &[("id", "string")],
        result: "null",
    },
    CommandSig {
        name: "run_shortcut",
        args: &[("id", "string")],
        result: "RunResultDto",
    },
    CommandSig {
        name: "cancel_run",
        args: &[("runId", "string")],
        result: "null",
    },
    CommandSig {
        name: "list_recent_runs",
        args: &[("id", "string"), ("limit", "number")],
        result: "RunRecordDto[]",
    },
];

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use lightning_core::Step;

    #[test]
    fn shortcut_dto_round_trip() {
        let mut shortcut = Shortcut::new("DTO");
        shortcut
            .steps
            .push(Step::new("text.text").with_param("text", Content::Text("hello".into())));
        let dto = ShortcutDto::from(&shortcut);
        let back = Shortcut::try_from(dto).unwrap();
        assert_eq!(back, shortcut);
    }

    #[test]
    fn action_def_dto_maps_enum_options() {
        let registry = lightning_actions::Registry::with_builtins();
        let defs: Vec<ActionDefDto> = registry.defs().iter().map(Into::into).collect();
        let change_case = defs.iter().find(|d| d.id == "text.change_case").unwrap();
        let case_param = change_case.params.iter().find(|p| p.key == "case").unwrap();
        assert_eq!(case_param.kind, "enum");
        assert!(
            case_param
                .options
                .as_ref()
                .unwrap()
                .contains(&"uppercase".to_owned())
        );
    }

    #[test]
    fn capability_snapshot_dto_carries_the_fix() {
        use lightning_platform::{Capability, Os};
        let snapshot = CapabilitySnapshot::unconstrained(Os::Linux).with(
            Capability::InputInjection,
            CapabilityStatus::Unavailable {
                reason: "Wayland without ydotool".into(),
                fix: Some(CapabilityFix::InstallTool {
                    tool: "ydotool".into(),
                    hint: "install it".into(),
                }),
            },
        );
        let dto = CapabilitySnapshotDto::from(&snapshot);
        let entry = dto
            .capabilities
            .iter()
            .find(|c| c.capability == "InputInjection")
            .unwrap();
        assert_eq!(entry.status, "unavailable");
        assert_eq!(entry.fix_tool.as_deref(), Some("ydotool"));
    }
}
