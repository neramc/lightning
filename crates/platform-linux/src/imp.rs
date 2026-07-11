// SPDX-License-Identifier: GPL-3.0-only
// Copyright (C) 2026 neramc

//! The actual Linux platform implementation.

use std::path::{Path, PathBuf};

use lightning_platform::{
    Capability, CapabilityFix, CapabilitySnapshot, CapabilityStatus, Os, PlatformError,
    PlatformOps,
};

fn install_fix(tool: &str, hint: &str) -> CapabilityFix {
    CapabilityFix::InstallTool { tool: tool.into(), hint: hint.into() }
}
use tokio::process::Command;

/// Which display session the process runs under.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Session {
    Wayland,
    X11,
    Headless,
}

fn detect_session() -> Session {
    if std::env::var_os("WAYLAND_DISPLAY").is_some() {
        Session::Wayland
    } else if std::env::var_os("DISPLAY").is_some() {
        Session::X11
    } else {
        Session::Headless
    }
}

fn has_tool(name: &str) -> bool {
    which::which(name).is_ok()
}

fn ydotool_socket() -> Option<PathBuf> {
    let dir = std::env::var_os("XDG_RUNTIME_DIR")?;
    let socket = PathBuf::from(dir).join(".ydotool_socket");
    socket.exists().then_some(socket)
}

/// Linux [`PlatformOps`] implementation.
#[derive(Default)]
pub struct LinuxPlatform;

impl LinuxPlatform {
    /// Create the platform handle.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

async fn run_command(program: &str, args: &[&str]) -> Result<std::process::Output, PlatformError> {
    // Arguments are always argv arrays — never string concatenation (§14).
    let output = Command::new(program).args(args).output().await?;
    Ok(output)
}

async fn run_checked(program: &str, args: &[&str]) -> Result<(), PlatformError> {
    let output = run_command(program, args).await?;
    if output.status.success() {
        Ok(())
    } else {
        Err(PlatformError::CommandFailed(format!(
            "{program} exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        )))
    }
}

#[async_trait::async_trait]
impl PlatformOps for LinuxPlatform {
    fn os(&self) -> Os {
        Os::Linux
    }

    fn probe(&self) -> CapabilitySnapshot {
        let session = detect_session();
        let environment = match session {
            Session::Wayland => Some("Wayland".to_owned()),
            Session::X11 => Some("X11".to_owned()),
            Session::Headless => Some("headless".to_owned()),
        };

        let mut snapshot = CapabilitySnapshot { os: Os::Linux, environment, ..CapabilitySnapshot::unconstrained(Os::Linux) };

        // Input injection: X11 native; Wayland needs the ydotool daemon or a
        // libei portal. Probe the socket, not just the binary — a binary
        // without its daemon still cannot inject (fix(platform-linux) lore).
        snapshot = snapshot.with(
            Capability::InputInjection,
            match session {
                Session::X11 => CapabilityStatus::Available,
                Session::Wayland => {
                    if ydotool_socket().is_some() {
                        CapabilityStatus::Available
                    } else {
                        CapabilityStatus::Unavailable {
                            reason: "Wayland session without a running ydotool daemon or libei portal".into(),
                            fix: Some(install_fix(
                                "ydotool",
                                "install ydotool and enable ydotoold (e.g. `systemctl --user enable --now ydotool`)",
                            )),
                        }
                    }
                }
                Session::Headless => CapabilityStatus::Unavailable {
                    reason: "no graphical session".into(),
                    fix: None,
                },
            },
        );

        // Screenshots: X11 direct; Wayland via the xdg-desktop-portal.
        snapshot = snapshot.with(
            Capability::Screenshot,
            match session {
                Session::X11 => CapabilityStatus::Available,
                Session::Wayland => CapabilityStatus::Degraded {
                    reason: "captured through the xdg-desktop-portal screenshot API".into(),
                },
                Session::Headless => CapabilityStatus::Unavailable {
                    reason: "no graphical session".into(),
                    fix: None,
                },
            },
        );

        // Clipboard tooling.
        let clipboard_ok = match session {
            Session::Wayland => has_tool("wl-copy"),
            Session::X11 => has_tool("xclip") || has_tool("xsel"),
            Session::Headless => false,
        };
        let clipboard_status = if clipboard_ok {
            CapabilityStatus::Available
        } else {
            CapabilityStatus::Unavailable {
                reason: "no clipboard utility for this session".into(),
                fix: Some(install_fix(
                    if session == Session::Wayland { "wl-clipboard" } else { "xclip" },
                    "install it from your distribution's package manager",
                )),
            }
        };
        snapshot = snapshot
            .with(Capability::ClipboardRead, clipboard_status.clone())
            .with(Capability::ClipboardWrite, clipboard_status);

        // NetworkManager for Wi-Fi actions.
        snapshot = snapshot.with(
            Capability::NetworkControl,
            if has_tool("nmcli") {
                CapabilityStatus::Available
            } else {
                CapabilityStatus::Unavailable {
                    reason: "NetworkManager (nmcli) not present".into(),
                    fix: Some(install_fix(
                        "nmcli",
                        "install NetworkManager",
                    )),
                }
            },
        );

        // Notifications: notify-send talks to any FreeDesktop daemon.
        snapshot = snapshot.with(
            Capability::Notifications,
            if has_tool("notify-send") {
                CapabilityStatus::Available
            } else {
                CapabilityStatus::Degraded {
                    reason: "notify-send missing; falling back to direct D-Bus".into(),
                }
            },
        );

        // MPRIS media control (playerctl is a convenience, D-Bus is the API).
        snapshot = snapshot.with(
            Capability::MediaControl,
            if has_tool("playerctl") {
                CapabilityStatus::Available
            } else {
                CapabilityStatus::Degraded { reason: "MPRIS over D-Bus without playerctl".into() }
            },
        );

        // TTS via speech-dispatcher.
        snapshot = snapshot.with(
            Capability::SpeechSynthesis,
            if has_tool("spd-say") {
                CapabilityStatus::Available
            } else {
                CapabilityStatus::Unavailable {
                    reason: "speech-dispatcher not installed".into(),
                    fix: Some(install_fix(
                        "speech-dispatcher",
                        "install speech-dispatcher",
                    )),
                }
            },
        );

        // Global hotkeys and window management are compositor topics on Wayland.
        if session == Session::Wayland {
            snapshot = snapshot
                .with(
                    Capability::GlobalHotkeys,
                    CapabilityStatus::Degraded {
                        reason: "register a compositor keybind that calls `lightning run`".into(),
                    },
                )
                .with(
                    Capability::WindowManagement,
                    CapabilityStatus::Degraded {
                        reason: "compositor-dependent (sway/i3/Hyprland IPC supported)".into(),
                    },
                );
        }

        snapshot
    }

    async fn reveal_in_file_manager(&self, path: &Path) -> Result<(), PlatformError> {
        let uri = format!("file://{}", path.display());
        // Prefer the FreeDesktop FileManager1 interface; fall back to xdg-open
        // on the parent directory (§8.3 F).
        let dbus = run_command(
            "gdbus",
            &[
                "call",
                "--session",
                "--dest",
                "org.freedesktop.FileManager1",
                "--object-path",
                "/org/freedesktop/FileManager1",
                "--method",
                "org.freedesktop.FileManager1.ShowItems",
                &format!("['{uri}']"),
                "",
            ],
        )
        .await;
        if matches!(&dbus, Ok(output) if output.status.success()) {
            return Ok(());
        }
        let parent = path.parent().unwrap_or(path);
        run_checked("xdg-open", &[&parent.display().to_string()]).await
    }

    async fn send_notification(&self, title: &str, body: &str) -> Result<(), PlatformError> {
        if !has_tool("notify-send") {
            return Err(PlatformError::MissingTool {
                tool: "notify-send".into(),
                hint: "install libnotify".into(),
            });
        }
        run_checked("notify-send", &["--app-name", "Lightning", title, body]).await
    }

    async fn open_url(&self, url: &str) -> Result<(), PlatformError> {
        run_checked("xdg-open", &[url]).await
    }

    async fn clipboard_read_text(&self) -> Result<String, PlatformError> {
        let (program, args): (&str, &[&str]) = match detect_session() {
            Session::Wayland if has_tool("wl-paste") => ("wl-paste", &["--no-newline"]),
            Session::X11 if has_tool("xclip") => ("xclip", &["-selection", "clipboard", "-o"]),
            Session::X11 if has_tool("xsel") => ("xsel", &["--clipboard", "--output"]),
            Session::Headless => {
                return Err(PlatformError::Unsupported {
                    os: "Linux (headless)".into(),
                    reason: "no graphical session".into(),
                });
            }
            _ => {
                return Err(PlatformError::MissingTool {
                    tool: "wl-clipboard / xclip".into(),
                    hint: "install wl-clipboard (Wayland) or xclip (X11)".into(),
                });
            }
        };
        let output = run_command(program, args).await?;
        if !output.status.success() {
            return Err(PlatformError::CommandFailed(format!("{program} failed")));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }

    async fn clipboard_write_text(&self, text: &str) -> Result<(), PlatformError> {
        use tokio::io::AsyncWriteExt;
        let program = match detect_session() {
            Session::Wayland if has_tool("wl-copy") => "wl-copy",
            Session::X11 if has_tool("xclip") => "xclip",
            Session::Headless => {
                return Err(PlatformError::Unsupported {
                    os: "Linux (headless)".into(),
                    reason: "no graphical session".into(),
                });
            }
            _ => {
                return Err(PlatformError::MissingTool {
                    tool: "wl-clipboard / xclip".into(),
                    hint: "install wl-clipboard (Wayland) or xclip (X11)".into(),
                });
            }
        };
        let args: &[&str] =
            if program == "xclip" { &["-selection", "clipboard"] } else { &[] };
        let mut child = Command::new(program)
            .args(args)
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(text.as_bytes()).await?;
        }
        let status = child.wait().await?;
        if status.success() {
            Ok(())
        } else {
            Err(PlatformError::CommandFailed(format!("{program} exited with {status}")))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_reports_an_environment() {
        let snapshot = LinuxPlatform::new().probe();
        assert_eq!(snapshot.os, Os::Linux);
        assert!(snapshot.environment.is_some());
    }

    #[test]
    fn headless_probe_marks_input_unavailable() {
        // This test runs in CI containers without a display; when a display
        // exists locally the assertion is skipped rather than faked.
        if detect_session() == Session::Headless {
            let snapshot = LinuxPlatform::new().probe();
            assert!(matches!(
                snapshot.status(Capability::InputInjection),
                CapabilityStatus::Unavailable { .. }
            ));
            assert_eq!(snapshot.os_label(), "Linux (headless)");
        }
    }
}
