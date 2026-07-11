// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Runtime capability probing results (CLAUDE.md §6.6).
//!
//! On startup and on relevant system changes each platform implementation
//! produces a [`CapabilitySnapshot`]. The engine intersects it with every
//! action's static [`PlatformSupport`](crate::PlatformSupport); the frontend
//! receives it over the `capability://changed` event so badges update live.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::support::Os;

/// A probeable runtime capability that actions may require.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Capability {
    /// Synthetic keyboard/mouse events (X11 native, Wayland via ydotool/libei,
    /// macOS via Accessibility permission, Windows via SendInput).
    InputInjection,
    /// Screen/window/region capture (macOS Screen Recording permission,
    /// Wayland via the screenshot portal).
    Screenshot,
    /// Reading the system clipboard.
    ClipboardRead,
    /// Writing the system clipboard.
    ClipboardWrite,
    /// Desktop notifications.
    Notifications,
    /// Wi-Fi / network control (Linux requires NetworkManager).
    NetworkControl,
    /// Media player control (Linux via MPRIS, Windows via SMTC).
    MediaControl,
    /// Global hotkey registration (limited on Wayland).
    GlobalHotkeys,
    /// Moving/resizing/focusing other apps' windows.
    WindowManagement,
    /// Text-to-speech (Linux requires speech-dispatcher).
    SpeechSynthesis,
}

/// Probe outcome for one capability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum CapabilityStatus {
    /// Works right now.
    Available,
    /// Works with caveats (e.g. "screenshots go through the xdg portal").
    Degraded {
        /// Technical explanation of the caveat.
        reason: String,
    },
    /// Cannot work in the current environment. `reason` is the technical
    /// detail; the user-facing message is the localized
    /// `action.unsupportedOnOs` / `action.needsTool` / `action.needsPermission`
    /// string chosen by the UI from `fix` (§8.1).
    Unavailable {
        /// Technical explanation (e.g. "Wayland session without ydotool").
        reason: String,
        /// How to fix it, if this is a solvable setup issue rather than an OS
        /// limit — never blame the OS for a solvable problem.
        fix: Option<CapabilityFix>,
    },
}

/// A user-actionable remedy surfaced as the "Fix it" button.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum CapabilityFix {
    /// A missing OS permission; the UI deep-links into OS settings.
    GrantPermission {
        /// Permission name shown to the user (i18n key suffix).
        permission: String,
    },
    /// A missing external tool; the UI shows an install hint.
    InstallTool {
        /// Tool binary name (e.g. `ydotool`).
        tool: String,
        /// Install hint (e.g. "sudo apt install ydotool").
        hint: String,
    },
}

/// The full probe result for the current machine.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitySnapshot {
    /// The running OS.
    pub os: Os,
    /// Environment refinement when the limits are environmental, e.g.
    /// `"Wayland"` / `"X11"` on Linux. Rendered as `Linux (Wayland)`.
    pub environment: Option<String>,
    /// Probed capabilities. Missing entries mean "no known constraint".
    pub capabilities: BTreeMap<Capability, CapabilityStatus>,
}

impl CapabilitySnapshot {
    /// A snapshot with no known constraints for `os`.
    #[must_use]
    pub fn unconstrained(os: Os) -> Self {
        Self {
            os,
            environment: None,
            capabilities: BTreeMap::new(),
        }
    }

    /// Status for a capability; absent entries are [`CapabilityStatus::Available`].
    #[must_use]
    pub fn status(&self, capability: Capability) -> CapabilityStatus {
        self.capabilities
            .get(&capability)
            .cloned()
            .unwrap_or(CapabilityStatus::Available)
    }

    /// Record a probe result, builder-style.
    #[must_use]
    pub fn with(mut self, capability: Capability, status: CapabilityStatus) -> Self {
        self.capabilities.insert(capability, status);
        self
    }

    /// The `{{os}}` interpolation value for `action.unsupportedOnOs`:
    /// `"Linux (Wayland)"` when an environment refinement exists, otherwise
    /// the plain OS display name.
    #[must_use]
    pub fn os_label(&self) -> String {
        match &self.environment {
            Some(env) => format!("{} ({env})", self.os.display_name()),
            None => self.os.display_name().to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_label_includes_environment() {
        let snap = CapabilitySnapshot {
            os: Os::Linux,
            environment: Some("Wayland".into()),
            capabilities: BTreeMap::new(),
        };
        assert_eq!(snap.os_label(), "Linux (Wayland)");
        assert_eq!(
            CapabilitySnapshot::unconstrained(Os::MacOs).os_label(),
            "macOS"
        );
    }

    #[test]
    fn absent_capability_defaults_to_available() {
        let snap = CapabilitySnapshot::unconstrained(Os::Windows);
        assert_eq!(
            snap.status(Capability::InputInjection),
            CapabilityStatus::Available
        );
    }

    #[test]
    fn recorded_status_is_returned() {
        let snap = CapabilitySnapshot::unconstrained(Os::Linux).with(
            Capability::InputInjection,
            CapabilityStatus::Unavailable {
                reason: "Wayland session without ydotool or libei portal".into(),
                fix: Some(CapabilityFix::InstallTool {
                    tool: "ydotool".into(),
                    hint: "install ydotool and enable its daemon".into(),
                }),
            },
        );
        assert!(matches!(
            snap.status(Capability::InputInjection),
            CapabilityStatus::Unavailable { .. }
        ));
    }
}
