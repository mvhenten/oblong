use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

/// macOS-style window switcher: cycles through all windows across
/// all outputs in most-recently-used order, using sway's focus
/// history from the tree.

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Node {
    id: i64,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    app_id: Option<String>,
    #[serde(rename = "type")]
    node_type: Option<String>,
    #[serde(default)]
    focused: bool,
    #[serde(default)]
    focus: Vec<i64>,
    #[serde(default)]
    nodes: Vec<Node>,
    #[serde(default)]
    floating_nodes: Vec<Node>,
}

#[derive(Debug)]
struct WindowInfo {
    id: i64,
}

/// Walk the tree in MRU order: at each container, visit children
/// in the order given by the `focus` array (most recent first).
fn collect_windows_mru(node: &Node, windows: &mut Vec<WindowInfo>) {
    let is_leaf = node.nodes.is_empty() && node.floating_nodes.is_empty();
    let is_window = matches!(
        node.node_type.as_deref(),
        Some("con") | Some("floating_con")
    );
    let has_content = node.name.as_ref().is_some_and(|n| !n.is_empty())
        || node.app_id.as_ref().is_some_and(|a| !a.is_empty());

    if is_leaf && is_window && has_content {
        windows.push(WindowInfo { id: node.id });
        return;
    }

    // Build a map of id -> child node for quick lookup
    let all_children: Vec<&Node> = node
        .nodes
        .iter()
        .chain(node.floating_nodes.iter())
        .collect();
    let child_map: HashMap<i64, &Node> = all_children.iter().map(|c| (c.id, *c)).collect();

    // Visit in focus order (MRU first), then any children not in the focus list
    for &child_id in &node.focus {
        if let Some(child) = child_map.get(&child_id) {
            collect_windows_mru(child, windows);
        }
    }
    for child in &all_children {
        if !node.focus.contains(&child.id) {
            collect_windows_mru(child, windows);
        }
    }
}

fn get_all_windows() -> Result<Vec<WindowInfo>, String> {
    let output = Command::new("swaymsg")
        .args(["-t", "get_tree"])
        .output()
        .map_err(|e| format!("Failed to run swaymsg: {e}"))?;

    if !output.status.success() {
        return Err("swaymsg get_tree failed".into());
    }

    let tree: Node =
        serde_json::from_slice(&output.stdout).map_err(|e| format!("Failed to parse tree: {e}"))?;

    let mut windows = Vec::new();
    collect_windows_mru(&tree, &mut windows);
    Ok(windows)
}

fn focus_window(id: i64) -> Result<(), String> {
    let cmd = format!("[con_id={}] focus", id);
    let output = Command::new("swaymsg")
        .arg(&cmd)
        .output()
        .map_err(|e| format!("swaymsg failed: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("focus failed: {stderr}"));
    }
    Ok(())
}

// ── Cycle state persistence ─────────────────────────────────
// When rapidly pressing Tab, we walk further down the MRU list.
// A timeout resets the cycle so the next Tab goes to the previous window.

const CYCLE_TIMEOUT_MS: u64 = 1500;

fn state_path() -> PathBuf {
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".into());
    PathBuf::from(dir).join("oblong-switch-state")
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

struct SwitchState {
    /// The MRU window id list at the start of this cycle
    window_ids: Vec<i64>,
    /// Current position in the cycle
    index: usize,
    timestamp: u64,
}

fn load_switch_state() -> Option<SwitchState> {
    let content = fs::read_to_string(state_path()).ok()?;
    let mut lines = content.lines();
    let ids_str = lines.next()?;
    let index: usize = lines.next()?.parse().ok()?;
    let timestamp: u64 = lines.next()?.parse().ok()?;
    let window_ids: Vec<i64> = ids_str
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();
    Some(SwitchState {
        window_ids,
        index,
        timestamp,
    })
}

fn save_switch_state(state: &SwitchState) {
    if let Ok(mut f) = fs::File::create(state_path()) {
        let ids: Vec<String> = state.window_ids.iter().map(|id| id.to_string()).collect();
        writeln!(f, "{}", ids.join(",")).ok();
        writeln!(f, "{}", state.index).ok();
        writeln!(f, "{}", now_millis()).ok();
    }
}

pub fn switch(direction: &str) -> Result<(), String> {
    if direction != "next" && direction != "prev" {
        return Err(format!("Unknown direction: {direction}. Use: next, prev"));
    }

    let windows = get_all_windows()?;
    if windows.len() <= 1 {
        return Ok(());
    }

    let current_ids: Vec<i64> = windows.iter().map(|w| w.id).collect();

    // Check if we're continuing an existing cycle.
    // We freeze the MRU list from the first press and keep using it,
    // because focusing a window changes sway's MRU order.
    let (cycle_ids, index) = if let Some(state) = load_switch_state() {
        let elapsed = now_millis().saturating_sub(state.timestamp);
        // Continue if within timeout and all saved windows still exist
        let all_exist = state.window_ids.iter().all(|id| current_ids.contains(id));
        if elapsed < CYCLE_TIMEOUT_MS && all_exist && !state.window_ids.is_empty() {
            let new_index = match direction {
                "next" => (state.index + 1) % state.window_ids.len(),
                "prev" => {
                    if state.index == 0 {
                        state.window_ids.len() - 1
                    } else {
                        state.index - 1
                    }
                }
                _ => unreachable!(),
            };
            (state.window_ids, new_index)
        } else {
            // Timeout or windows changed — start fresh with current MRU
            let idx = match direction {
                "next" => 1,
                "prev" => current_ids.len() - 1,
                _ => unreachable!(),
            };
            (current_ids, idx)
        }
    } else {
        let idx = match direction {
            "next" => 1,
            "prev" => current_ids.len() - 1,
            _ => unreachable!(),
        };
        (current_ids, idx)
    };

    let target_id = cycle_ids[index];

    // Save state BEFORE focusing, so the MRU list we saved stays stable
    save_switch_state(&SwitchState {
        window_ids: cycle_ids,
        index,
        timestamp: 0, // filled by save
    });

    focus_window(target_id)
}
