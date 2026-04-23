# Oblong

Window management for Sway with a macOS-like feel. Snap windows, switch with MRU order, use Super+C/V for copy/paste, and configure everything through a GUI.

![Shortcuts](https://raw.githubusercontent.com/mvhenten/oblong/screenshots/shortcuts.png)

## Features

- **Window snapping** — halves, thirds, two-thirds, corners, center. Cycles on repeat.
- **MRU window switching** — Super+Tab cycles most-recently-used (no daemon needed)
- **macOS-style Super key** — Super+C/V/X/A/Z for copy, paste, cut, select all, undo (via wtype, togglable)
- **Screenshot shortcuts** — Super+Shift+3/4/5 for full/region/window capture
- **GUI config editor** — four tabs for shortcuts, displays, appearance, and behavior
- **Key recording** — click ⌨ then press a combo to capture it
- **Conflict detection** — warns on duplicate keybindings, can auto-fix

## Usage

```bash
# Snap the focused window
oblong snap left       # cycles: half → third → two-thirds
oblong snap right
oblong snap up
oblong snap down
oblong snap topleft / topright / bottomleft / bottomright
oblong snap center     # cycles: 60% → 50% → 33%
oblong snap maximize
oblong snap restore    # back to tiling

# Switch windows (MRU order)
oblong switch next     # Super+Tab
oblong switch prev     # Super+Shift+Tab

# Open the GUI
oblong gui
oblong                 # gui is the default
```

## Install

```bash
cargo build --release
cp target/release/oblong ~/.local/bin/
```

### Optional dependencies

- **wtype** — required for macOS-style Super+C/V shortcuts (`apt install wtype`)
- **grim** + **slurp** — for screenshot shortcuts
- **swaylock** + **swayidle** — for lock screen

## GUI

| | |
|---|---|
| ![Shortcuts](https://raw.githubusercontent.com/mvhenten/oblong/screenshots/shortcuts.png) | ![Displays](https://raw.githubusercontent.com/mvhenten/oblong/screenshots/displays.png) |
| ![Appearance](https://raw.githubusercontent.com/mvhenten/oblong/screenshots/appearance.png) | ![Behavior](https://raw.githubusercontent.com/mvhenten/oblong/screenshots/behavior.png) |

- **Shortcuts** — edit keybindings with live recording, duplicate detection, and conflict resolution
- **Displays** — resolution, scale, and relative positioning for connected monitors
- **Appearance** — font, gaps, borders, and per-class window colors with inline color picker
- **Behavior** — focus policy, floating defaults, macOS-style Super key toggle

## File Layout

```
~/.config/oblong/
    bindings.json          # source of truth for shortcuts
    outputs.json           # source of truth for display config
    appearance.json        # source of truth for appearance
    defaults.json          # source of truth for behavior

~/.config/sway/oblong/
    shortcuts.conf         # generated sway bindsyms
    outputs.conf           # generated sway output blocks
    appearance.conf        # generated sway appearance
    defaults.conf          # generated sway behavior defaults
    super-key-helper.sh    # generated script for Super+C/V

~/.config/sway/config
    include ~/.config/sway/oblong/*.conf   # one line added by oblong
```

The JSON files are the source of truth. The `.conf` files are generated — don't edit them by hand.

## Design Decisions

### Never touch the user's config

Oblong is **additive only**. It never modifies `~/.config/sway/config` beyond adding a single `include` line. All managed settings go into `~/.config/sway/oblong/`. Removing the include reverts everything.

### MRU window switching without a daemon

Sway's tree already tracks focus order in each container's `focus` array. The switcher reads this on each invocation and freezes the MRU list for the duration of a rapid Tab cycle (1.5s timeout). No background process needed.

### macOS-style Super key

When enabled, `Super+C` sends `Ctrl+Shift+C` in terminals (copy) and `Ctrl+C` in GUI apps. Detection is automatic via `swaymsg get_tree`. Requires `wtype`.

### Snap cycling

Repeated snaps cycle: **half → third → two-thirds**. State tracked in `$XDG_RUNTIME_DIR/oblong-state` with 1.5s timeout.

### Keybinding conflicts

Duplicate `bindsym` in sway fires both — it's not last-wins. Oblong detects this and the "Fix Conflicts" button comments out conflicting lines prefixed with `# [oblong]`.
