use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Command;

// ── Data from swaymsg ───────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct SwayOutput {
    pub name: String,
    pub make: String,
    pub model: String,
    pub serial: String,
    pub rect: SwayRect,
    pub modes: Vec<SwayMode>,
    pub scale: f64,
    pub transform: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SwayRect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SwayMode {
    pub width: i32,
    pub height: i32,
    pub refresh: i32,
}

impl SwayMode {
    #[allow(dead_code)]
    pub fn label(&self) -> String {
        let hz = self.refresh as f64 / 1000.0;
        format!("{}x{} @ {:.1}Hz", self.width, self.height, hz)
    }
}

impl SwayOutput {
    pub fn description(&self) -> String {
        format!("{} {} ({})", self.make.trim(), self.model.trim(), self.name)
    }

    /// Stable identifier using make/model/serial.
    /// Survives DP link resets (lock screen, DPMS) where port names change.
    pub fn stable_id(&self) -> String {
        let id = format!("{} {} {}", self.make.trim(), self.model.trim(), self.serial.trim());
        id.trim().to_string()
    }

    pub fn current_mode(&self) -> Option<&SwayMode> {
        // The current mode matches the rect dimensions
        self.modes.iter().find(|m| {
            m.width == self.rect.width && m.height == self.rect.height
        })
    }
}

pub fn query_outputs() -> Result<Vec<SwayOutput>, String> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_outputs"])
        .output()
        .map_err(|e| format!("Failed to run swaymsg: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("swaymsg failed: {stderr}"));
    }

    let json = String::from_utf8_lossy(&output.stdout);
    serde_json::from_str(&json).map_err(|e| format!("Failed to parse outputs: {e}"))
}

// ── Our output config ───────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputConfig {
    pub name: String,
    pub resolution: Option<String>,  // e.g. "2560x1440"
    pub refresh: Option<f64>,        // e.g. 59.951
    pub scale: Option<f64>,          // e.g. 1.0
    pub transform: Option<String>,   // e.g. "normal", "90", "180", "270"
    pub position: Option<OutputPosition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OutputPosition {
    Absolute { x: i32, y: i32 },
    LeftOf(String),
    RightOf(String),
    Above(String),
    Below(String),
}

impl OutputPosition {
    #[allow(dead_code)]
    pub fn label(&self) -> String {
        match self {
            OutputPosition::Absolute { x, y } => format!("Position {x}, {y}"),
            OutputPosition::LeftOf(name) => format!("Left of {name}"),
            OutputPosition::RightOf(name) => format!("Right of {name}"),
            OutputPosition::Above(name) => format!("Above {name}"),
            OutputPosition::Below(name) => format!("Below {name}"),
        }
    }

    pub fn relation_index(&self) -> usize {
        match self {
            OutputPosition::LeftOf(_) => 0,
            OutputPosition::RightOf(_) => 1,
            OutputPosition::Above(_) => 2,
            OutputPosition::Below(_) => 3,
            OutputPosition::Absolute { .. } => 4,
        }
    }

    pub fn target_name(&self) -> Option<&str> {
        match self {
            OutputPosition::LeftOf(n)
            | OutputPosition::RightOf(n)
            | OutputPosition::Above(n)
            | OutputPosition::Below(n) => Some(n),
            OutputPosition::Absolute { .. } => None,
        }
    }
}

pub const POSITION_RELATIONS: &[&str] = &["Left of", "Right of", "Above", "Below"];

/// Build an OutputConfig from current sway state
pub fn config_from_sway(output: &SwayOutput) -> OutputConfig {
    let mode = output.current_mode();
    OutputConfig {
        name: output.stable_id(),
        resolution: mode.map(|m| format!("{}x{}", m.width, m.height)),
        refresh: mode.map(|m| m.refresh as f64 / 1000.0),
        scale: Some(output.scale),
        transform: Some(output.transform.clone()),
        position: Some(OutputPosition::Absolute {
            x: output.rect.x,
            y: output.rect.y,
        }),
    }
}

/// Infer relative positions from absolute coordinates
/// Find a SwayOutput matching a config name (which is a stable_id).
fn find_output_by_config_name<'a>(outputs: &'a [SwayOutput], name: &str) -> Option<&'a SwayOutput> {
    outputs.iter().find(|o| o.stable_id() == name)
}

pub fn infer_relative_positions(configs: &mut Vec<OutputConfig>, outputs: &[SwayOutput]) {
    if configs.len() < 2 {
        return;
    }

    // Find the leftmost/topmost output as the "anchor"
    let anchor_idx = configs
        .iter()
        .enumerate()
        .min_by_key(|(_, c)| match &c.position {
            Some(OutputPosition::Absolute { x, y }) => (*x, *y),
            _ => (i32::MAX, i32::MAX),
        })
        .map(|(i, _)| i)
        .unwrap_or(0);

    let anchor_name = configs[anchor_idx].name.clone();
    let anchor_rect = find_output_by_config_name(outputs, &anchor_name)
        .map(|o| &o.rect);

    for i in 0..configs.len() {
        if i == anchor_idx {
            continue;
        }
        if let (Some(OutputPosition::Absolute { x, y }), Some(ar)) =
            (&configs[i].position, anchor_rect)
        {
            let x = *x;
            let y = *y;
            let pos = if x >= ar.x + ar.width && y.abs_diff(ar.y) as i32 <= ar.height / 2 {
                OutputPosition::RightOf(anchor_name.clone())
            } else if x + find_output_by_config_name(outputs, &configs[i].name).map_or(0, |o| o.rect.width) <= ar.x
                && y.abs_diff(ar.y) as i32 <= ar.height / 2
            {
                OutputPosition::LeftOf(anchor_name.clone())
            } else if y + find_output_by_config_name(outputs, &configs[i].name).map_or(0, |o| o.rect.height) <= ar.y {
                OutputPosition::Above(anchor_name.clone())
            } else if y >= ar.y + ar.height {
                OutputPosition::Below(anchor_name.clone())
            } else {
                OutputPosition::RightOf(anchor_name.clone())
            };
            configs[i].position = Some(pos);
        }
    }
}

// ── Persistence ─────────────────────────────────────────────

fn outputs_config_path() -> std::path::PathBuf {
    super::config::config_dir().join("outputs.json")
}

pub fn load_output_configs() -> Option<Vec<OutputConfig>> {
    let data = fs::read_to_string(outputs_config_path()).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_output_configs(configs: &[OutputConfig]) {
    if let Ok(json) = serde_json::to_string_pretty(configs) {
        fs::write(outputs_config_path(), json).ok();
    }
}

// ── Sway config generation ─────────────────────────────────

pub fn write_outputs_conf(configs: &[OutputConfig]) -> Result<(), String> {
    let oblong_dir = std::path::PathBuf::from(
        std::env::var("HOME").unwrap_or_else(|_| ".".into()),
    )
    .join(".config")
    .join("sway")
    .join("oblong");
    fs::create_dir_all(&oblong_dir).map_err(|e| e.to_string())?;

    let mut output = String::from("# ── Oblong outputs — auto-generated, do not edit by hand ──\n\n");

    // Resolve relative positions to absolute for sway config
    let resolved = resolve_positions(configs);

    for (conf, pos) in configs.iter().zip(resolved.iter()) {
        output.push_str(&format!("output \"{}\" {{\n", conf.name));

        if let Some(res) = &conf.resolution {
            if let Some(hz) = conf.refresh {
                output.push_str(&format!("    mode {}@{:.3}Hz\n", res, hz));
            } else {
                output.push_str(&format!("    mode {}\n", res));
            }
        }

        if let Some(scale) = conf.scale {
            output.push_str(&format!("    scale {}\n", scale));
        }

        if let Some(transform) = &conf.transform {
            if transform != "normal" {
                output.push_str(&format!("    transform {}\n", transform));
            }
        }

        if let Some((x, y)) = pos {
            output.push_str(&format!("    position {} {}\n", x, y));
        }

        output.push_str("}\n\n");
    }

    fs::write(oblong_dir.join("outputs.conf"), &output).map_err(|e| e.to_string())?;

    // Ensure include line is present
    super::config::ensure_include()?;

    Ok(())
}

/// Resolve relative positions to absolute (x, y) pairs
fn resolve_positions(configs: &[OutputConfig]) -> Vec<Option<(i32, i32)>> {
    let mut positions: Vec<Option<(i32, i32)>> = vec![None; configs.len()];

    // First pass: place outputs with absolute positions
    for (i, conf) in configs.iter().enumerate() {
        if let Some(OutputPosition::Absolute { x, y }) = &conf.position {
            positions[i] = Some((*x, *y));
        }
    }

    // If no anchor, place first at 0,0
    if positions.iter().all(|p| p.is_none()) && !configs.is_empty() {
        positions[0] = Some((0, 0));
    }

    // Second pass: resolve relative positions
    // Iterate a few times to handle chains
    for _ in 0..configs.len() {
        for i in 0..configs.len() {
            if positions[i].is_some() {
                continue;
            }
            if let Some(pos) = &configs[i].position {
                let target_name = pos.target_name();
                if let Some(target_name) = target_name {
                    // Find target's resolved position and its config for dimensions
                    let target = configs
                        .iter()
                        .enumerate()
                        .find(|(_, c)| c.name == target_name);
                    if let Some((ti, tc)) = target {
                        if let Some((tx, ty)) = positions[ti] {
                            let tw = parse_res_width(&tc.resolution);
                            let th = parse_res_height(&tc.resolution);
                            let resolved = match pos {
                                OutputPosition::RightOf(_) => (tx + tw, ty),
                                OutputPosition::LeftOf(_) => {
                                    let my_w = parse_res_width(&configs[i].resolution);
                                    (tx - my_w, ty)
                                }
                                OutputPosition::Below(_) => (tx, ty + th),
                                OutputPosition::Above(_) => {
                                    let my_h = parse_res_height(&configs[i].resolution);
                                    (tx, ty - my_h)
                                }
                                _ => continue,
                            };
                            positions[i] = Some(resolved);
                        }
                    }
                }
            }
        }
    }

    positions
}

fn parse_res_width(res: &Option<String>) -> i32 {
    res.as_ref()
        .and_then(|r| r.split('x').next()?.parse().ok())
        .unwrap_or(1920)
}

fn parse_res_height(res: &Option<String>) -> i32 {
    res.as_ref()
        .and_then(|r| r.split('x').nth(1)?.parse().ok())
        .unwrap_or(1080)
}

// ── Available modes as strings ──────────────────────────────

pub fn unique_modes(output: &SwayOutput) -> Vec<String> {
    let mut seen = Vec::new();
    for m in &output.modes {
        let res = format!("{}x{}", m.width, m.height);
        if !seen.contains(&res) {
            seen.push(res);
        }
    }
    seen
}

pub fn refresh_rates_for_resolution(output: &SwayOutput, resolution: &str) -> Vec<f64> {
    let parts: Vec<&str> = resolution.split('x').collect();
    if parts.len() != 2 {
        return vec![];
    }
    let w: i32 = parts[0].parse().unwrap_or(0);
    let h: i32 = parts[1].parse().unwrap_or(0);

    let mut rates: Vec<f64> = output
        .modes
        .iter()
        .filter(|m| m.width == w && m.height == h)
        .map(|m| m.refresh as f64 / 1000.0)
        .collect();
    rates.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
    rates.dedup_by(|a, b| (*a - *b).abs() < 0.5);
    rates
}
