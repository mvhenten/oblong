use iced::widget::{
    button, column, container, horizontal_rule, pick_list, row, scrollable, slider, text,
    text_input, Column,
};
use iced::{alignment, color, window, Border, Element, Font, Length, Task, Theme};

use crate::appearance::*;
use crate::config::*;
use crate::defaults::*;
use crate::outputs::*;

const APP_FONT_BYTES: &[u8] = include_bytes!("../fonts/DejaVuSans.ttf");
const APP_FONT: Font = Font::with_name("DejaVu Sans");

pub fn run() -> iced::Result {
    iced::application("Oblong", App::update, App::view)
        .theme(|_| Theme::Dark)
        .font(APP_FONT_BYTES)
        .default_font(APP_FONT)
        .window_size((700.0, 800.0))
        .run()
}

// ── Button styles ───────────────────────────────────────────

fn dark_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(iced::Background::Color(color!(0x3a3a3a))),
        text_color: color!(0xcccccc),
        border: Border {
            color: color!(0x555555),
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: Default::default(),
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(color!(0x4a4a4a))),
            text_color: color!(0xeeeeee),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(color!(0x2a2a2a))),
            ..base
        },
        _ => base,
    }
}

fn tab_active(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(iced::Background::Color(color!(0x444444))),
        text_color: color!(0xeeeeee),
        border: Border {
            color: color!(0x88bb88),
            width: 0.0,
            radius: 4.0.into(),
        },
        shadow: Default::default(),
    }
}

fn tab_inactive(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(iced::Background::Color(color!(0x2a2a2a))),
        text_color: color!(0x888888),
        border: Border {
            color: color!(0x444444),
            width: 0.0,
            radius: 4.0.into(),
        },
        shadow: Default::default(),
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(color!(0x353535))),
            text_color: color!(0xbbbbbb),
            ..base
        },
        _ => base,
    }
}

fn spinner_button(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(iced::Background::Color(color!(0x3a3a3a))),
        text_color: color!(0xaaaaaa),
        border: Border {
            color: color!(0x555555),
            width: 1.0,
            radius: 3.0.into(),
        },
        shadow: Default::default(),
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(color!(0x4a4a4a))),
            text_color: color!(0xeeeeee),
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(color!(0x2a2a2a))),
            ..base
        },
        _ => base,
    }
}

// ── Composite widgets ───────────────────────────────────────

/// Number input with − / + spinner buttons.
fn spinner<'a>(
    value: &str,
    placeholder: &str,
    on_input: impl Fn(String) -> Message + 'a,
    on_dec: Message,
    on_inc: Message,
) -> Element<'a, Message> {
    let dec = button(text("−").size(14))
        .on_press(on_dec)
        .style(spinner_button)
        .padding([2, 8]);
    let input = text_input(placeholder, value)
        .on_input(on_input)
        .size(13)
        .width(Length::Fixed(56.0));
    let inc = button(text("+").size(14))
        .on_press(on_inc)
        .style(spinner_button)
        .padding([2, 8]);
    row![dec, input, inc]
        .spacing(2)
        .align_y(alignment::Vertical::Center)
        .into()
}

/// Parse a "#rrggbb" hex string into (r, g, b).
fn parse_hex_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.strip_prefix('#').unwrap_or(hex);
    if hex.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Parse a "#rrggbb" hex string into an iced Color.
fn parse_hex_color(hex: &str) -> Option<iced::Color> {
    let (r, g, b) = parse_hex_rgb(hex)?;
    Some(iced::Color::from_rgb8(r, g, b))
}

/// Color hex input with a clickable preview swatch.
fn color_input<'a>(
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
    on_click: Message,
    is_active: bool,
) -> Element<'a, Message> {
    let swatch_color = parse_hex_color(value).unwrap_or(iced::Color::from_rgb8(0x33, 0x33, 0x33));
    let highlight = if is_active { color!(0xeeeeee) } else { color!(0x666666) };
    let swatch = button(
        container(text(""))
            .width(Length::Fixed(16.0))
            .height(Length::Fixed(16.0))
            .style(move |_theme: &Theme| container::Style {
                background: Some(iced::Background::Color(swatch_color)),
                border: Border {
                    color: highlight,
                    width: if is_active { 2.0 } else { 1.0 },
                    radius: 2.0.into(),
                },
                ..Default::default()
            }),
    )
    .on_press(on_click)
    .padding(0)
    .style(|_theme: &Theme, _status| button::Style {
        background: None,
        text_color: color!(0xffffff),
        border: Border { width: 0.0, ..Default::default() },
        shadow: Default::default(),
    });
    let input = text_input("#rrggbb", value)
        .on_input(on_input)
        .size(11)
        .width(Length::Fill);
    row![swatch, input]
        .spacing(4)
        .align_y(alignment::Vertical::Center)
        .into()
}

/// RGB slider editor panel for the active color.
fn color_editor<'a>(hex: &'a str) -> Element<'a, Message> {
    let (r, g, b) = parse_hex_rgb(hex).unwrap_or((0, 0, 0));
    let preview_color = iced::Color::from_rgb8(r, g, b);

    let preview = container(text(""))
        .width(Length::Fixed(48.0))
        .height(Length::Fixed(48.0))
        .style(move |_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(preview_color)),
            border: Border {
                color: color!(0x666666),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        });

    let r_label = text("R").size(12).color(color!(0xcc6666)).width(Length::Fixed(16.0));
    let r_val = text(format!("{r}")).size(12).width(Length::Fixed(30.0));
    let r_slider = slider(0..=255u8, r, move |v| Message::AppColorSlider(v, g, b))
        .width(Length::Fill);
    let r_row = row![r_label, r_slider, r_val]
        .spacing(8)
        .align_y(alignment::Vertical::Center);

    let g_label = text("G").size(12).color(color!(0x66cc66)).width(Length::Fixed(16.0));
    let g_val = text(format!("{g}")).size(12).width(Length::Fixed(30.0));
    let g_slider = slider(0..=255u8, g, move |v| Message::AppColorSlider(r, v, b))
        .width(Length::Fill);
    let g_row = row![g_label, g_slider, g_val]
        .spacing(8)
        .align_y(alignment::Vertical::Center);

    let b_label = text("B").size(12).color(color!(0x6666cc)).width(Length::Fixed(16.0));
    let b_val = text(format!("{b}")).size(12).width(Length::Fixed(30.0));
    let b_slider = slider(0..=255u8, b, move |v| Message::AppColorSlider(r, g, v))
        .width(Length::Fill);
    let b_row = row![b_label, b_slider, b_val]
        .spacing(8)
        .align_y(alignment::Vertical::Center);

    let hex_label = text(hex).size(13).color(color!(0xaaaaaa));

    let sliders = column![r_row, g_row, b_row].spacing(4).width(Length::Fill);

    let editor = row![preview, sliders, hex_label]
        .spacing(12)
        .align_y(alignment::Vertical::Center);

    container(editor)
        .padding(12)
        .width(Length::Fill)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(color!(0x2a2a2a))),
            border: Border {
                color: color!(0x444444),
                width: 1.0,
                radius: 4.0.into(),
            },
            ..Default::default()
        })
        .into()
}

// ── Default snap bindings ───────────────────────────────────

fn default_snap_bindings() -> Vec<Binding> {
    let bin = std::env::current_exe()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "oblong".into());

    vec![
        Binding { keys: "$mod+Left".into(),       command: format!("exec {bin} snap left") },
        Binding { keys: "$mod+Right".into(),      command: format!("exec {bin} snap right") },
        Binding { keys: "$mod+Up".into(),         command: format!("exec {bin} snap up") },
        Binding { keys: "$mod+Down".into(),       command: format!("exec {bin} snap down") },
        Binding { keys: "$mod+m".into(),          command: format!("exec {bin} snap maximize") },
        Binding { keys: "$mod+c".into(),          command: format!("exec {bin} snap center") },
        Binding { keys: "$mod+BackSpace".into(),  command: format!("exec {bin} snap restore") },
        Binding { keys: "$mod+u".into(),          command: format!("exec {bin} snap topleft") },
        Binding { keys: "$mod+i".into(),          command: format!("exec {bin} snap topright") },
        Binding { keys: "$mod+Shift+u".into(),    command: format!("exec {bin} snap bottomleft") },
        Binding { keys: "$mod+Shift+i".into(),    command: format!("exec {bin} snap bottomright") },
    ]
}

// ── Tabs ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tab {
    Shortcuts,
    Displays,
    Appearance,
    Behavior,
}

// ── App state ───────────────────────────────────────────────

struct App {
    tab: Tab,
    // Shortcuts
    groups: Vec<BindingGroup>,
    // Displays
    sway_outputs: Vec<SwayOutput>,
    output_configs: Vec<OutputConfig>,
    // Appearance
    appearance: AppearanceConfig,
    system_fonts: Vec<String>,
    editing_color: Option<(ColorTarget, ColorField)>,
    // Behavior
    defaults: DefaultsConfig,
    // Shared
    status: String,
}

impl Default for App {
    fn default() -> Self {
        let groups = load_config().unwrap_or_else(|| {
            group_bindings(default_snap_bindings())
        });

        let (sway_outputs, output_configs) = match query_outputs() {
            Ok(outputs) => {
                let configs = load_output_configs()
                    .and_then(|mut cfgs| {
                        if migrate_output_configs(&mut cfgs, &outputs) {
                            Some(cfgs)
                        } else {
                            None // migration failed, regenerate
                        }
                    })
                    .unwrap_or_else(|| {
                        let mut cfgs: Vec<OutputConfig> =
                            outputs.iter().map(config_from_sway).collect();
                        infer_relative_positions(&mut cfgs, &outputs);
                        cfgs
                    });
                (outputs, configs)
            }
            Err(_) => (vec![], vec![]),
        };

        let appearance = load_appearance().unwrap_or_default();
        let system_fonts = list_system_fonts();
        let defaults = load_defaults().unwrap_or_default();

        // Auto-generate defaults.conf if it doesn't exist
        let defaults_path = std::path::PathBuf::from(
            std::env::var("HOME").unwrap_or_else(|_| ".".into()),
        )
        .join(".config/sway/oblong/defaults.conf");
        if !defaults_path.exists() {
            let _ = write_defaults_conf(&defaults);
            apply_defaults_live(&defaults);
        }

        Self {
            tab: Tab::Shortcuts,
            groups,
            sway_outputs,
            output_configs,
            appearance,
            system_fonts,
            editing_color: None,
            defaults,
            status: String::new(),
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    SwitchTab(Tab),
    // Shortcuts
    KeysChanged(usize, usize, String),
    SaveShortcuts,
    SaveAndReloadShortcuts,
    FixConflicts,
    // Displays
    OutputResolution(usize, String),
    OutputScale(usize, String),
    OutputScaleStep(usize, i32),
    OutputPosition(usize, String),
    OutputPositionTarget(usize, String),
    SaveDisplays,
    SaveAndReloadDisplays,
    RefreshOutputs,
    // Appearance
    AppFontFamily(String),
    AppFontSize(String),
    AppFontSizeStep(i32),
    AppGapsInner(String),
    AppGapsInnerStep(i32),
    AppGapsOuter(String),
    AppGapsOuterStep(i32),
    AppBorderWidth(String),
    AppBorderWidthStep(i32),
    AppBorderStyle(String),
    AppColor(ColorTarget, ColorField, String),
    AppColorSelect(ColorTarget, ColorField),
    AppColorSlider(u8, u8, u8),
    SaveAppearance,
    SaveAndReloadAppearance,
    // Behavior
    DefaultsFocusActivation(String),
    DefaultsFocusFollowsMouse(String),
    DefaultsPopupFullscreen(String),
    DefaultsAutoBackAndForth(bool),
    DefaultsMouseWarping(String),
    DefaultsFloatingWidth(String),
    DefaultsFloatingWidthStep(i32),
    DefaultsFloatingHeight(String),
    DefaultsFloatingHeightStep(i32),
    SaveDefaults,
    SaveAndReloadDefaults,
    // Shared
    Quit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorTarget {
    Focused,
    FocusedInactive,
    Unfocused,
    Urgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorField {
    Border,
    Background,
    Text,
    Indicator,
    ChildBorder,
}

impl App {
    fn all_bindings(&self) -> Vec<&Binding> {
        self.groups.iter().flat_map(|g| &g.bindings).collect()
    }

    fn other_output_names(&self, exclude: &str) -> Vec<String> {
        self.output_configs
            .iter()
            .filter(|c| c.name != exclude)
            .map(|c| c.name.clone())
            .collect()
    }

    fn get_color_hex(&self, target: ColorTarget, field: ColorField) -> &str {
        let cs = match target {
            ColorTarget::Focused => &self.appearance.colors.focused,
            ColorTarget::FocusedInactive => &self.appearance.colors.focused_inactive,
            ColorTarget::Unfocused => &self.appearance.colors.unfocused,
            ColorTarget::Urgent => &self.appearance.colors.urgent,
        };
        match field {
            ColorField::Border => &cs.border,
            ColorField::Background => &cs.background,
            ColorField::Text => &cs.text,
            ColorField::Indicator => &cs.indicator,
            ColorField::ChildBorder => &cs.child_border,
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::SwitchTab(tab) => {
                self.tab = tab;
                self.status.clear();
            }

            // ── Shortcuts ───────────────────────────────
            Message::KeysChanged(gi, bi, new_keys) => {
                if let Some(g) = self.groups.get_mut(gi) {
                    if let Some(b) = g.bindings.get_mut(bi) {
                        b.keys = new_keys;
                    }
                }
                self.status.clear();
            }
            Message::SaveShortcuts => {
                match self.save_shortcuts() {
                    Ok(()) => self.status = "✓ Saved.".into(),
                    Err(e) if e.starts_with("Saved") => self.status = format!("⚠ {e}"),
                    Err(e) => self.status = format!("✗ Error: {e}"),
                }
            }
            Message::SaveAndReloadShortcuts => {
                match self.save_shortcuts() {
                    Ok(()) => self.sway_reload("Saved"),
                    Err(e) if e.starts_with("Saved") => {
                        self.sway_reload(&e);
                    }
                    Err(e) => self.status = format!("✗ Error: {e}"),
                }
            }
            Message::FixConflicts => {
                let all = self.all_bindings();
                let conflicts = detect_conflicts(&all);
                if conflicts.is_empty() {
                    self.status = "No conflicts found.".into();
                } else {
                    match comment_out_conflicts(&conflicts) {
                        Ok(n) => self.status = format!("✓ Commented out {n} conflicting binding(s)."),
                        Err(e) => self.status = format!("✗ Error: {e}"),
                    }
                }
            }

            // ── Displays ────────────────────────────────
            Message::OutputResolution(idx, res) => {
                if let Some(conf) = self.output_configs.get_mut(idx) {
                    // Pick highest refresh rate for this resolution
                    if let Some(sway_out) = self.sway_outputs.iter().find(|o| o.stable_id() == conf.name || o.name == conf.name) {
                        let rates = refresh_rates_for_resolution(sway_out, &res);
                        conf.refresh = rates.first().copied();
                    }
                    conf.resolution = Some(res);
                }
                self.status.clear();
            }
            Message::OutputScale(idx, scale_str) => {
                if let Some(conf) = self.output_configs.get_mut(idx) {
                    if let Ok(s) = scale_str.parse::<f64>() {
                        conf.scale = Some(s);
                    }
                }
                self.status.clear();
            }
            Message::OutputScaleStep(idx, delta) => {
                if let Some(conf) = self.output_configs.get_mut(idx) {
                    let cur = conf.scale.unwrap_or(1.0);
                    let step = if delta > 0 { 0.25 } else { -0.25 };
                    conf.scale = Some((cur + step).max(0.25));
                }
                self.status.clear();
            }
            Message::OutputPosition(idx, relation) => {
                if let Some(conf) = self.output_configs.get(idx) {
                    let conf_name = conf.name.clone();
                    let others = self.other_output_names(&conf_name);
                    let target = conf
                        .position
                        .as_ref()
                        .and_then(|p| p.target_name().map(String::from))
                        .unwrap_or_else(|| others.first().cloned().unwrap_or_default());

                    let new_pos = match relation.as_str() {
                        "Left of" => OutputPosition::LeftOf(target),
                        "Right of" => OutputPosition::RightOf(target),
                        "Above" => OutputPosition::Above(target),
                        "Below" => OutputPosition::Below(target),
                        _ => return Task::none(),
                    };
                    if let Some(conf) = self.output_configs.get_mut(idx) {
                        conf.position = Some(new_pos);
                    }
                }
                self.status.clear();
            }
            Message::OutputPositionTarget(idx, target) => {
                if let Some(conf) = self.output_configs.get_mut(idx) {
                    let rel_idx = conf
                        .position
                        .as_ref()
                        .map(|p| p.relation_index())
                        .unwrap_or(1);
                    conf.position = Some(match rel_idx {
                        0 => OutputPosition::LeftOf(target),
                        1 => OutputPosition::RightOf(target),
                        2 => OutputPosition::Above(target),
                        3 => OutputPosition::Below(target),
                        _ => return Task::none(),
                    });
                }
                self.status.clear();
            }
            Message::SaveDisplays => {
                match self.save_displays() {
                    Ok(()) => self.status = "✓ Display config saved.".into(),
                    Err(e) => self.status = format!("✗ Error: {e}"),
                }
            }
            Message::SaveAndReloadDisplays => {
                match self.save_displays() {
                    Ok(()) => self.sway_reload("Display config saved"),
                    Err(e) => self.status = format!("✗ Error: {e}"),
                }
            }
            Message::RefreshOutputs => {
                match query_outputs() {
                    Ok(outputs) => {
                        let mut cfgs: Vec<OutputConfig> =
                            outputs.iter().map(config_from_sway).collect();
                        infer_relative_positions(&mut cfgs, &outputs);
                        self.sway_outputs = outputs;
                        self.output_configs = cfgs;
                        self.status = "Refreshed from sway.".into();
                    }
                    Err(e) => self.status = format!("Error: {e}"),
                }
            }

            // ── Appearance ──────────────────────────────
            Message::AppFontFamily(family) => {
                self.appearance.font_family = family;
                self.status.clear();
            }
            Message::AppFontSize(val) => {
                if let Ok(s) = val.parse::<u32>() {
                    self.appearance.font_size = s;
                }
                self.status.clear();
            }
            Message::AppFontSizeStep(delta) => {
                self.appearance.font_size = (self.appearance.font_size as i32 + delta).max(1) as u32;
                self.status.clear();
            }
            Message::AppGapsInner(val) => {
                if let Ok(g) = val.parse::<u32>() {
                    self.appearance.gaps_inner = g;
                }
                self.status.clear();
            }
            Message::AppGapsInnerStep(delta) => {
                self.appearance.gaps_inner = (self.appearance.gaps_inner as i32 + delta).max(0) as u32;
                self.status.clear();
            }
            Message::AppGapsOuter(val) => {
                if let Ok(g) = val.parse::<u32>() {
                    self.appearance.gaps_outer = g;
                }
                self.status.clear();
            }
            Message::AppGapsOuterStep(delta) => {
                self.appearance.gaps_outer = (self.appearance.gaps_outer as i32 + delta).max(0) as u32;
                self.status.clear();
            }
            Message::AppBorderWidth(val) => {
                if let Ok(w) = val.parse::<u32>() {
                    self.appearance.border_width = w;
                }
                self.status.clear();
            }
            Message::AppBorderWidthStep(delta) => {
                self.appearance.border_width = (self.appearance.border_width as i32 + delta).max(0) as u32;
                self.status.clear();
            }
            Message::AppBorderStyle(val) => {
                if let Some(style) = BorderStyle::from_str(&val) {
                    self.appearance.border_style = style;
                }
                self.status.clear();
            }
            Message::AppColor(target, field, val) => {
                let cs = match target {
                    ColorTarget::Focused => &mut self.appearance.colors.focused,
                    ColorTarget::FocusedInactive => &mut self.appearance.colors.focused_inactive,
                    ColorTarget::Unfocused => &mut self.appearance.colors.unfocused,
                    ColorTarget::Urgent => &mut self.appearance.colors.urgent,
                };
                match field {
                    ColorField::Border => cs.border = val,
                    ColorField::Background => cs.background = val,
                    ColorField::Text => cs.text = val,
                    ColorField::Indicator => cs.indicator = val,
                    ColorField::ChildBorder => cs.child_border = val,
                }
                self.status.clear();
            }
            Message::AppColorSelect(target, field) => {
                let key = (target, field);
                if self.editing_color == Some(key) {
                    self.editing_color = None;
                } else {
                    self.editing_color = Some(key);
                }
            }
            Message::AppColorSlider(r, g, b) => {
                let hex = format!("#{:02x}{:02x}{:02x}", r, g, b);
                if let Some((target, field)) = self.editing_color {
                    let cs = match target {
                        ColorTarget::Focused => &mut self.appearance.colors.focused,
                        ColorTarget::FocusedInactive => &mut self.appearance.colors.focused_inactive,
                        ColorTarget::Unfocused => &mut self.appearance.colors.unfocused,
                        ColorTarget::Urgent => &mut self.appearance.colors.urgent,
                    };
                    match field {
                        ColorField::Border => cs.border = hex,
                        ColorField::Background => cs.background = hex,
                        ColorField::Text => cs.text = hex,
                        ColorField::Indicator => cs.indicator = hex,
                        ColorField::ChildBorder => cs.child_border = hex,
                    }
                }
                self.status.clear();
            }
            Message::SaveAppearance => {
                match write_appearance_conf(&self.appearance) {
                    Ok(()) => {
                        save_appearance(&self.appearance);
                        self.status = "Saved.".into();
                    }
                    Err(e) => self.status = format!("Error: {e}"),
                }
            }
            Message::SaveAndReloadAppearance => {
                match write_appearance_conf(&self.appearance) {
                    Ok(()) => {
                        save_appearance(&self.appearance);
                        apply_appearance_live(&self.appearance);
                        self.sway_reload("Appearance saved");
                    }
                    Err(e) => self.status = format!("Error: {e}"),
                }
            }

            // ── Behavior ────────────────────────────────
            Message::DefaultsFocusActivation(val) => {
                self.defaults.focus_on_window_activation = val;
                self.status.clear();
            }
            Message::DefaultsFocusFollowsMouse(val) => {
                self.defaults.focus_follows_mouse = val;
                self.status.clear();
            }
            Message::DefaultsPopupFullscreen(val) => {
                self.defaults.popup_during_fullscreen = val;
                self.status.clear();
            }
            Message::DefaultsAutoBackAndForth(val) => {
                self.defaults.workspace_auto_back_and_forth = val;
                self.status.clear();
            }
            Message::DefaultsMouseWarping(val) => {
                self.defaults.mouse_warping = val;
                self.status.clear();
            }
            Message::DefaultsFloatingWidth(val) => {
                if let Ok(w) = val.parse::<i32>() {
                    let (_, h) = self.defaults.default_floating_size.unwrap_or((1200, 800));
                    self.defaults.default_floating_size = Some((w, h));
                }
                self.status.clear();
            }
            Message::DefaultsFloatingWidthStep(delta) => {
                let (w, h) = self.defaults.default_floating_size.unwrap_or((1200, 800));
                self.defaults.default_floating_size = Some(((w + delta * 50).max(200), h));
                self.status.clear();
            }
            Message::DefaultsFloatingHeight(val) => {
                if let Ok(h) = val.parse::<i32>() {
                    let (w, _) = self.defaults.default_floating_size.unwrap_or((1200, 800));
                    self.defaults.default_floating_size = Some((w, h));
                }
                self.status.clear();
            }
            Message::DefaultsFloatingHeightStep(delta) => {
                let (w, h) = self.defaults.default_floating_size.unwrap_or((1200, 800));
                self.defaults.default_floating_size = Some((w, (h + delta * 50).max(200)));
                self.status.clear();
            }
            Message::SaveDefaults => {
                match write_defaults_conf(&self.defaults) {
                    Ok(()) => {
                        save_defaults(&self.defaults);
                        self.status = "✓ Saved.".into();
                    }
                    Err(e) => self.status = format!("✗ Error: {e}"),
                }
            }
            Message::SaveAndReloadDefaults => {
                match write_defaults_conf(&self.defaults) {
                    Ok(()) => {
                        save_defaults(&self.defaults);
                        apply_defaults_live(&self.defaults);
                        self.sway_reload("Behavior saved");
                    }
                    Err(e) => self.status = format!("✗ Error: {e}"),
                }
            }

            Message::Quit => {
                return window::get_oldest().then(|id| window::close(id.expect("no window")));
            }
        }
        Task::none()
    }

    fn sway_reload(&mut self, prefix: &str) {
        match std::process::Command::new("swaymsg").arg("reload").output() {
            Ok(_) => self.status = format!("✓ {prefix} & sway reloaded."),
            Err(e) => self.status = format!("✓ {prefix}, but reload failed: {e}"),
        }
    }

    fn save_shortcuts(&self) -> Result<(), String> {
        let all: Vec<&Binding> = self.all_bindings();
        write_sway_config(&all)?;
        save_config(&self.groups);

        let conflicts = detect_conflicts(&all);
        if conflicts.is_empty() {
            Ok(())
        } else {
            Err(format!(
                "Saved, but {} key(s) also bound in main config: {}. Use 'Fix Conflicts' to resolve.",
                conflicts.len(),
                conflicts.join(", ")
            ))
        }
    }

    fn save_displays(&self) -> Result<(), String> {
        write_outputs_conf(&self.output_configs)?;
        save_output_configs(&self.output_configs);
        Ok(())
    }

    // ── View ────────────────────────────────────────────────

    fn view(&self) -> Element<'_, Message> {
        let title = text("Oblong").size(24);
        let subtitle = text("Window management for Sway")
            .size(13)
            .color(color!(0x888888));
        let header = column![title, subtitle].spacing(4);

        let tabs = self.view_tabs();

        let content: Element<'_, Message> = match self.tab {
            Tab::Shortcuts => self.view_shortcuts(),
            Tab::Displays => self.view_displays(),
            Tab::Appearance => self.view_appearance(),
            Tab::Behavior => self.view_behavior(),
        };

        let status = text(&self.status).size(13).color(color!(0x5a8a5a));

        let main_content = column![
            header,
            tabs,
            content,
            status,
        ]
        .spacing(8)
        .padding(24)
        .max_width(700);

        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .into()
    }

    fn view_tabs(&self) -> Element<'_, Message> {
        let shortcut_tab = button(text("Shortcuts").size(14))
            .on_press(Message::SwitchTab(Tab::Shortcuts))
            .style(if self.tab == Tab::Shortcuts { tab_active } else { tab_inactive })
            .padding([8, 20]);

        let display_tab = button(text("Displays").size(14))
            .on_press(Message::SwitchTab(Tab::Displays))
            .style(if self.tab == Tab::Displays { tab_active } else { tab_inactive })
            .padding([8, 20]);

        let appearance_tab = button(text("Appearance").size(14))
            .on_press(Message::SwitchTab(Tab::Appearance))
            .style(if self.tab == Tab::Appearance { tab_active } else { tab_inactive })
            .padding([8, 20]);

        let behavior_tab = button(text("Behavior").size(14))
            .on_press(Message::SwitchTab(Tab::Behavior))
            .style(if self.tab == Tab::Behavior { tab_active } else { tab_inactive })
            .padding([8, 20]);

        row![shortcut_tab, display_tab, appearance_tab, behavior_tab]
            .spacing(2)
            .into()
    }

    fn view_shortcuts(&self) -> Element<'_, Message> {
        let mut content_col = Column::new().spacing(12);

        for (gi, group) in self.groups.iter().enumerate() {
            let group_title = text(&group.name).size(16).color(color!(0x88bb88));

            let bindings = group
                .bindings
                .iter()
                .enumerate()
                .fold(Column::new().spacing(4), |col, (bi, binding)| {
                    let label = text(label_for_command(&binding.command))
                        .size(13)
                        .width(Length::Fixed(200.0));

                    let input = text_input("e.g. $mod+Shift+Left", &binding.keys)
                        .on_input(move |val| Message::KeysChanged(gi, bi, val))
                        .size(13)
                        .width(Length::Fill);

                    col.push(
                        row![label, input]
                            .spacing(12)
                            .align_y(alignment::Vertical::Center),
                    )
                });

            content_col = content_col
                .push(group_title)
                .push(bindings)
                .push(horizontal_rule(1));
        }

        let buttons = row![
            button(text("Save").size(14))
                .on_press(Message::SaveShortcuts)
                .style(dark_button)
                .padding([8, 16]),
            button(text("Save & Reload").size(14))
                .on_press(Message::SaveAndReloadShortcuts)
                .style(dark_button)
                .padding([8, 16]),
            button(text("Fix Conflicts").size(14))
                .on_press(Message::FixConflicts)
                .style(dark_button)
                .padding([8, 16]),
            button(text("Close").size(14))
                .on_press(Message::Quit)
                .style(dark_button)
                .padding([8, 16]),
        ]
        .spacing(8);

        column![
            scrollable(
                container(content_col)
                    .padding(iced::Padding { right: 32.0, ..iced::Padding::ZERO })
            )
            .height(Length::Fill),
            buttons,
        ]
        .spacing(12)
        .height(Length::Fill)
        .into()
    }

    fn view_displays(&self) -> Element<'_, Message> {
        if self.sway_outputs.is_empty() {
            return column![
                text("No displays detected. Is sway running?").size(14).color(color!(0xcc8888)),
                button(text("Retry").size(14))
                    .on_press(Message::RefreshOutputs)
                    .style(dark_button)
                    .padding([8, 16]),
            ]
            .spacing(12)
            .height(Length::Fill)
            .into();
        }

        let mut content_col = Column::new().spacing(16);

        for (i, conf) in self.output_configs.iter().enumerate() {
            let sway_out = self.sway_outputs.iter().find(|o| o.stable_id() == conf.name || o.name == conf.name);
            let description = sway_out
                .map(|o| o.description())
                .unwrap_or_else(|| conf.name.clone());

            let output_title = text(description).size(16).color(color!(0x88bb88));

            // Resolution picker
            let modes: Vec<String> = sway_out
                .map(|o| unique_modes(o))
                .unwrap_or_default();
            let current_res = conf.resolution.clone();
            let res_label = text("Resolution").size(13).width(Length::Fixed(100.0));
            let res_picker = pick_list(
                modes,
                current_res,
                move |val| Message::OutputResolution(i, val),
            )
            .text_size(13)
            .width(Length::Fill);
            let res_row = row![res_label, res_picker]
                .spacing(12)
                .align_y(alignment::Vertical::Center);

            // Scale
            let scale_label = text("Scale").size(13).width(Length::Fixed(100.0));
            let scale_val = conf.scale.map(|s| format!("{s}")).unwrap_or_default();
            let idx = i;
            let scale_widget = spinner(
                &scale_val,
                "1.0",
                move |val| Message::OutputScale(idx, val),
                Message::OutputScaleStep(i, -1),
                Message::OutputScaleStep(i, 1),
            );
            let scale_row = row![scale_label, scale_widget]
                .spacing(12)
                .align_y(alignment::Vertical::Center);

            // Position (only for non-first outputs)
            let mut output_section = Column::new().spacing(6);
            output_section = output_section
                .push(output_title)
                .push(res_row)
                .push(scale_row);

            if self.output_configs.len() > 1 {
                let others = self.other_output_names(&conf.name);
                if !others.is_empty() {
                    let relations: Vec<String> = POSITION_RELATIONS.iter().map(|s| s.to_string()).collect();
                    let current_relation = conf.position.as_ref().map(|p| {
                        match p {
                            OutputPosition::LeftOf(_) => "Left of",
                            OutputPosition::RightOf(_) => "Right of",
                            OutputPosition::Above(_) => "Above",
                            OutputPosition::Below(_) => "Below",
                            OutputPosition::Absolute { .. } => "Right of",
                        }.to_string()
                    });
                    let current_target = conf
                        .position
                        .as_ref()
                        .and_then(|p| p.target_name().map(String::from))
                        .or_else(|| others.first().cloned());

                    let pos_label = text("Position").size(13).width(Length::Fixed(100.0));
                    let idx = i;
                    let rel_picker = pick_list(
                        relations,
                        current_relation,
                        move |val| Message::OutputPosition(idx, val),
                    )
                    .text_size(13)
                    .width(Length::Fixed(100.0));

                    let target_picker = pick_list(
                        others,
                        current_target,
                        move |val| Message::OutputPositionTarget(i, val),
                    )
                    .text_size(13)
                    .width(Length::Fill);

                    let pos_row = row![pos_label, rel_picker, target_picker]
                        .spacing(8)
                        .align_y(alignment::Vertical::Center);

                    output_section = output_section.push(pos_row);
                }
            }

            content_col = content_col
                .push(output_section)
                .push(horizontal_rule(1));
        }

        let buttons = row![
            button(text("Save").size(14))
                .on_press(Message::SaveDisplays)
                .style(dark_button)
                .padding([8, 16]),
            button(text("Save & Reload").size(14))
                .on_press(Message::SaveAndReloadDisplays)
                .style(dark_button)
                .padding([8, 16]),
            button(text("Refresh").size(14))
                .on_press(Message::RefreshOutputs)
                .style(dark_button)
                .padding([8, 16]),
            button(text("Close").size(14))
                .on_press(Message::Quit)
                .style(dark_button)
                .padding([8, 16]),
        ]
        .spacing(8);

        column![
            scrollable(
                container(content_col)
                    .padding(iced::Padding { right: 32.0, ..iced::Padding::ZERO })
            )
            .height(Length::Fill),
            buttons,
        ]
        .spacing(12)
        .height(Length::Fill)
        .into()
    }

    fn view_appearance(&self) -> Element<'_, Message> {
        let conf = &self.appearance;
        let mut content_col = Column::new().spacing(16);

        // ── Font ──
        let font_title = text("Font").size(16).color(color!(0x88bb88));

        let font_label = text("Family").size(13).width(Length::Fixed(100.0));
        let font_picker = pick_list(
            self.system_fonts.clone(),
            Some(conf.font_family.clone()),
            Message::AppFontFamily,
        )
        .text_size(13)
        .width(Length::Fill);
        let font_row = row![font_label, font_picker]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        let size_label = text("Size").size(13).width(Length::Fixed(100.0));
        let size_widget = spinner(
            &conf.font_size.to_string(),
            "11",
            Message::AppFontSize,
            Message::AppFontSizeStep(-1),
            Message::AppFontSizeStep(1),
        );
        let size_row = row![size_label, size_widget]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        content_col = content_col
            .push(font_title)
            .push(font_row)
            .push(size_row)
            .push(horizontal_rule(1));

        // ── Gaps ──
        let gaps_title = text("Gaps").size(16).color(color!(0x88bb88));

        let inner_label = text("Inner").size(13).width(Length::Fixed(100.0));
        let inner_widget = spinner(
            &conf.gaps_inner.to_string(),
            "10",
            Message::AppGapsInner,
            Message::AppGapsInnerStep(-1),
            Message::AppGapsInnerStep(1),
        );
        let inner_row = row![inner_label, inner_widget]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        let outer_label = text("Outer").size(13).width(Length::Fixed(100.0));
        let outer_widget = spinner(
            &conf.gaps_outer.to_string(),
            "5",
            Message::AppGapsOuter,
            Message::AppGapsOuterStep(-1),
            Message::AppGapsOuterStep(1),
        );
        let outer_row = row![outer_label, outer_widget]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        content_col = content_col
            .push(gaps_title)
            .push(inner_row)
            .push(outer_row)
            .push(horizontal_rule(1));

        // ── Borders ──
        let border_title = text("Borders").size(16).color(color!(0x88bb88));

        let style_label = text("Style").size(13).width(Length::Fixed(100.0));
        let styles: Vec<String> = BorderStyle::ALL.iter().map(|s| s.to_string()).collect();
        let style_picker = pick_list(
            styles,
            Some(conf.border_style.to_string()),
            Message::AppBorderStyle,
        )
        .text_size(13)
        .width(Length::Fixed(120.0));
        let style_row = row![style_label, style_picker]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        let width_label = text("Width").size(13).width(Length::Fixed(100.0));
        let width_widget = spinner(
            &conf.border_width.to_string(),
            "2",
            Message::AppBorderWidth,
            Message::AppBorderWidthStep(-1),
            Message::AppBorderWidthStep(1),
        );
        let width_row = row![width_label, width_widget]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        content_col = content_col
            .push(border_title)
            .push(style_row)
            .push(width_row)
            .push(horizontal_rule(1));

        // ── Colors ──
        let colors_title = text("Window Colors").size(16).color(color!(0x88bb88));
        content_col = content_col.push(colors_title);

        let color_targets = [
            ("Focused",          ColorTarget::Focused,         &conf.colors.focused),
            ("Focused Inactive", ColorTarget::FocusedInactive, &conf.colors.focused_inactive),
            ("Unfocused",        ColorTarget::Unfocused,       &conf.colors.unfocused),
            ("Urgent",           ColorTarget::Urgent,          &conf.colors.urgent),
        ];

        // Column headers
        let header = row![
            text("").size(11).width(Length::Fixed(120.0)),
            text("border").size(11).color(color!(0x888888)).width(Length::Fill),
            text("bg").size(11).color(color!(0x888888)).width(Length::Fill),
            text("text").size(11).color(color!(0x888888)).width(Length::Fill),
            text("indicator").size(11).color(color!(0x888888)).width(Length::Fill),
            text("child").size(11).color(color!(0x888888)).width(Length::Fill),
        ]
        .spacing(4);
        content_col = content_col.push(header);

        for (label, target, cs) in &color_targets {
            let t = *target;
            let ec = self.editing_color;
            let fields: [(ColorField, &str); 5] = [
                (ColorField::Border, &cs.border),
                (ColorField::Background, &cs.background),
                (ColorField::Text, &cs.text),
                (ColorField::Indicator, &cs.indicator),
                (ColorField::ChildBorder, &cs.child_border),
            ];

            // Preview bar showing what this color class looks like
            let border_c = parse_hex_color(&cs.child_border).unwrap_or(color!(0x333333));
            let bg_c = parse_hex_color(&cs.background).unwrap_or(color!(0x222222));
            let text_c = parse_hex_color(&cs.text).unwrap_or(color!(0xffffff));
            let indicator_c = parse_hex_color(&cs.indicator).unwrap_or(color!(0x333333));
            let preview_bar = container(
                row![
                    // Indicator pip
                    container(text(""))
                        .width(Length::Fixed(4.0))
                        .height(Length::Fixed(20.0))
                        .style(move |_: &Theme| container::Style {
                            background: Some(iced::Background::Color(indicator_c)),
                            ..Default::default()
                        }),
                    // Title text on background
                    container(
                        text(format!("  {} — Window Title", label))
                            .size(12)
                            .color(text_c)
                    )
                    .width(Length::Fill)
                    .padding([4, 8])
                    .style(move |_: &Theme| container::Style {
                        background: Some(iced::Background::Color(bg_c)),
                        ..Default::default()
                    }),
                ]
                .align_y(alignment::Vertical::Center)
            )
            .width(Length::Fill)
            .style(move |_: &Theme| container::Style {
                background: Some(iced::Background::Color(border_c)),
                border: Border {
                    color: border_c,
                    width: 2.0,
                    radius: 3.0.into(),
                },
                ..Default::default()
            });

            let mut color_row = row![text(*label).size(13).width(Length::Fixed(120.0))].spacing(4);
            for (field, val) in &fields {
                let f = *field;
                let active = ec == Some((t, f));
                color_row = color_row.push(color_input(
                    val,
                    move |v| Message::AppColor(t, f, v),
                    Message::AppColorSelect(t, f),
                    active,
                ));
            }
            let color_row = color_row.align_y(alignment::Vertical::Center);
            content_col = content_col.push(preview_bar).push(color_row);
        }

        // Show color editor if a swatch is selected
        if let Some((target, field)) = self.editing_color {
            let hex = self.get_color_hex(target, field);
            content_col = content_col.push(color_editor(hex));
        }

        let buttons = row![
            button(text("Save").size(14))
                .on_press(Message::SaveAppearance)
                .style(dark_button)
                .padding([8, 16]),
            button(text("Save & Reload").size(14))
                .on_press(Message::SaveAndReloadAppearance)
                .style(dark_button)
                .padding([8, 16]),
            button(text("Close").size(14))
                .on_press(Message::Quit)
                .style(dark_button)
                .padding([8, 16]),
        ]
        .spacing(8);

        column![
            scrollable(
                container(content_col)
                    .padding(iced::Padding { right: 32.0, ..iced::Padding::ZERO })
            )
            .height(Length::Fill),
            buttons,
        ]
        .spacing(12)
        .height(Length::Fill)
        .into()
    }

    fn view_behavior(&self) -> Element<'_, Message> {
        let conf = &self.defaults;
        let mut content_col = Column::new().spacing(16);

        // ── Window Focus ──
        let focus_title = text("Window Focus").size(16).color(color!(0x88bb88));

        let activation_label = text("New windows").size(13).width(Length::Fixed(160.0));
        let activation_options: Vec<String> = vec!["focus".into(), "smart".into(), "urgent".into(), "none".into()];
        let activation_hint = text(match conf.focus_on_window_activation.as_str() {
            "focus" => "always steal focus",
            "smart" => "focus if on active workspace",
            "urgent" => "mark urgent, don't focus",
            "none" => "do nothing",
            _ => "",
        }).size(11).color(color!(0x888888));
        let activation_picker = pick_list(
            activation_options,
            Some(conf.focus_on_window_activation.clone()),
            Message::DefaultsFocusActivation,
        )
        .text_size(13)
        .width(Length::Fixed(120.0));
        let activation_row = row![activation_label, activation_picker, activation_hint]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        let follows_label = text("Focus follows mouse").size(13).width(Length::Fixed(160.0));
        let follows_options: Vec<String> = vec!["yes".into(), "no".into(), "always".into()];
        let follows_hint = text(match conf.focus_follows_mouse.as_str() {
            "yes" => "focus window under cursor",
            "no" => "click to focus only",
            "always" => "aggressive — even across outputs",
            _ => "",
        }).size(11).color(color!(0x888888));
        let follows_picker = pick_list(
            follows_options,
            Some(conf.focus_follows_mouse.clone()),
            Message::DefaultsFocusFollowsMouse,
        )
        .text_size(13)
        .width(Length::Fixed(120.0));
        let follows_row = row![follows_label, follows_picker, follows_hint]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        let warping_label = text("Mouse warping").size(13).width(Length::Fixed(160.0));
        let warping_options: Vec<String> = vec!["output".into(), "container".into(), "none".into()];
        let warping_hint = text(match conf.mouse_warping.as_str() {
            "output" => "warp to focused output",
            "container" => "warp to focused container",
            "none" => "never move the cursor",
            _ => "",
        }).size(11).color(color!(0x888888));
        let warping_picker = pick_list(
            warping_options,
            Some(conf.mouse_warping.clone()),
            Message::DefaultsMouseWarping,
        )
        .text_size(13)
        .width(Length::Fixed(120.0));
        let warping_row = row![warping_label, warping_picker, warping_hint]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        content_col = content_col
            .push(focus_title)
            .push(activation_row)
            .push(follows_row)
            .push(warping_row)
            .push(horizontal_rule(1));

        // ── Workspaces ──
        let ws_title = text("Workspaces").size(16).color(color!(0x88bb88));

        let baf_label = text("Auto back-and-forth").size(13).width(Length::Fixed(160.0));
        let baf_options: Vec<String> = vec!["yes".into(), "no".into()];
        let baf_hint = text(if conf.workspace_auto_back_and_forth {
            "repeat shortcut to return"
        } else {
            "stay on workspace"
        }).size(11).color(color!(0x888888));
        let baf_picker = pick_list(
            baf_options,
            Some(if conf.workspace_auto_back_and_forth { "yes".to_string() } else { "no".to_string() }),
            |val: String| Message::DefaultsAutoBackAndForth(val == "yes"),
        )
        .text_size(13)
        .width(Length::Fixed(120.0));
        let baf_row = row![baf_label, baf_picker, baf_hint]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        content_col = content_col
            .push(ws_title)
            .push(baf_row)
            .push(horizontal_rule(1));

        // ── Popups ──
        let popup_title = text("Popups & Fullscreen").size(16).color(color!(0x88bb88));

        let popup_label = text("Popup during fullscreen").size(13).width(Length::Fixed(160.0));
        let popup_options: Vec<String> = vec!["smart".into(), "ignore".into(), "leave_fullscreen".into()];
        let popup_hint = text(match conf.popup_during_fullscreen.as_str() {
            "smart" => "show if from focused app",
            "ignore" => "never show",
            "leave_fullscreen" => "exit fullscreen to show",
            _ => "",
        }).size(11).color(color!(0x888888));
        let popup_picker = pick_list(
            popup_options,
            Some(conf.popup_during_fullscreen.clone()),
            Message::DefaultsPopupFullscreen,
        )
        .text_size(13)
        .width(Length::Fixed(120.0));
        let popup_row = row![popup_label, popup_picker, popup_hint]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        content_col = content_col
            .push(popup_title)
            .push(popup_row)
            .push(horizontal_rule(1));

        // ── Floating Windows ──
        let float_title = text("Floating Windows").size(16).color(color!(0x88bb88));
        let (fw, fh) = conf.default_floating_size.unwrap_or((1200, 800));

        let fw_label = text("Default width").size(13).width(Length::Fixed(160.0));
        let fw_widget = spinner(
            &fw.to_string(),
            "1200",
            Message::DefaultsFloatingWidth,
            Message::DefaultsFloatingWidthStep(-1),
            Message::DefaultsFloatingWidthStep(1),
        );
        let fw_row = row![fw_label, fw_widget, text("px").size(11).color(color!(0x888888))]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        let fh_label = text("Default height").size(13).width(Length::Fixed(160.0));
        let fh_widget = spinner(
            &fh.to_string(),
            "800",
            Message::DefaultsFloatingHeight,
            Message::DefaultsFloatingHeightStep(-1),
            Message::DefaultsFloatingHeightStep(1),
        );
        let fh_row = row![fh_label, fh_widget, text("px").size(11).color(color!(0x888888))]
            .spacing(12)
            .align_y(alignment::Vertical::Center);

        content_col = content_col
            .push(float_title)
            .push(fw_row)
            .push(fh_row);

        let buttons = row![
            button(text("Save").size(14))
                .on_press(Message::SaveDefaults)
                .style(dark_button)
                .padding([8, 16]),
            button(text("Save & Reload").size(14))
                .on_press(Message::SaveAndReloadDefaults)
                .style(dark_button)
                .padding([8, 16]),
            button(text("Close").size(14))
                .on_press(Message::Quit)
                .style(dark_button)
                .padding([8, 16]),
        ]
        .spacing(8);

        column![
            scrollable(
                container(content_col)
                    .padding(iced::Padding { right: 32.0, ..iced::Padding::ZERO })
            )
            .height(Length::Fill),
            buttons,
        ]
        .spacing(12)
        .height(Length::Fill)
        .into()
    }
}
