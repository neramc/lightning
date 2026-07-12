// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Command glue — thin adapters from the IPC surface (the COMMANDS table in
//! `lightning-ipc-types`) onto the crates. No business logic here.

use lightning_core::{Content, RunContext};
use lightning_ipc_types as ipc;
use lightning_store as store;
use tauri::Emitter;
use uuid::Uuid;

use crate::state::AppState;

type CmdResult<T> = Result<T, String>;

fn err_string(err: impl std::fmt::Display) -> String {
    err.to_string()
}

#[tauri::command]
pub fn list_actions(state: tauri::State<'_, AppState>) -> Vec<ipc::ActionDefDto> {
    state.registry.defs().iter().map(Into::into).collect()
}

#[tauri::command]
pub fn get_capabilities(state: tauri::State<'_, AppState>) -> ipc::CapabilitySnapshotDto {
    (&state.platform.probe()).into()
}

#[tauri::command]
pub fn list_shortcuts(state: tauri::State<'_, AppState>) -> CmdResult<Vec<ipc::ShortcutMetaDto>> {
    let index = state.index.lock().map_err(err_string)?;
    Ok(index
        .list()
        .map_err(err_string)?
        .into_iter()
        .map(|meta| ipc::ShortcutMetaDto {
            id: meta.id.to_string(),
            name: meta.name,
            icon_glyph: meta.icon_glyph,
            gradient: meta.gradient,
            hotkey: meta.hotkey,
            is_automation: meta.is_automation,
        })
        .collect())
}

#[tauri::command]
pub fn load_shortcut(
    state: tauri::State<'_, AppState>,
    id: String,
) -> CmdResult<ipc::ShortcutDto> {
    let path = state.shortcuts_dir.join(format!("{id}.lightning"));
    let shortcut = store::load_shortcut(&path).map_err(err_string)?;
    Ok((&shortcut).into())
}

#[tauri::command]
pub fn save_shortcut(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    shortcut: ipc::ShortcutDto,
) -> CmdResult<ipc::ShortcutMetaDto> {
    let shortcut: lightning_core::Shortcut = shortcut.try_into().map_err(err_string)?;
    let path = store::save_shortcut(&state.shortcuts_dir, &shortcut).map_err(err_string)?;
    {
        let index = state.index.lock().map_err(err_string)?;
        index.upsert(&shortcut, &path).map_err(err_string)?;
    }
    let _ = app.emit(ipc::EVENT_STORE_CHANGED, shortcut.id.to_string());
    Ok(ipc::ShortcutMetaDto {
        id: shortcut.id.to_string(),
        name: shortcut.name.clone(),
        icon_glyph: shortcut.icon.glyph.clone(),
        gradient: shortcut.icon.gradient.clone(),
        hotkey: shortcut.hotkey.clone(),
        is_automation: shortcut.is_automation(),
    })
}

#[tauri::command]
pub fn delete_shortcut(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
) -> CmdResult<()> {
    let uuid: Uuid = id.parse().map_err(err_string)?;
    store::delete_shortcut_file(&state.shortcuts_dir, uuid).map_err(err_string)?;
    {
        let index = state.index.lock().map_err(err_string)?;
        index.remove(uuid).map_err(err_string)?;
    }
    let _ = app.emit(ipc::EVENT_STORE_CHANGED, id);
    Ok(())
}

#[tauri::command]
pub async fn run_shortcut(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
    id: String,
) -> CmdResult<ipc::RunResultDto> {
    let path = state.shortcuts_dir.join(format!("{id}.lightning"));
    let shortcut = store::load_shortcut(&path).map_err(err_string)?;
    let shortcut_id: Uuid = id.parse().map_err(err_string)?;

    // Wire run://progress events to the webview (§9.3).
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut ctx = RunContext::new(state.platform.probe()).with_progress(tx);
    let run_id = ctx.run_id;
    let cancel = ctx.cancellation();
    state
        .running
        .lock()
        .map_err(err_string)?
        .insert(run_id, cancel);
    let forwarder = {
        let app = app.clone();
        tokio::spawn(async move {
            while let Some(progress) = rx.recv().await {
                let (phase, preview, message) = match &progress.phase {
                    lightning_core::StepPhase::Started => ("started", None, None),
                    lightning_core::StepPhase::Finished { preview } => {
                        ("finished", Some(preview.clone()), None)
                    }
                    lightning_core::StepPhase::Failed { message } => {
                        ("failed", None, Some(message.clone()))
                    }
                    lightning_core::StepPhase::Skipped { message } => {
                        ("skipped", None, Some(message.clone()))
                    }
                };
                let _ = app.emit(
                    ipc::EVENT_RUN_PROGRESS,
                    ipc::RunProgressDto {
                        run_id: progress.run_id.to_string(),
                        step: progress.step.to_string(),
                        action_id: progress.action_id.clone(),
                        phase: phase.to_owned(),
                        preview,
                        message,
                    },
                );
            }
        })
    };

    let started_at = chrono::Utc::now();
    let outcome = state.engine.run(&shortcut, &mut ctx).await;
    let duration_ms = (chrono::Utc::now() - started_at).num_milliseconds();
    state.running.lock().map_err(err_string)?.remove(&run_id);
    forwarder.abort();

    let (status, output, error) = match &outcome {
        Ok(output) => ("success", output.clone(), None),
        Err(lightning_core::ActionError::Cancelled) => {
            ("cancelled", Content::Nothing, Some("cancelled".to_owned()))
        }
        Err(err) => ("error", Content::Nothing, Some(err.to_string())),
    };
    {
        let index = state.index.lock().map_err(err_string)?;
        let _ = index.record_run(&lightning_store::RunRecord {
            shortcut_id,
            started_at,
            duration_ms,
            status: match status {
                "success" => lightning_store::RunStatus::Success,
                "cancelled" => lightning_store::RunStatus::Cancelled,
                _ => lightning_store::RunStatus::Error,
            },
            error: error.clone(),
        });
    }

    Ok(ipc::RunResultDto {
        run_id: run_id.to_string(),
        status: status.to_owned(),
        output: (&output).into(),
        error,
        log: ctx.log.entries().iter().map(Into::into).collect(),
    })
}

#[tauri::command]
pub fn cancel_run(state: tauri::State<'_, AppState>, run_id: String) -> CmdResult<()> {
    let run_id: Uuid = run_id.parse().map_err(err_string)?;
    if let Some(token) = state.running.lock().map_err(err_string)?.get(&run_id) {
        token.cancel();
    }
    Ok(())
}

#[tauri::command]
pub fn list_recent_runs(
    state: tauri::State<'_, AppState>,
    id: String,
    limit: u32,
) -> CmdResult<Vec<ipc::RunRecordDto>> {
    let uuid: Uuid = id.parse().map_err(err_string)?;
    let index = state.index.lock().map_err(err_string)?;
    Ok(index
        .recent_runs(uuid, limit)
        .map_err(err_string)?
        .into_iter()
        .map(|run| ipc::RunRecordDto {
            shortcut_id: run.shortcut_id.to_string(),
            started_at: run.started_at.to_rfc3339(),
            duration_ms: run.duration_ms as f64,
            status: match run.status {
                lightning_store::RunStatus::Success => "success".to_owned(),
                lightning_store::RunStatus::Error => "error".to_owned(),
                lightning_store::RunStatus::Cancelled => "cancelled".to_owned(),
            },
            error: run.error,
        })
        .collect())
}
