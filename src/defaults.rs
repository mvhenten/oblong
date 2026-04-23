use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Sway behavior defaults that make it work like a proper desktop.
/// These are settings that aren't exposed in other tabs but are
/// essential for good UX.

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct DefaultsConfig {
    /// New windows steal focus (instead of launching behind)
    pub focus_on_window_activation: String,
    /// Mouse focus behavior
    pub focus_follows_mouse: String,
    /// What happens when a popup appears during fullscreen
    pub popup_during_fullscreen: String,
    /// Repeated workspace switch goes back
    pub workspace_auto_back_and_forth: bool,
    /// Mouse warps to focused container
    pub mouse_warping: String,
    /// Float all new windows by default (macOS-like)
    pub float_by_default: bool,
    /// macOS-style Super+C/V/A/Z shortcuts
    pub super_copy_paste: bool,
    /// Screen blank timeout in seconds (0 = disabled)
    pub screen_blank_timeout: u32,
    /// Lock screen timeout in seconds (0 = disabled)
    pub lock_timeout: u32,
}

impl Default for DefaultsConfig {
    fn default() -> Self {
        Self {
            focus_on_window_activation: "focus".into(),
            focus_follows_mouse: "yes".into(),
            popup_during_fullscreen: "smart".into(),
            workspace_auto_back_and_forth: true,
            mouse_warping: "output".into(),
            float_by_default: true,
            super_copy_paste: false,
            screen_blank_timeout: 300,
            lock_timeout: 600,
        }
    }
}

// ── Paths ───────────────────────────────────────────────────

fn defaults_config_path() -> PathBuf {
    super::config::config_dir().join("defaults.json")
}

fn sway_oblong_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
        .join(".config")
        .join("sway")
        .join("oblong")
}

// ── Persistence ─────────────────────────────────────────────

pub fn load_defaults() -> Option<DefaultsConfig> {
    let data = fs::read_to_string(defaults_config_path()).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_defaults(config: &DefaultsConfig) {
    if let Ok(json) = serde_json::to_string_pretty(config) {
        fs::write(defaults_config_path(), json).ok();
    }
}

// ── Sway config generation ─────────────────────────────────

pub fn write_defaults_conf(config: &DefaultsConfig) -> Result<(), String> {
    let dir = sway_oblong_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let mut out = String::from("# ── Oblong defaults — auto-generated, do not edit by hand ──\n\n");

    // Focus behavior — new windows get focus instead of opening behind
    out.push_str(&format!(
        "focus_on_window_activation {}\n",
        config.focus_on_window_activation
    ));

    // Mouse focus
    out.push_str(&format!(
        "focus_follows_mouse {}\n",
        config.focus_follows_mouse
    ));

    // Popups during fullscreen
    out.push_str(&format!(
        "popup_during_fullscreen {}\n",
        config.popup_during_fullscreen
    ));

    // Workspace back-and-forth (makes Super+Tab work properly)
    out.push_str(&format!(
        "workspace_auto_back_and_forth {}\n",
        if config.workspace_auto_back_and_forth {
            "yes"
        } else {
            "no"
        }
    ));

    // Mouse warping
    out.push_str(&format!("mouse_warping {}\n", config.mouse_warping));

    // Float all windows by default (macOS-like behavior)
    if config.float_by_default {
        out.push_str("\n# Float all new windows by default (macOS-like)\n");
        out.push_str("for_window [app_id=\".*\"] floating enable\n");
        out.push_str("for_window [title=\".*\"] floating enable\n");
    }

    // Float and center the oblong GUI itself
    out.push_str("\n# Oblong GUI: float, center, reasonable size\n");
    out.push_str("for_window [app_id=\"Oblong\"] floating enable\n");
    out.push_str("for_window [app_id=\"Oblong\"] move position center\n");

    // macOS-style Super+C/V/A/Z via wtype
    if config.super_copy_paste {
        let helper = dir.join("super-key-helper.sh");
        write_super_key_helper(&helper).map_err(|e| e.to_string())?;
        let h = helper.display();
        out.push_str("\n# macOS-style Super key shortcuts (requires wtype)\n");
        out.push_str(&format!("bindsym --no-repeat $mod+c exec {} copy\n", h));
        out.push_str(&format!("bindsym --no-repeat $mod+v exec {} paste\n", h));
        out.push_str(&format!(
            "bindsym --no-repeat $mod+a exec {} select-all\n",
            h
        ));
        out.push_str(&format!("bindsym --no-repeat $mod+z exec {} undo\n", h));
        out.push_str(&format!(
            "bindsym --no-repeat $mod+Shift+z exec {} redo\n",
            h
        ));
        out.push_str(&format!("bindsym --no-repeat $mod+x exec {} cut\n", h));
    }

    // Screen blanking & auto-lock via swayidle
    if config.screen_blank_timeout > 0 || config.lock_timeout > 0 {
        out.push_str("\n# Screen blanking & auto-lock\n");
        let mut idle_args = String::from("exec swayidle -w");
        if config.screen_blank_timeout > 0 {
            idle_args.push_str(&format!(
                " timeout {} 'swaymsg \"output * power off\"' resume 'swaymsg \"output * power on\"'",
                config.screen_blank_timeout
            ));
        }
        if config.lock_timeout > 0 {
            idle_args.push_str(&format!(
                " timeout {} 'swaylock -c 1a1a1a'",
                config.lock_timeout
            ));
        }
        idle_args.push_str(" before-sleep 'swaylock -c 1a1a1a'");
        out.push_str(&idle_args);
        out.push('\n');
    }

    out.push('\n');

    fs::write(dir.join("defaults.conf"), &out).map_err(|e| e.to_string())?;

    super::config::ensure_include()?;

    Ok(())
}

/// Check if wtype is available on the system.
pub fn has_wtype() -> bool {
    std::process::Command::new("which")
        .arg("wtype")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn write_super_key_helper(path: &std::path::Path) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let script = r#"#!/bin/bash
# Helper for macOS-style Super key shortcuts.
# Detects whether the focused app is a terminal and sends the
# appropriate key combo via wtype.

TERMINALS="foot|Alacritty|kitty|wezterm|alacritty|xterm|urxvt|terminator|gnome-terminal|konsole|tilix"

APP_ID=$(swaymsg -t get_tree | jq -r '.. | select(.focused? == true) | .app_id // empty' 2>/dev/null)
IS_TERM=0
if echo "$APP_ID" | grep -qiE "^($TERMINALS)$"; then
    IS_TERM=1
fi

case "$1" in
    copy)
        if [ $IS_TERM -eq 1 ]; then
            wtype -M ctrl -M shift -k c -m shift -m ctrl
        else
            wtype -M ctrl -k c -m ctrl
        fi
        ;;
    paste)
        if [ $IS_TERM -eq 1 ]; then
            wtype -M ctrl -M shift -k v -m shift -m ctrl
        else
            wtype -M ctrl -k v -m ctrl
        fi
        ;;
    cut)
        if [ $IS_TERM -eq 1 ]; then
            wtype -M ctrl -M shift -k x -m shift -m ctrl
        else
            wtype -M ctrl -k x -m ctrl
        fi
        ;;
    select-all)
        wtype -M ctrl -k a -m ctrl
        ;;
    undo)
        wtype -M ctrl -k z -m ctrl
        ;;
    redo)
        wtype -M ctrl -M shift -k z -m shift -m ctrl
        ;;
esac
"#;
    fs::write(path, script)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

/// Apply defaults to the running sway instance.
pub fn apply_defaults_live(config: &DefaultsConfig) {
    let mut commands = vec![
        format!(
            "focus_on_window_activation {}",
            config.focus_on_window_activation
        ),
        format!("focus_follows_mouse {}", config.focus_follows_mouse),
        format!("popup_during_fullscreen {}", config.popup_during_fullscreen),
        format!(
            "workspace_auto_back_and_forth {}",
            if config.workspace_auto_back_and_forth {
                "yes"
            } else {
                "no"
            }
        ),
        format!("mouse_warping {}", config.mouse_warping),
    ];

    if config.float_by_default {
        commands.push("for_window [app_id=\".*\"] floating enable".into());
        commands.push("for_window [title=\".*\"] floating enable".into());
    }

    for cmd in &commands {
        let _ = std::process::Command::new("swaymsg").arg(cmd).output();
    }

    // Restart swayidle with new timeouts
    let _ = std::process::Command::new("pkill").arg("swayidle").output();
    if config.screen_blank_timeout > 0 || config.lock_timeout > 0 {
        let mut args = vec!["-w".to_string()];
        if config.screen_blank_timeout > 0 {
            args.extend([
                "timeout".into(),
                config.screen_blank_timeout.to_string(),
                "swaymsg \"output * power off\"".into(),
                "resume".into(),
                "swaymsg \"output * power on\"".into(),
            ]);
        }
        if config.lock_timeout > 0 {
            args.extend([
                "timeout".into(),
                config.lock_timeout.to_string(),
                "swaylock -c 1a1a1a".into(),
            ]);
        }
        args.extend(["before-sleep".into(), "swaylock -c 1a1a1a".into()]);
        let _ = std::process::Command::new("swayidle").args(&args).spawn();
    }
}
