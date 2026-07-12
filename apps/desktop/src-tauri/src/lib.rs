// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Thin Tauri shell (CLAUDE.md §6.1): setup, command glue, tray, windows.
//! If a function is more than glue, it belongs in a crate.

mod commands;
mod state;
mod tray;

use tauri::Manager;

use crate::state::AppState;

/// Launch the app. `--tray` starts headless: the window stays hidden and
/// triggers keep running (§6.7).
pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let state = match AppState::initialize() {
        Ok(state) => state,
        Err(err) => {
            tracing::error!(%err, "failed to initialize app state");
            std::process::exit(1);
        }
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--tray"]),
        ))
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            commands::list_actions,
            commands::get_capabilities,
            commands::list_shortcuts,
            commands::load_shortcut,
            commands::save_shortcut,
            commands::delete_shortcut,
            commands::run_shortcut,
            commands::cancel_run,
            commands::list_recent_runs,
        ])
        .setup(|app| {
            tray::setup(app)?;
            // Headless tray mode: `lightning --tray` (autostart uses it).
            if std::env::args().any(|arg| arg == "--tray")
                && let Some(window) = app.get_webview_window("main")
            {
                let _ = window.hide();
            }
            // First capability broadcast so badges are correct immediately.
            let state = app.state::<AppState>();
            let snapshot: lightning_ipc_types::CapabilitySnapshotDto =
                (&state.platform.probe()).into();
            use tauri::Emitter;
            let _ = app.emit(lightning_ipc_types::EVENT_CAPABILITY_CHANGED, snapshot);
            Ok(())
        })
        .on_window_event(|window, event| {
            // Closing the window keeps triggers alive; only Quit stops them.
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|err| {
            tracing::error!(%err, "error while running Lightning");
            std::process::exit(1);
        });
}
