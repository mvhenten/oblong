use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ── Data model ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppearanceConfig {
    pub font_family: String,
    pub font_size: u32,
    pub gaps_inner: u32,
    pub gaps_outer: u32,
    pub border_width: u32,
    pub border_style: BorderStyle,
    pub colors: WindowColors,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BorderStyle {
    Pixel,
    Normal,
    None,
}

impl BorderStyle {
    pub const ALL: &'static [BorderStyle] = &[
        BorderStyle::Pixel,
        BorderStyle::Normal,
        BorderStyle::None,
    ];
}

impl std::fmt::Display for BorderStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BorderStyle::Pixel => write!(f, "pixel"),
            BorderStyle::Normal => write!(f, "normal"),
            BorderStyle::None => write!(f, "none"),
        }
    }
}

impl BorderStyle {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "pixel" => Some(BorderStyle::Pixel),
            "normal" => Some(BorderStyle::Normal),
            "none" => Some(BorderStyle::None),
            _ => None,
        }
    }
}

/// Sway window border colors.
/// Each color class has: border, background, text, indicator, child_border.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowColors {
    pub focused: ColorSet,
    pub focused_inactive: ColorSet,
    pub unfocused: ColorSet,
    pub urgent: ColorSet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorSet {
    pub border: String,
    pub background: String,
    pub text: String,
    pub indicator: String,
    pub child_border: String,
}

// ── Defaults ────────────────────────────────────────────────

impl Default for AppearanceConfig {
    fn default() -> Self {
        Self {
            font_family: "Noto Sans".into(),
            font_size: 11,
            gaps_inner: 10,
            gaps_outer: 5,
            border_width: 2,
            border_style: BorderStyle::Pixel,
            colors: WindowColors::default(),
        }
    }
}

impl Default for WindowColors {
    fn default() -> Self {
        Self {
            focused: ColorSet {
                border: "#88bb88".into(),
                background: "#2d2d2d".into(),
                text: "#ffffff".into(),
                indicator: "#88bb88".into(),
                child_border: "#88bb88".into(),
            },
            focused_inactive: ColorSet {
                border: "#555555".into(),
                background: "#2d2d2d".into(),
                text: "#888888".into(),
                indicator: "#555555".into(),
                child_border: "#555555".into(),
            },
            unfocused: ColorSet {
                border: "#333333".into(),
                background: "#1d1d1d".into(),
                text: "#888888".into(),
                indicator: "#333333".into(),
                child_border: "#333333".into(),
            },
            urgent: ColorSet {
                border: "#cc8888".into(),
                background: "#2d2d2d".into(),
                text: "#ffffff".into(),
                indicator: "#cc8888".into(),
                child_border: "#cc8888".into(),
            },
        }
    }
}

// ── Available fonts ─────────────────────────────────────────

/// Query system fonts using fc-list and return sorted unique family names.
pub fn list_system_fonts() -> Vec<String> {
    let output = std::process::Command::new("fc-list")
        .args([":", "family"])
        .output();

    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut fonts: Vec<String> = text
                .lines()
                .flat_map(|line| line.split(','))
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            fonts.sort();
            fonts.dedup();
            fonts
        }
        Err(_) => vec![
            "Noto Sans".into(),
            "DejaVu Sans".into(),
            "Liberation Sans".into(),
            "monospace".into(),
        ],
    }
}

// ── Persistence ─────────────────────────────────────────────

fn appearance_config_path() -> PathBuf {
    super::config::config_dir().join("appearance.json")
}

pub fn load_appearance() -> Option<AppearanceConfig> {
    let data = fs::read_to_string(appearance_config_path()).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_appearance(config: &AppearanceConfig) {
    if let Ok(json) = serde_json::to_string_pretty(config) {
        fs::write(appearance_config_path(), json).ok();
    }
}

// ── Sway config generation ─────────────────────────────────

fn sway_oblong_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
        .join(".config")
        .join("sway")
        .join("oblong")
}

pub fn write_appearance_conf(config: &AppearanceConfig) -> Result<(), String> {
    let dir = sway_oblong_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let mut out = String::from("# ── Oblong appearance — auto-generated, do not edit by hand ──\n\n");

    // Font
    out.push_str(&format!("font pango:{} {}\n\n", config.font_family, config.font_size));

    // Gaps
    out.push_str(&format!("gaps inner {}\n", config.gaps_inner));
    out.push_str(&format!("gaps outer {}\n\n", config.gaps_outer));

    // Borders
    match config.border_style {
        BorderStyle::None => {
            out.push_str("default_border none\n");
            out.push_str("default_floating_border none\n\n");
        }
        style => {
            out.push_str(&format!("default_border {} {}\n", style, config.border_width));
            out.push_str(&format!("default_floating_border {} {}\n\n", style, config.border_width));
        }
    }

    // Colors
    fn color_line(class: &str, cs: &ColorSet) -> String {
        format!(
            "client.{:<20} {} {} {} {} {}\n",
            class, cs.border, cs.background, cs.text, cs.indicator, cs.child_border
        )
    }
    out.push_str(&color_line("focused", &config.colors.focused));
    out.push_str(&color_line("focused_inactive", &config.colors.focused_inactive));
    out.push_str(&color_line("unfocused", &config.colors.unfocused));
    out.push_str(&color_line("urgent", &config.colors.urgent));

    fs::write(dir.join("appearance.conf"), &out).map_err(|e| e.to_string())?;

    comment_out_appearance_conflicts()?;
    super::config::ensure_include()?;

    Ok(())
}

/// Appearance directives that oblong manages.
/// If these appear in the main sway config, they conflict with our generated conf.
const APPEARANCE_PREFIXES: &[&str] = &[
    "client.focused",
    "client.unfocused",
    "client.focused_inactive",
    "client.urgent",
    "default_border",
    "default_floating_border",
    "gaps inner",
    "gaps outer",
    "font ",
];

/// Comment out any appearance directives in the main sway config that
/// would conflict with our generated appearance.conf.
fn comment_out_appearance_conflicts() -> Result<(), String> {
    let main_config = super::config::sway_config_path();
    let content = fs::read_to_string(&main_config).map_err(|e| e.to_string())?;
    let mut changed = false;
    let output: Vec<String> = content
        .lines()
        .map(|line| {
            let trimmed = line.trim();
            // Skip lines already commented out (by us or the user)
            if trimmed.starts_with('#') {
                return line.to_string();
            }
            let dominated = APPEARANCE_PREFIXES
                .iter()
                .any(|prefix| trimmed.starts_with(prefix));
            if dominated {
                changed = true;
                format!("# [oblong] {}", line)
            } else {
                line.to_string()
            }
        })
        .collect();

    if changed {
        fs::write(&main_config, output.join("\n")).map_err(|e| e.to_string())?;
    }
    Ok(())
}
