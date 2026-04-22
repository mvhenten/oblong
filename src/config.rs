use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

// ── Data model ──────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    pub keys: String,
    pub command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BindingGroup {
    pub name: String,
    pub bindings: Vec<Binding>,
}

// ── Paths ───────────────────────────────────────────────────

pub fn sway_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".config").join("sway").join("config")
}

pub fn config_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    let dir = PathBuf::from(home).join(".config").join("oblong");
    fs::create_dir_all(&dir).ok();
    dir
}

pub fn config_path() -> PathBuf {
    config_dir().join("bindings.json")
}

// ── Config persistence ──────────────────────────────────────

pub fn load_config() -> Option<Vec<BindingGroup>> {
    let data = fs::read_to_string(config_path()).ok()?;
    serde_json::from_str(&data).ok()
}

pub fn save_config(groups: &[BindingGroup]) {
    if let Ok(json) = serde_json::to_string_pretty(groups) {
        fs::write(config_path(), json).ok();
    }
}

// ── Sway config parser ─────────────────────────────────────

pub fn parse_sway_bindings(content: &str) -> Vec<Binding> {
    let mut bindings = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        if let Some(rest) = trimmed.strip_prefix("bindsym ") {
            let mut full = String::new();
            let first_line = rest.trim();

            if first_line.ends_with('\\') {
                full.push_str(first_line.trim_end_matches('\\').trim());
                i += 1;
                while i < lines.len() {
                    let cont = lines[i].trim();
                    if cont.ends_with('\\') {
                        full.push(' ');
                        full.push_str(cont.trim_end_matches('\\').trim());
                        i += 1;
                    } else {
                        full.push(' ');
                        full.push_str(cont);
                        i += 1;
                        break;
                    }
                }
            } else {
                full = first_line.to_string();
                i += 1;
            }

            let full = full.trim().to_string();
            if let Some(split_pos) = full.find(|c: char| c == ' ' || c == '\t') {
                let keys = full[..split_pos].trim().to_string();
                let command = full[split_pos..].trim().to_string();
                bindings.push(Binding { keys, command });
            }
        } else {
            i += 1;
        }
    }

    bindings
}

// ── Categorization ──────────────────────────────────────────

pub fn categorize(b: &Binding) -> &'static str {
    let cmd = b.command.to_lowercase();

    if cmd.contains("oblong snap") {
        "Window Snapping"
    } else if cmd.contains("oblong switch") {
        "Window Switching"
    } else if cmd.contains("floating enable") && (cmd.contains("resize set") || cmd.contains("move position")) {
        "Window Snapping"
    } else if cmd.contains("floating disable") || cmd.contains("floating toggle") {
        "Window Snapping"
    } else if cmd.contains("move container to output") {
        "Displays"
    } else if cmd.starts_with("focus ") {
        "Focus"
    } else if cmd.starts_with("move ") && !cmd.contains("workspace") && !cmd.contains("output") {
        "Move Window"
    } else if cmd.contains("workspace") {
        "Workspaces"
    } else if cmd.starts_with("layout ") || cmd.starts_with("fullscreen") {
        "Layout"
    } else if cmd.starts_with("exec ") {
        "Apps & Commands"
    } else if cmd == "kill" {
        "Apps & Commands"
    } else {
        "Other"
    }
}

pub fn group_bindings(bindings: Vec<Binding>) -> Vec<BindingGroup> {
    let category_order = [
        "Window Snapping",
        "Window Switching",
        "Displays",
        "Focus",
        "Move Window",
        "Workspaces",
        "Layout",
        "Apps & Commands",
        "Other",
    ];

    let mut groups: Vec<BindingGroup> = category_order
        .iter()
        .map(|name| BindingGroup {
            name: name.to_string(),
            bindings: Vec::new(),
        })
        .collect();

    for b in bindings {
        let cat = categorize(&b);
        if let Some(group) = groups.iter_mut().find(|g| g.name == cat) {
            group.bindings.push(b);
        }
    }

    groups.retain(|g| !g.bindings.is_empty());
    groups
}

// ── Label generation ────────────────────────────────────────

pub fn label_for_command(cmd: &str) -> String {
    let cmd_lower = cmd.to_lowercase();

    // Snap commands from our CLI
    if cmd_lower.contains("oblong snap") {
        if let Some(dir) = cmd_lower.split_whitespace().last() {
            return match dir {
                "left" => "Snap Left (½ → ⅓ → ⅔)",
                "right" => "Snap Right (½ → ⅓ → ⅔)",
                "up" => "Snap Top (½ → ⅓ → ⅔)",
                "down" => "Snap Bottom (½ → ⅓ → ⅔)",
                "topleft" => "Top Left (½ → ⅓ → ⅔)",
                "topright" => "Top Right (½ → ⅓ → ⅔)",
                "bottomleft" => "Bottom Left (½ → ⅓ → ⅔)",
                "bottomright" => "Bottom Right (½ → ⅓ → ⅔)",
                "maximize" => "Maximize",
                "center" => "Center (⅔ → ½ → ⅓)",
                "restore" => "Restore Tiling",
                other => return format!("Snap {other}"),
            }.into();
        }
    }

    // Switch commands
    if cmd_lower.contains("oblong switch") {
        if let Some(dir) = cmd_lower.split_whitespace().last() {
            return match dir {
                "next" => "Next Window",
                "prev" => "Previous Window",
                other => return format!("Switch {other}"),
            }.into();
        }
    }

    // Legacy inline commands
    if cmd_lower.contains("width 100ppt height 100ppt") && cmd_lower.contains("move position 0 0") {
        return "Maximize".into();
    }
    if cmd_lower.contains("width 50ppt height 100ppt") && cmd_lower.contains("position 0 0") && !cmd_lower.contains("50ppt 0") {
        return "Left Half".into();
    }
    if cmd_lower.contains("width 50ppt height 100ppt") && cmd_lower.contains("50ppt 0") {
        return "Right Half".into();
    }
    if cmd_lower.contains("width 100ppt height 50ppt") && cmd_lower.contains("position 0 0") {
        return "Top Half".into();
    }
    if cmd_lower.contains("width 100ppt height 50ppt") && cmd_lower.contains("0 50ppt") {
        return "Bottom Half".into();
    }
    if cmd_lower == "floating disable" { return "Restore Tiling".into(); }
    if cmd_lower == "floating toggle" { return "Toggle Floating".into(); }
    if cmd_lower == "fullscreen toggle" { return "Toggle Fullscreen".into(); }

    // Displays
    if cmd_lower.contains("output left") { return "Move to Prev Display".into(); }
    if cmd_lower.contains("output right") { return "Move to Next Display".into(); }

    // Focus
    if cmd_lower == "focus left" { return "Focus Left".into(); }
    if cmd_lower == "focus right" { return "Focus Right".into(); }
    if cmd_lower == "focus up" { return "Focus Up".into(); }
    if cmd_lower == "focus down" { return "Focus Down".into(); }

    // Move
    if cmd_lower == "move left" { return "Move Left".into(); }
    if cmd_lower == "move right" { return "Move Right".into(); }
    if cmd_lower == "move up" { return "Move Up".into(); }
    if cmd_lower == "move down" { return "Move Down".into(); }

    // Workspaces
    if let Some(n) = cmd_lower.strip_prefix("workspace number ") {
        return format!("Workspace {n}");
    }
    if cmd_lower.contains("move container to workspace number") {
        let n = cmd_lower.split_whitespace().last().unwrap_or("?");
        return format!("Move to Workspace {n}");
    }
    if cmd_lower == "workspace back_and_forth" { return "Last Workspace".into(); }

    // Layout
    if cmd_lower == "layout stacking" { return "Stacking Layout".into(); }
    if cmd_lower == "layout tabbed" { return "Tabbed Layout".into(); }
    if cmd_lower == "layout toggle split" { return "Toggle Split".into(); }

    // Apps
    if cmd_lower == "kill" { return "Close Window".into(); }
    if cmd_lower.contains("$launcher") || cmd_lower.contains("fuzzel") { return "App Launcher".into(); }
    if cmd_lower.contains("$term") || cmd_lower.contains("terminal") { return "Terminal".into(); }
    if cmd_lower.contains("grim") && cmd_lower.contains("slurp") { return "Screenshot (region)".into(); }
    if cmd_lower.contains("grim") { return "Screenshot".into(); }
    if cmd_lower.contains("swaymsg exit") { return "Exit Sway".into(); }

    cmd.to_string()
}

// ── Sway config paths ───────────────────────────────────────

fn sway_oblong_dir() -> PathBuf {
    PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
        .join(".config")
        .join("sway")
        .join("oblong")
}

// ── Include management ──────────────────────────────────────

const INCLUDE_LINE: &str = "include ~/.config/sway/oblong/*.conf";

/// Ensure the wildcard include line is present in the main sway config.
/// Idempotent — adds it at the end if missing, never duplicates.
pub fn ensure_include() -> Result<(), String> {
    let main_config = sway_config_path();
    let content = fs::read_to_string(&main_config).map_err(|e| e.to_string())?;

    if content.lines().any(|l| l.trim() == INCLUDE_LINE) {
        return Ok(());
    }

    let mut updated = content.trim_end().to_string();
    updated.push_str(&format!("\n\n# ── Oblong window management ──\n{}\n", INCLUDE_LINE));
    fs::write(&main_config, updated).map_err(|e| e.to_string())?;

    Ok(())
}

// ── Conflict detection ──────────────────────────────────────

/// Find keys that are bound in both the main sway config and our managed bindings.
pub fn detect_conflicts(managed: &[&Binding]) -> Vec<String> {
    let content = fs::read_to_string(sway_config_path()).unwrap_or_default();
    let sway_bindings = parse_sway_bindings(&content);
    let managed_keys: Vec<&str> = managed.iter().map(|b| b.keys.trim()).collect();

    sway_bindings
        .iter()
        .filter(|b| managed_keys.contains(&b.keys.trim()))
        .map(|b| b.keys.clone())
        .collect()
}

/// Comment out conflicting bindsyms in the main sway config.
/// One-time operation for import flow. Marks lines clearly so user can undo.
pub fn comment_out_conflicts(keys_to_disable: &[String]) -> Result<usize, String> {
    let main_config = sway_config_path();
    let content = fs::read_to_string(&main_config).map_err(|e| e.to_string())?;
    let lines: Vec<&str> = content.lines().collect();
    let mut output: Vec<String> = Vec::new();
    let mut count = 0;
    let mut i = 0;

    while i < lines.len() {
        let trimmed = lines[i].trim();

        let conflict = if let Some(rest) = trimmed.strip_prefix("bindsym ") {
            keys_to_disable.iter().any(|key| {
                rest.starts_with(&format!("{} ", key))
                    || rest.starts_with(&format!("{}\\" , key))
            })
        } else {
            false
        };

        if conflict {
            output.push(format!("# [oblong] {}", lines[i]));
            count += 1;
            // Handle continuation lines
            while i < lines.len() && lines[i].trim_end().ends_with('\\') {
                i += 1;
                if i < lines.len() {
                    output.push(format!("# [oblong] {}", lines[i]));
                }
            }
            i += 1;
        } else {
            output.push(lines[i].to_string());
            i += 1;
        }
    }

    fs::write(&main_config, output.join("\n")).map_err(|e| e.to_string())?;
    Ok(count)
}

// ── Sway config writing ────────────────────────────────────

/// Write managed bindings to ~/.config/sway/oblong/shortcuts.conf
/// and ensure the include line is present. Does NOT modify user bindings.
pub fn write_sway_config(bindings: &[&Binding]) -> Result<(), String> {
    let oblong_dir = sway_oblong_dir();
    fs::create_dir_all(&oblong_dir).map_err(|e| e.to_string())?;

    let conf_file = oblong_dir.join("shortcuts.conf");

    let mut output = String::from(
        "# ── Oblong — auto-generated, do not edit by hand ──\n\n",
    );

    for b in bindings {
        if b.keys.trim().is_empty() {
            continue;
        }
        let label = label_for_command(&b.command);
        output.push_str(&format!("# {}\nbindsym {} {}\n\n", label, b.keys, b.command));
    }

    fs::write(&conf_file, &output).map_err(|e| e.to_string())?;

    ensure_include()?;

    Ok(())
}
