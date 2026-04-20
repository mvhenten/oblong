use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// State file tracks what we last did, so repeated presses cycle through sizes.
fn state_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR")
        .unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(dir).join("oblong-state")
}

#[derive(Debug)]
struct SnapState {
    direction: String,
    step: usize,
    timestamp: u64,
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn load_state() -> Option<SnapState> {
    let content = fs::read_to_string(state_path()).ok()?;
    let mut lines = content.lines();
    let direction = lines.next()?.to_string();
    let step: usize = lines.next()?.parse().ok()?;
    let timestamp: u64 = lines.next()?.parse().ok()?;
    Some(SnapState { direction, step, timestamp })
}

fn save_state(direction: &str, step: usize) {
    if let Ok(mut f) = fs::File::create(state_path()) {
        writeln!(f, "{}\n{}\n{}", direction, step, now_millis()).ok();
    }
}

const CYCLE_TIMEOUT_MS: u64 = 1500;

/// The size fractions to cycle through for edge snapping.
/// Each entry is (width_pct, height_pct, x_expr, y_expr).
struct SnapGeometry {
    w: &'static str,
    h: &'static str,
    x: &'static str,
    y: &'static str,
}

fn edge_cycle(direction: &str) -> Vec<SnapGeometry> {
    match direction {
        "left" => vec![
            SnapGeometry { w: "50ppt",  h: "100ppt", x: "0", y: "0" },
            SnapGeometry { w: "33ppt",  h: "100ppt", x: "0", y: "0" },
            SnapGeometry { w: "67ppt",  h: "100ppt", x: "0", y: "0" },
        ],
        "right" => vec![
            SnapGeometry { w: "50ppt",  h: "100ppt", x: "50ppt",  y: "0" },
            SnapGeometry { w: "33ppt",  h: "100ppt", x: "67ppt",  y: "0" },
            SnapGeometry { w: "67ppt",  h: "100ppt", x: "33ppt",  y: "0" },
        ],
        "up" => vec![
            SnapGeometry { w: "100ppt", h: "50ppt",  x: "0", y: "0" },
            SnapGeometry { w: "100ppt", h: "33ppt",  x: "0", y: "0" },
            SnapGeometry { w: "100ppt", h: "67ppt",  x: "0", y: "0" },
        ],
        "down" => vec![
            SnapGeometry { w: "100ppt", h: "50ppt",  x: "0", y: "50ppt" },
            SnapGeometry { w: "100ppt", h: "33ppt",  x: "0", y: "67ppt" },
            SnapGeometry { w: "100ppt", h: "67ppt",  x: "0", y: "33ppt" },
        ],
        "topleft" => vec![
            SnapGeometry { w: "50ppt",  h: "50ppt",  x: "0", y: "0" },
            SnapGeometry { w: "33ppt",  h: "50ppt",  x: "0", y: "0" },
            SnapGeometry { w: "67ppt",  h: "50ppt",  x: "0", y: "0" },
        ],
        "topright" => vec![
            SnapGeometry { w: "50ppt",  h: "50ppt",  x: "50ppt",  y: "0" },
            SnapGeometry { w: "33ppt",  h: "50ppt",  x: "67ppt",  y: "0" },
            SnapGeometry { w: "67ppt",  h: "50ppt",  x: "33ppt",  y: "0" },
        ],
        "bottomleft" => vec![
            SnapGeometry { w: "50ppt",  h: "50ppt",  x: "0", y: "50ppt" },
            SnapGeometry { w: "33ppt",  h: "50ppt",  x: "0", y: "50ppt" },
            SnapGeometry { w: "67ppt",  h: "50ppt",  x: "0", y: "50ppt" },
        ],
        "bottomright" => vec![
            SnapGeometry { w: "50ppt",  h: "50ppt",  x: "50ppt",  y: "50ppt" },
            SnapGeometry { w: "33ppt",  h: "50ppt",  x: "67ppt",  y: "50ppt" },
            SnapGeometry { w: "67ppt",  h: "50ppt",  x: "33ppt",  y: "50ppt" },
        ],
        "center" => vec![
            SnapGeometry { w: "60ppt",  h: "80ppt",  x: "center", y: "center" },
            SnapGeometry { w: "50ppt",  h: "70ppt",  x: "center", y: "center" },
            SnapGeometry { w: "33ppt",  h: "60ppt",  x: "center", y: "center" },
        ],
        _ => vec![],
    }
}

fn swaymsg(cmd: &str) -> Result<(), String> {
    let output = Command::new("swaymsg")
        .arg(cmd)
        .output()
        .map_err(|e| format!("Failed to run swaymsg: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("swaymsg failed: {stderr}"));
    }
    Ok(())
}

pub fn snap(direction: &str) -> Result<(), String> {
    match direction {
        "maximize" => {
            swaymsg("floating enable, resize set width 100ppt height 100ppt, move position 0 0")?;
            save_state(direction, 0);
            return Ok(());
        }
        "restore" => {
            swaymsg("floating disable")?;
            save_state(direction, 0);
            return Ok(());
        }
        _ => {}
    }

    let cycle = edge_cycle(direction);
    if cycle.is_empty() {
        return Err(format!("Unknown direction: {direction}. Use: left, right, up, down, topleft, topright, bottomleft, bottomright, center, maximize, restore"));
    }

    // Determine which step we're on
    let step = if let Some(state) = load_state() {
        let elapsed = now_millis().saturating_sub(state.timestamp);
        if state.direction == direction && elapsed < CYCLE_TIMEOUT_MS {
            (state.step + 1) % cycle.len()
        } else {
            0
        }
    } else {
        0
    };

    let geo = &cycle[step];

    let cmd = if geo.x == "center" {
        format!(
            "floating enable, resize set width {} height {}, move position center",
            geo.w, geo.h
        )
    } else {
        format!(
            "floating enable, resize set width {} height {}, move position {} {}",
            geo.w, geo.h, geo.x, geo.y
        )
    };

    swaymsg(&cmd)?;
    save_state(direction, step);

    Ok(())
}
