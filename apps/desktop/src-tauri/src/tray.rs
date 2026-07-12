// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Tray icon + menu. Closing the window keeps triggers alive; only Quit
//! stops them (§6.7). Labels come from packages/i18n (§2.9) — the locale
//! files are embedded at compile time and selected by the user's setting.

use tauri::Manager;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::TrayIconBuilder;

const EN_COMMON: &str = include_str!("../../../../packages/i18n/locales/en/common.json");
const KO_COMMON: &str = include_str!("../../../../packages/i18n/locales/ko/common.json");

fn tray_label(locale: &str, key: &str) -> String {
    let source = if locale.starts_with("ko") {
        KO_COMMON
    } else {
        EN_COMMON
    };
    serde_json::from_str::<serde_json::Value>(source)
        .ok()
        .and_then(|v| v["tray"][key].as_str().map(str::to_owned))
        .unwrap_or_else(|| key.to_owned())
}

fn user_locale() -> String {
    // The UI-facing subset persists via the frontend; the tray falls back to
    // the OS locale env until the settings bridge lands.
    std::env::var("LANG").unwrap_or_default()
}

pub fn setup(app: &tauri::App) -> tauri::Result<()> {
    let locale = user_locale();
    let open = MenuItem::with_id(app, "open", tray_label(&locale, "open"), true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", tray_label(&locale, "quit"), true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&open, &quit])?;

    TrayIconBuilder::with_id("main")
        .icon(
            app.default_window_icon()
                .cloned()
                .ok_or(tauri::Error::WindowNotFound)?,
        )
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "quit" => app.exit(0),
            "open" => {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
            _ => {}
        })
        .build(app)?;
    Ok(())
}
