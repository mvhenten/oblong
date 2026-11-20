use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::config::Binding;

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
    /// macOS-style Cmd app shortcuts (copy, paste, tabs, find, line jumps)
    pub super_copy_paste: bool,
    /// Command Cmd+Space runs (empty = unbound)
    pub launcher: String,
    /// Command Cmd+Return runs (empty = unbound)
    pub terminal: String,
    /// Cmd+Shift+3/4/5 screenshots of the screen, a region or the focused window
    pub screenshots: bool,
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
            super_copy_paste: true,
            launcher: "fuzzel".into(),
            terminal: "foot".into(),
            screenshots: true,
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

    // macOS-style Cmd app shortcuts via wtype
    if config.super_copy_paste {
        let helper = super_key_helper_path();
        write_helper(&helper, SUPER_KEY_HELPER).map_err(|e| e.to_string())?;
        out.push_str("\n# macOS-style Cmd app shortcuts (requires wtype)\n");
        out.push_str(&render_app_shortcuts(&helper.display().to_string()));
    }

    if config.screenshots {
        write_helper(&screenshot_helper_path(), SCREENSHOT_HELPER).map_err(|e| e.to_string())?;
    }
    let launch = launch_bindings(config);
    if !launch.is_empty() {
        out.push_str("\n# Launcher, terminal and screenshots\n");
        for b in &launch {
            out.push_str(&format!("bindsym --no-repeat {} {}\n", b.keys, b.command));
        }
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

fn super_key_helper_path() -> PathBuf {
    sway_oblong_dir().join("super-key-helper.sh")
}

/// Cmd app shortcuts as (keys, helper action, repeats while held).
/// The `kill` action is sway's own command rather than a helper action.
const APP_SHORTCUTS: &[(&str, &str, bool)] = &[
    ("$mod+c", "copy", false),
    ("$mod+v", "paste", false),
    ("$mod+x", "cut", false),
    ("$mod+a", "select-all", false),
    ("$mod+z", "undo", false),
    ("$mod+Shift+z", "redo", false),
    ("$mod+t", "new-tab", false),
    ("$mod+Shift+t", "reopen-tab", false),
    ("$mod+n", "new-window", false),
    ("$mod+w", "close-tab", false),
    ("$mod+l", "address-bar", false),
    ("$mod+r", "reload", false),
    ("$mod+f", "find", false),
    ("$mod+q", "kill", false),
    ("$mod+Left", "line-start", true),
    ("$mod+Right", "line-end", true),
    ("$mod+Up", "doc-start", true),
    ("$mod+Down", "doc-end", true),
    ("Mod1+Left", "word-left", true),
    ("Mod1+Right", "word-right", true),
    ("$mod+BackSpace", "delete-line", true),
    ("Mod1+BackSpace", "delete-word", true),
];

fn app_shortcut_command(helper: &str, action: &str) -> String {
    if action == "kill" {
        return "kill".into();
    }
    format!("exec {helper} {action}")
}

fn render_app_shortcuts(helper: &str) -> String {
    APP_SHORTCUTS
        .iter()
        .map(|(keys, action, repeat)| {
            let flag = if *repeat { "" } else { "--no-repeat " };
            format!(
                "bindsym {flag}{keys} {}\n",
                app_shortcut_command(helper, action)
            )
        })
        .collect()
}

fn screenshot_helper_path() -> PathBuf {
    sway_oblong_dir().join("screenshot.sh")
}

/// The launcher, terminal and screenshot bindings `write_defaults_conf` generates.
fn launch_bindings(config: &DefaultsConfig) -> Vec<Binding> {
    let mut bindings = Vec::new();
    for (keys, command) in [
        ("$mod+space", &config.launcher),
        ("$mod+Return", &config.terminal),
    ] {
        let command = command.trim();
        if !command.is_empty() {
            bindings.push(Binding {
                keys: keys.into(),
                command: format!("exec {command}"),
            });
        }
    }
    if config.screenshots {
        let helper = screenshot_helper_path().display().to_string();
        for (n, target) in [(3, "full"), (4, "region"), (5, "window")] {
            bindings.push(Binding {
                keys: format!("$mod+Shift+{n}"),
                command: format!("exec {helper} {target}"),
            });
        }
    }
    bindings
}

/// Every Cmd binding `write_defaults_conf` generates, for conflict
/// detection against the main sway config.
pub fn app_bindings(config: &DefaultsConfig) -> Vec<Binding> {
    let mut bindings = Vec::new();
    if config.super_copy_paste {
        let helper = super_key_helper_path().display().to_string();
        bindings.extend(APP_SHORTCUTS.iter().map(|(keys, action, _)| Binding {
            keys: (*keys).into(),
            command: app_shortcut_command(&helper, action),
        }));
    }
    bindings.extend(launch_bindings(config));
    bindings
}

/// Check if wtype is available on the system.
pub fn has_wtype() -> bool {
    std::process::Command::new("which")
        .arg("wtype")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn write_helper(path: &std::path::Path, script: &str) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::write(path, script)?;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755))
}

const SCREENSHOT_HELPER: &str = r#"#!/bin/bash
# Screenshot to ~/Pictures: full screen, a selected region, or the focused window.

DIR="$HOME/Pictures"
mkdir -p "$DIR"
FILE="$DIR/screenshot-$(date +%Y-%m-%d-%H%M%S).png"

case "$1" in
    full) grim "$FILE" ;;
    region) grim -g "$(slurp)" "$FILE" ;;
    window)
        GEOMETRY=$(swaymsg -t get_tree | jq -r '.. | select(.focused? == true) | .rect | "\(.x),\(.y) \(.width)x\(.height)"')
        grim -g "$GEOMETRY" "$FILE"
        ;;
esac
"#;

const SUPER_KEY_HELPER: &str = r#"#!/bin/bash
# Helper for macOS-style Super key shortcuts.
# Detects whether the focused app is a terminal and sends the
# appropriate key combo via wtype.

TERMINALS="foot|Alacritty|kitty|wezterm|alacritty|xterm|urxvt|terminator|gnome-terminal|konsole|tilix"

APP_ID=$(swaymsg -t get_tree | jq -r '.. | select(.focused? == true) | .app_id // empty' 2>/dev/null)
IS_TERM=0
if echo "$APP_ID" | grep -qiE "^($TERMINALS)$"; then
    IS_TERM=1
fi

# Ctrl+Shift+<key> in terminals (so Ctrl+C stays an interrupt), Ctrl+<key> elsewhere.
app_ctrl() {
    if [ $IS_TERM -eq 1 ]; then
        wtype -M ctrl -M shift -k "$1" -m shift -m ctrl
    else
        wtype -M ctrl -k "$1" -m ctrl
    fi
}

ctrl() {
    wtype -M ctrl -k "$1" -m ctrl
}

case "$1" in
    copy) app_ctrl c ;;
    paste) app_ctrl v ;;
    cut) app_ctrl x ;;
    new-tab) app_ctrl t ;;
    new-window) app_ctrl n ;;
    close-tab) app_ctrl w ;;
    address-bar) app_ctrl l ;;
    reload) app_ctrl r ;;
    find) app_ctrl f ;;
    select-all) ctrl a ;;
    undo) ctrl z ;;
    redo) wtype -M ctrl -M shift -k z -m shift -m ctrl ;;
    reopen-tab) wtype -M ctrl -M shift -k t -m shift -m ctrl ;;
    line-start) wtype -k Home ;;
    line-end) wtype -k End ;;
    doc-start) ctrl Home ;;
    doc-end) ctrl End ;;
    word-left)
        if [ $IS_TERM -eq 1 ]; then wtype -M alt -k b -m alt; else ctrl Left; fi
        ;;
    word-right)
        if [ $IS_TERM -eq 1 ]; then wtype -M alt -k f -m alt; else ctrl Right; fi
        ;;
    delete-line)
        if [ $IS_TERM -eq 1 ]; then ctrl u; else wtype -M shift -k Home -m shift -k BackSpace; fi
        ;;
    delete-word)
        if [ $IS_TERM -eq 1 ]; then ctrl w; else ctrl BackSpace; fi
        ;;
esac
"#;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{default_bindings, parse_sway_bindings, render_shortcuts};

    fn normalized(keys: &str) -> String {
        let mut parts: Vec<String> = keys.split('+').map(str::to_lowercase).collect();
        let key = parts.pop().unwrap_or_default();
        parts.sort_unstable();
        parts.push(key);
        parts.join("+")
    }

    #[test]
    fn default_keymap_binds_each_combo_once_and_leaves_helper_keys_free() {
        let window = default_bindings("oblong");
        let window_refs: Vec<&Binding> = window.iter().collect();
        let mut keys: Vec<String> = parse_sway_bindings(&render_shortcuts(&window_refs))
            .iter()
            .map(|b| normalized(&b.keys))
            .collect();
        assert_eq!(keys.len(), window.len());
        let app = app_bindings(&DefaultsConfig::default());
        assert_eq!(app.len(), APP_SHORTCUTS.len() + 5);
        keys.extend(app.iter().map(|b| normalized(&b.keys)));

        let total = keys.len();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), total, "a key combo is bound twice");

        let vt_switch = (1..=12).map(|n| normalized(&format!("Ctrl+Mod1+F{n}")));
        let helper_sends = [
            "Ctrl+c",
            "Ctrl+v",
            "Ctrl+x",
            "Ctrl+a",
            "Ctrl+z",
            "Ctrl+t",
            "Ctrl+n",
            "Ctrl+w",
            "Ctrl+l",
            "Ctrl+r",
            "Ctrl+f",
            "Ctrl+u",
            "Ctrl+Left",
            "Ctrl+Right",
            "Ctrl+Home",
            "Ctrl+End",
            "Ctrl+BackSpace",
            "Mod1+b",
            "Mod1+f",
            "Home",
            "End",
            "Shift+Home",
            "BackSpace",
        ]
        .map(normalized);
        for reserved in vt_switch.chain(helper_sends) {
            assert!(!keys.contains(&reserved), "{reserved} must stay unbound");
        }
    }
}
