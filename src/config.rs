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
    PathBuf::from(home)
        .join(".config")
        .join("sway")
        .join("config")
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
            if let Some(split_pos) = full.find([' ', '\t']) {
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
    } else if (cmd.contains("floating enable")
        && (cmd.contains("resize set") || cmd.contains("move position")))
        || cmd.contains("floating disable")
        || cmd.contains("floating toggle")
    {
        "Window Snapping"
    } else if cmd.contains("move container to output") {
        "Displays"
    } else if cmd.starts_with("focus ") {
        "Focus"
    } else if cmd.starts_with("move ") && !cmd.contains("workspace") && !cmd.contains("output") {
        "Move Window"
    } else if cmd.contains("workspace") {
        "Workspaces"
    } else if cmd.starts_with("layout ")
        || cmd.starts_with("fullscreen")
        || cmd.starts_with("split")
        || cmd.starts_with("mode ")
    {
        "Layout"
    } else if cmd.starts_with("exec ") || cmd == "kill" || cmd == "reload" {
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
            }
            .into();
        }
    }

    // Switch commands
    if cmd_lower.contains("oblong switch") {
        if let Some(dir) = cmd_lower.split_whitespace().last() {
            return match dir {
                "next" => "Next Window",
                "prev" => "Previous Window",
                other => return format!("Switch {other}"),
            }
            .into();
        }
    }

    // Legacy inline commands
    if cmd_lower.contains("width 100ppt height 100ppt") && cmd_lower.contains("move position 0 0") {
        return "Maximize".into();
    }
    if cmd_lower.contains("width 50ppt height 100ppt")
        && cmd_lower.contains("position 0 0")
        && !cmd_lower.contains("50ppt 0")
    {
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
    if cmd_lower == "floating disable" {
        return "Restore Tiling".into();
    }
    if cmd_lower == "floating toggle" {
        return "Toggle Floating".into();
    }
    if cmd_lower == "fullscreen toggle" || cmd_lower == "fullscreen" {
        return "Toggle Fullscreen".into();
    }

    // Displays
    if cmd_lower.contains("output left") {
        return "Move to Prev Display".into();
    }
    if cmd_lower.contains("output right") {
        return "Move to Next Display".into();
    }

    // Focus
    if cmd_lower == "focus left" {
        return "Focus Left".into();
    }
    if cmd_lower == "focus right" {
        return "Focus Right".into();
    }
    if cmd_lower == "focus up" {
        return "Focus Up".into();
    }
    if cmd_lower == "focus down" {
        return "Focus Down".into();
    }

    // Move
    if cmd_lower == "move left" {
        return "Move Left".into();
    }
    if cmd_lower == "move right" {
        return "Move Right".into();
    }
    if cmd_lower == "move up" {
        return "Move Up".into();
    }
    if cmd_lower == "move down" {
        return "Move Down".into();
    }

    // Workspaces
    if let Some(n) = cmd_lower.strip_prefix("workspace number ") {
        return format!("Workspace {n}");
    }
    if cmd_lower.contains("move container to workspace number") {
        let n = cmd_lower.split_whitespace().last().unwrap_or("?");
        return format!("Move to Workspace {n}");
    }
    if cmd_lower == "workspace back_and_forth" {
        return "Last Workspace".into();
    }

    // Layout
    if cmd_lower == "layout stacking" {
        return "Stacking Layout".into();
    }
    if cmd_lower == "layout tabbed" {
        return "Tabbed Layout".into();
    }
    if cmd_lower == "layout toggle split" {
        return "Toggle Split".into();
    }
    if cmd_lower == "splith" {
        return "Split Horizontal".into();
    }
    if cmd_lower == "splitv" {
        return "Split Vertical".into();
    }
    if cmd_lower == "mode \"resize\"" {
        return "Resize Mode".into();
    }
    if cmd_lower == "reload" {
        return "Reload Sway".into();
    }

    // Apps
    if cmd_lower == "kill" {
        return "Close Window".into();
    }
    if cmd_lower.contains("$launcher") || cmd_lower.contains("fuzzel") {
        return "App Launcher".into();
    }
    if cmd_lower.contains("$term") || cmd_lower.contains("terminal") {
        return "Terminal".into();
    }
    if cmd_lower.contains("grim") && cmd_lower.contains("slurp") {
        return "Screenshot (region)".into();
    }
    if cmd_lower.contains("grim") {
        return "Screenshot".into();
    }
    if cmd_lower.contains("swaymsg exit") {
        return "Exit Sway".into();
    }

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
    updated.push_str(&format!(
        "\n\n# ── Oblong window management ──\n{}\n",
        INCLUDE_LINE
    ));
    fs::write(&main_config, updated).map_err(|e| e.to_string())?;

    Ok(())
}

// ── Conflict detection ──────────────────────────────────────

/// Find main sway config bindings that clash with our managed bindings: the
/// same key combo, or a command oblong now binds to a different key.
pub fn detect_conflicts(managed: &[&Binding]) -> Vec<String> {
    let content = fs::read_to_string(sway_config_path()).unwrap_or_default();
    conflicting_keys(&parse_sway_bindings(&content), managed)
}

fn conflicting_keys(sway_bindings: &[Binding], managed: &[&Binding]) -> Vec<String> {
    sway_bindings
        .iter()
        .filter(|b| {
            managed
                .iter()
                .any(|m| m.keys.trim() == b.keys.trim() || m.command.trim() == b.command.trim())
        })
        .map(|b| b.keys.clone())
        .collect()
}

// ── Default keymap ─────────────────────────────────────────

const SWAY_EXIT: &str = "exec swaynag -t warning -m 'You pressed the exit shortcut. Do you really want to exit sway? This will end your Wayland session.' -B 'Yes, exit sway' 'swaymsg exit'";

/// The default window-management keymap: Rectangle's Ctrl+Option chords for
/// snapping and sway's window commands, leaving Cmd ($mod) to app shortcuts.
pub fn default_bindings(bin: &str) -> Vec<Binding> {
    let bind = |keys: &str, command: String| Binding {
        keys: keys.into(),
        command,
    };
    let snap = |keys: &str, dir: &str| bind(keys, format!("exec {bin} snap {dir}"));

    let mut bindings = vec![
        snap("Ctrl+Mod1+Left", "left"),
        snap("Ctrl+Mod1+Right", "right"),
        snap("Ctrl+Mod1+Up", "up"),
        snap("Ctrl+Mod1+Down", "down"),
        snap("Ctrl+Mod1+Return", "maximize"),
        snap("Ctrl+Mod1+c", "center"),
        snap("Ctrl+Mod1+BackSpace", "restore"),
        snap("Ctrl+Mod1+u", "topleft"),
        snap("Ctrl+Mod1+i", "topright"),
        snap("Ctrl+Mod1+j", "bottomleft"),
        snap("Ctrl+Mod1+k", "bottomright"),
        bind("$mod+Tab", format!("exec {bin} switch next")),
        bind("$mod+Shift+Tab", format!("exec {bin} switch prev")),
    ];

    for dir in ["Left", "Right", "Up", "Down"] {
        let lower = dir.to_lowercase();
        bindings.push(bind(
            &format!("Ctrl+Mod1+Shift+{dir}"),
            format!("focus {lower}"),
        ));
        bindings.push(bind(
            &format!("$mod+Ctrl+Mod1+{dir}"),
            format!("move {lower}"),
        ));
    }

    for n in 1..=9 {
        bindings.push(bind(&format!("Ctrl+{n}"), format!("workspace number {n}")));
        bindings.push(bind(
            &format!("Ctrl+Shift+{n}"),
            format!("move container to workspace number {n}"),
        ));
    }

    bindings.extend([
        bind("$mod+Ctrl+f", "fullscreen".into()),
        bind("Ctrl+Mod1+r", "mode \"resize\"".into()),
        bind("Ctrl+Mod1+s", "layout stacking".into()),
        bind("Ctrl+Mod1+w", "layout tabbed".into()),
        bind("Ctrl+Mod1+e", "layout toggle split".into()),
        bind("Ctrl+Mod1+b", "splith".into()),
        bind("Ctrl+Mod1+v", "splitv".into()),
        bind("Ctrl+Mod1+Shift+r", "reload".into()),
        bind("$mod+Shift+q", SWAY_EXIT.into()),
    ]);

    bindings
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
                rest.starts_with(&format!("{} ", key)) || rest.starts_with(&format!("{}\\", key))
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

/// Render bindings as sway config. A key combo bound more than once keeps
/// only its last binding, matching sway's own "later binding wins" rule
/// without the overwrite warning.
pub fn render_shortcuts(bindings: &[&Binding]) -> String {
    let mut output = String::from("# ── Oblong — auto-generated, do not edit by hand ──\n\n");

    for (i, b) in bindings.iter().enumerate() {
        let keys = b.keys.trim();
        if keys.is_empty() {
            continue;
        }
        if bindings[i + 1..]
            .iter()
            .any(|later| later.keys.trim() == keys)
        {
            continue;
        }
        let label = label_for_command(&b.command);
        output.push_str(&format!("# {}\nbindsym {} {}\n\n", label, keys, b.command));
    }

    output
}

/// Write managed bindings to ~/.config/sway/oblong/shortcuts.conf
/// and ensure the include line is present. Does NOT modify user bindings.
pub fn write_sway_config(bindings: &[&Binding]) -> Result<(), String> {
    let oblong_dir = sway_oblong_dir();
    fs::create_dir_all(&oblong_dir).map_err(|e| e.to_string())?;

    let conf_file = oblong_dir.join("shortcuts.conf");

    fs::write(&conf_file, render_shortcuts(bindings)).map_err(|e| e.to_string())?;

    ensure_include()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_shortcuts_binds_each_key_combo_once_and_last_wins() {
        let sway = "\
bindsym Mod4+Shift+3 move container to workspace number 3
bindsym Mod4+Shift+4 move container to workspace number 4
bindsym Mod4+Shift+5 move container to workspace number 5
bindsym Mod4+Shift+3 exec grim ~/Pictures/screenshot-full.png
bindsym Mod4+Shift+4 exec grim -g \"$(slurp)\" ~/Pictures/screenshot-region.png
bindsym Mod4+Shift+5 exec grim ~/Pictures/screenshot-window.png
";
        let groups = group_bindings(parse_sway_bindings(sway));
        let all: Vec<&Binding> = groups.iter().flat_map(|g| &g.bindings).collect();
        let rendered = render_shortcuts(&all);

        let bound = parse_sway_bindings(&rendered);
        let mut keys: Vec<&str> = bound.iter().map(|b| b.keys.as_str()).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(
            keys.len(),
            bound.len(),
            "duplicate key combo in:\n{rendered}"
        );
        assert_eq!(bound.len(), 3);
        assert!(bound.iter().all(|b| b.command.contains("grim")));
    }

    #[test]
    fn conflicts_include_sway_window_bindings_the_default_keymap_moved() {
        let sway = parse_sway_bindings(
            "\
bindsym $mod+Left focus left
bindsym $mod+1 workspace number 1
bindsym $mod+Return exec $term
bindsym $mod+Tab exec something-else
",
        );
        let defaults = default_bindings("oblong");
        let managed: Vec<&Binding> = defaults.iter().collect();

        assert_eq!(
            conflicting_keys(&sway, &managed),
            vec!["$mod+Left", "$mod+1", "$mod+Tab"]
        );
    }
}
