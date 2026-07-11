// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Binary entry point. Everything real lives in the library crate so the
//! shell stays thin (CLAUDE.md §6.1).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    lightning_desktop::run();
}
