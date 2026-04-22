use serde::Deserialize;
use std::process::Command;

/// macOS-style window switcher: cycles through all windows across
/// all outputs, raises them, and gives focus.

#[derive(Debug, Deserialize)]
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
    nodes: Vec<Node>,
    #[serde(default)]
    floating_nodes: Vec<Node>,
}

#[derive(Debug)]
struct WindowInfo {
    id: i64,
    focused: bool,
}

fn collect_windows(node: &Node, windows: &mut Vec<WindowInfo>) {
    let is_leaf = node.nodes.is_empty() && node.floating_nodes.is_empty();
    let is_window = matches!(
        node.node_type.as_deref(),
        Some("con") | Some("floating_con")
    );
    let has_content = node.name.as_ref().map_or(false, |n| !n.is_empty())
        || node.app_id.as_ref().map_or(false, |a| !a.is_empty());

    if is_leaf && is_window && has_content {
        windows.push(WindowInfo {
            id: node.id,
            focused: node.focused,
        });
    }

    for child in &node.nodes {
        collect_windows(child, windows);
    }
    for child in &node.floating_nodes {
        collect_windows(child, windows);
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

    let tree: Node = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse tree: {e}"))?;

    let mut windows = Vec::new();
    collect_windows(&tree, &mut windows);
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

pub fn switch(direction: &str) -> Result<(), String> {
    let windows = get_all_windows()?;
    if windows.len() <= 1 {
        return Ok(()); // nothing to switch to
    }

    let focused_idx = windows.iter().position(|w| w.focused);

    let target_idx = match direction {
        "next" => match focused_idx {
            Some(i) => (i + 1) % windows.len(),
            None => 0,
        },
        "prev" => match focused_idx {
            Some(0) => windows.len() - 1,
            Some(i) => i - 1,
            None => 0,
        },
        _ => return Err(format!("Unknown direction: {direction}. Use: next, prev")),
    };

    focus_window(windows[target_idx].id)
}
