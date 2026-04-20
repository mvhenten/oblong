# Oblong

Window management for Sway. Snap windows to halves, thirds, and two-thirds with keyboard shortcuts — cycling through sizes on repeated presses. Includes a tabbed GUI for editing shortcuts and display configuration.

## Usage

```bash
# Snap the focused window
oblong snap left       # cycles: half → third → two-thirds
oblong snap right
oblong snap up
oblong snap down
oblong snap topleft
oblong snap topright
oblong snap bottomleft
oblong snap bottomright
oblong snap center     # cycles: 60% → 50% → 33%
oblong snap maximize
oblong snap restore    # back to tiling

# Open the GUI
oblong gui
oblong              # gui is the default
```

## Install

```bash
cargo build --release
cp target/release/oblong ~/.local/bin/
```

## GUI

Two tabs:

- **Shortcuts** — edit key bindings for all snap directions. Detects conflicting bindings in your main sway config and can comment them out with "Fix Conflicts".
- **Displays** — configure connected monitors: resolution, scale, and relative positioning (left of, right of, above, below). Queries live output info from sway.

## File Layout

```
~/.config/oblong/
    bindings.json          # source of truth for shortcuts
    outputs.json           # source of truth for display config

~/.config/sway/oblong/
    shortcuts.conf         # generated sway bindsyms
    outputs.conf           # generated sway output blocks

~/.config/sway/config
    include ~/.config/sway/oblong/*.conf   # one line added by oblong
```

The JSON files are the source of truth. The `.conf` files are generated output — don't edit them by hand.

## Design Decisions

### Never touch the user's config

Oblong is **additive only**. We never modify the user's main `~/.config/sway/config` beyond adding a single wildcard `include` line. All managed settings go into namespaced includes under `~/.config/sway/oblong/`.

Sway's last-wins semantics means our includes override anything in the main config for output settings. Removing the include line reverts everything.

### Keybinding conflicts

Duplicate `bindsym` for the same key combo is not last-wins in sway — both fire. Oblong detects this and warns on save. The "Fix Conflicts" button comments out the conflicting lines in the main config, prefixed with `# [oblong]` so they're easy to find and revert.

### Snap cycling

Repeated snaps in the same direction cycle through sizes: **half → third → two-thirds**. State is tracked in `$XDG_RUNTIME_DIR/oblong-state` with a 1.5s timeout — if you wait longer, it resets to half.

### Adding new config types

New `.conf` files dropped into `~/.config/sway/oblong/` are picked up automatically by the wildcard include. No changes to the user's main config needed.

## Planned

- [ ] Appearance tab (gaps, borders, colors, font)
- [ ] Input tab (keyboard layout, repeat rate, touchpad)
- [ ] Autostart tab (exec / exec_always)
- [ ] Gap-aware snap positioning
- [ ] Per-monitor snap settings
