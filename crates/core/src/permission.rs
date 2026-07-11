// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Permission classes (CLAUDE.md §14) — declared per action, granted per
//! shortcut, remembered (except `elevated`), revocable in Settings → Privacy.

use serde::{Deserialize, Serialize};

/// A class of dangerous behavior an action may perform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PermissionClass {
    /// Reading files outside the app's own data directory.
    FsRead,
    /// Writing/deleting files outside the app's own data directory.
    FsWrite,
    /// Running any script or OS bridge (shell, PowerShell, AppleScript…).
    Shell,
    /// Keystroke / mouse injection.
    Input,
    /// Network requests.
    Network,
    /// Changing system settings (Wi-Fi, power, registry, defaults…).
    SystemSettings,
    /// Camera / microphone / location.
    Device,
    /// Admin/root operations — always re-prompts, never remembered.
    Elevated,
}

impl PermissionClass {
    /// Whether a grant may be remembered across runs. `Elevated` never is.
    #[must_use]
    pub const fn is_rememberable(self) -> bool {
        !matches!(self, PermissionClass::Elevated)
    }

    /// Stable kebab-case id used in settings files and i18n keys.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            PermissionClass::FsRead => "fs-read",
            PermissionClass::FsWrite => "fs-write",
            PermissionClass::Shell => "shell",
            PermissionClass::Input => "input",
            PermissionClass::Network => "network",
            PermissionClass::SystemSettings => "system-settings",
            PermissionClass::Device => "device",
            PermissionClass::Elevated => "elevated",
        }
    }
}

impl std::fmt::Display for PermissionClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.id())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn elevated_is_never_remembered() {
        assert!(!PermissionClass::Elevated.is_rememberable());
        assert!(PermissionClass::Shell.is_rememberable());
    }

    #[test]
    fn serde_uses_kebab_case_ids() {
        let json = serde_json::to_string(&PermissionClass::SystemSettings).expect("serialize");
        assert_eq!(json, "\"system-settings\"");
    }
}
