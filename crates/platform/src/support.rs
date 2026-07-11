// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! Static per-platform support declarations (CLAUDE.md §6.5, §8.1).

use serde::{Deserialize, Serialize};

/// The operating systems Lightning targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Os {
    /// Windows 10 1809+ / 11.
    Windows,
    /// macOS 11+.
    #[serde(rename = "macos")]
    MacOs,
    /// Linux (any distribution; X11 vs Wayland is a runtime capability matter).
    Linux,
    /// FreeBSD 14+ (Tier 3, best-effort).
    FreeBsd,
}

impl Os {
    /// The user-visible name, exactly as interpolated into the
    /// `action.unsupportedOnOs` i18n string: `Windows` / `macOS` / `Linux` /
    /// `FreeBSD`.
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Os::Windows => "Windows",
            Os::MacOs => "macOS",
            Os::Linux => "Linux",
            Os::FreeBsd => "FreeBSD",
        }
    }

    /// The OS this binary was compiled for.
    #[must_use]
    pub const fn current() -> Os {
        #[cfg(target_os = "windows")]
        {
            Os::Windows
        }
        #[cfg(target_os = "macos")]
        {
            Os::MacOs
        }
        #[cfg(target_os = "freebsd")]
        {
            Os::FreeBsd
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "freebsd")))]
        {
            Os::Linux
        }
    }
}

impl std::fmt::Display for Os {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.display_name())
    }
}

/// How well a single action works on one OS.
///
/// Declared statically per action; refined at runtime by the
/// [`CapabilitySnapshot`](crate::CapabilitySnapshot). A wrong `Full` is worse
/// than an honest `None` (CLAUDE.md §17.5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "level", rename_all = "camelCase")]
pub enum SupportLevel {
    /// Works without conditions.
    Full,
    /// Works with a condition — a permission, an optional tool, or a specific
    /// environment. The note is a short technical explanation for docs/dev
    /// UIs; the user-facing message comes from i18n.
    Partial {
        /// Why the support is conditional (e.g. "Wayland needs ydotool").
        note: String,
    },
    /// Not supported: the block renders grayed with the localized badge.
    None,
}

/// The per-OS support matrix an action declares in its `ActionDef`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformSupport {
    /// Support level on Windows.
    pub windows: SupportLevel,
    /// Support level on macOS.
    pub macos: SupportLevel,
    /// Support level on Linux.
    pub linux: SupportLevel,
    /// Support level on FreeBSD (generally follows Linux minus systemd).
    pub freebsd: SupportLevel,
}

impl PlatformSupport {
    /// Full support everywhere — for pure, engine-level actions.
    #[must_use]
    pub fn all_full() -> Self {
        Self {
            windows: SupportLevel::Full,
            macos: SupportLevel::Full,
            linux: SupportLevel::Full,
            freebsd: SupportLevel::Full,
        }
    }

    /// Support on exactly one OS, `None` everywhere else (OS-exclusive actions, §8.4).
    #[must_use]
    pub fn only(os: Os) -> Self {
        let mut support = Self {
            windows: SupportLevel::None,
            macos: SupportLevel::None,
            linux: SupportLevel::None,
            freebsd: SupportLevel::None,
        };
        *support.for_os_mut(os) = SupportLevel::Full;
        support
    }

    /// Replace one OS entry, builder-style.
    #[must_use]
    pub fn with(mut self, os: Os, level: SupportLevel) -> Self {
        *self.for_os_mut(os) = level;
        self
    }

    /// The declared level for `os`.
    #[must_use]
    pub fn for_os(&self, os: Os) -> &SupportLevel {
        match os {
            Os::Windows => &self.windows,
            Os::MacOs => &self.macos,
            Os::Linux => &self.linux,
            Os::FreeBsd => &self.freebsd,
        }
    }

    fn for_os_mut(&mut self, os: Os) -> &mut SupportLevel {
        match os {
            Os::Windows => &mut self.windows,
            Os::MacOs => &mut self.macos,
            Os::Linux => &mut self.linux,
            Os::FreeBsd => &mut self.freebsd,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_names_are_canonical() {
        assert_eq!(Os::Windows.display_name(), "Windows");
        assert_eq!(Os::MacOs.display_name(), "macOS");
        assert_eq!(Os::Linux.display_name(), "Linux");
        assert_eq!(Os::FreeBsd.display_name(), "FreeBSD");
    }

    #[test]
    fn only_marks_a_single_os() {
        let s = PlatformSupport::only(Os::MacOs);
        assert_eq!(s.macos, SupportLevel::Full);
        assert_eq!(s.windows, SupportLevel::None);
        assert_eq!(s.linux, SupportLevel::None);
        assert_eq!(s.freebsd, SupportLevel::None);
    }

    #[test]
    fn with_overrides_one_entry() {
        let s = PlatformSupport::all_full().with(
            Os::Linux,
            SupportLevel::Partial {
                note: "Wayland needs ydotool".into(),
            },
        );
        assert!(matches!(s.for_os(Os::Linux), SupportLevel::Partial { .. }));
        assert_eq!(*s.for_os(Os::Windows), SupportLevel::Full);
    }

    #[test]
    fn serde_shape_is_stable() {
        let json =
            serde_json::to_value(SupportLevel::Partial { note: "n".into() }).expect("serializable");
        assert_eq!(json["level"], "partial");
        assert_eq!(json["note"], "n");
    }
}
