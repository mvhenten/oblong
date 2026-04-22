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
    out.push_str(&format!("focus_on_window_activation {}\n", config.focus_on_window_activation));

    // Mouse focus
    out.push_str(&format!("focus_follows_mouse {}\n", config.focus_follows_mouse));

    // Popups during fullscreen
    out.push_str(&format!("popup_during_fullscreen {}\n", config.popup_during_fullscreen));

    // Workspace back-and-forth (makes Super+Tab work properly)
    out.push_str(&format!(
        "workspace_auto_back_and_forth {}\n",
        if config.workspace_auto_back_and_forth { "yes" } else { "no" }
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
    out.push_str("for_window [title=\"Oblong\"] floating enable\n");
    out.push_str("for_window [title=\"Oblong\"] move position center\n");

    out.push('\n');

    fs::write(dir.join("defaults.conf"), &out).map_err(|e| e.to_string())?;

    super::config::ensure_include()?;

    Ok(())
}

/// Apply defaults to the running sway instance.
pub fn apply_defaults_live(config: &DefaultsConfig) {
    let mut commands = vec![
        format!("focus_on_window_activation {}", config.focus_on_window_activation),
        format!("focus_follows_mouse {}", config.focus_follows_mouse),
        format!("popup_during_fullscreen {}", config.popup_during_fullscreen),
        format!(
            "workspace_auto_back_and_forth {}",
            if config.workspace_auto_back_and_forth { "yes" } else { "no" }
        ),
        format!("mouse_warping {}", config.mouse_warping),
    ];

    if config.float_by_default {
        commands.push("for_window [app_id=\".*\"] floating enable".into());
        commands.push("for_window [title=\".*\"] floating enable".into());
    }

    for cmd in &commands {
        let _ = std::process::Command::new("swaymsg")
            .arg(cmd)
            .output();
    }
}
