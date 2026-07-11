// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Schema round-trip and migration tests with insta snapshots
//! (CLAUDE.md §11). The snapshot pins the on-disk `.lightning` shape — a
//! diff here means a schema change, which requires a version bump plus a
//! migration (§2.6).

#![allow(clippy::unwrap_used)]

use lightning_core::{Content, Icon, ParamValue, Shortcut, Step, TriggerConfig};
use uuid::Uuid;

/// A deterministic shortcut exercising every serialized construct.
fn representative_shortcut() -> Shortcut {
    let mut step = Step::new("text.change_case");
    step.uuid = Uuid::from_u128(0xA1);
    step.params.insert(
        "case".into(),
        ParamValue::Literal {
            value: Content::Text("uppercase".into()),
        },
    );
    step.params.insert(
        "source".into(),
        ParamValue::Variable {
            name: "greeting".into(),
        },
    );
    step.params.insert(
        "note".into(),
        ParamValue::Template {
            template: "was {{greeting}}".into(),
        },
    );

    let mut condition = Step::new("control.if");
    condition.uuid = Uuid::from_u128(0xB1);
    condition.params.insert(
        "op".into(),
        ParamValue::Literal {
            value: Content::Text("hasValue".into()),
        },
    );
    let mut then_step = Step::new("control.nothing");
    then_step.uuid = Uuid::from_u128(0xB2);
    then_step.error_policy = lightning_core::ErrorPolicy::Retry {
        attempts: 2,
        backoff_ms: 250,
    };
    let condition = condition
        .with_branch("then", vec![then_step])
        .with_branch("otherwise", vec![]);

    Shortcut {
        schema_version: lightning_core::CURRENT_SCHEMA_VERSION,
        id: Uuid::from_u128(1),
        name: "Representative".into(),
        description: "Pins the .lightning schema".into(),
        icon: Icon {
            glyph: "⚡".into(),
            gradient: "scripting".into(),
        },
        hotkey: Some("Ctrl+Shift+L".into()),
        steps: vec![step, condition],
        trigger: Some(TriggerConfig {
            trigger_id: "trigger.interval".into(),
            config: serde_json::json!({ "seconds": 300 }),
            enabled: true,
            cooldown_ms: Some(2000),
        }),
    }
}

#[test]
fn lightning_file_shape_is_pinned() {
    let json = representative_shortcut().to_pretty_json().unwrap();
    insta::assert_snapshot!("representative_lightning_file", json);
}

#[test]
fn round_trip_is_lossless() {
    let original = representative_shortcut();
    let json = original.to_pretty_json().unwrap();
    let reparsed = Shortcut::from_json_str(&json).unwrap();
    assert_eq!(reparsed, original);
}

#[test]
fn current_version_files_open_without_migration() {
    let json = representative_shortcut().to_pretty_json().unwrap();
    let shortcut = Shortcut::from_json_str(&json).unwrap();
    assert_eq!(
        shortcut.schema_version,
        lightning_core::CURRENT_SCHEMA_VERSION
    );
}

#[test]
fn future_version_files_are_rejected_with_a_clear_error() {
    let mut value: serde_json::Value = serde_json::to_value(representative_shortcut()).unwrap();
    value["schemaVersion"] = serde_json::json!(999);
    let err = Shortcut::from_json_str(&value.to_string()).unwrap_err();
    assert!(err.to_string().contains("999"));
}

#[test]
fn minimal_documents_get_defaults() {
    // Only the required fields — everything else must default, so files
    // written by older minor versions keep opening.
    let json = r#"{
        "schemaVersion": 1,
        "id": "00000000-0000-0000-0000-000000000009",
        "name": "Minimal"
    }"#;
    let shortcut = Shortcut::from_json_str(json).unwrap();
    assert_eq!(shortcut.steps.len(), 0);
    assert!(shortcut.trigger.is_none());
    assert_eq!(shortcut.icon.glyph, "⚡");
}
