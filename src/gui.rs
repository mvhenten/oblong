use iced::widget::{
    button, column, container, horizontal_rule, pick_list, row, scrollable, text, text_input,
    Column,
};
use iced::{alignment, color, window, Border, Element, Length, Task, Theme};

use crate::config::*;
use crate::outputs::*;

pub fn run() -> iced::Result {
    iced::application("Oblong", App::update, App::view)
        .theme(|_| Theme::Dark)
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
}

// ── App state ───────────────────────────────────────────────

struct App {
    tab: Tab,
    // Shortcuts
    groups: Vec<BindingGroup>,
    // Displays
    sway_outputs: Vec<SwayOutput>,
    output_configs: Vec<OutputConfig>,
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
                let configs = load_output_configs().unwrap_or_else(|| {
                    let mut cfgs: Vec<OutputConfig> =
                        outputs.iter().map(config_from_sway).collect();
                    infer_relative_positions(&mut cfgs, &outputs);
                    cfgs
                });
                (outputs, configs)
            }
            Err(_) => (vec![], vec![]),
        };

        Self {
            tab: Tab::Shortcuts,
            groups,
            sway_outputs,
            output_configs,
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
    OutputPosition(usize, String),
    OutputPositionTarget(usize, String),
    SaveDisplays,
    SaveAndReloadDisplays,
    RefreshOutputs,
    // Shared
    Quit,
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
                    if let Some(sway_out) = self.sway_outputs.iter().find(|o| o.name == conf.name) {
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
                    Err(e) => self.status = format!("✗ {e}"),
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

        row![shortcut_tab, display_tab]
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
            let sway_out = self.sway_outputs.iter().find(|o| o.name == conf.name);
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
            let scale_input = text_input("1.0", &scale_val)
                .on_input(move |val| Message::OutputScale(i, val))
                .size(13)
                .width(Length::Fixed(80.0));
            let scale_row = row![scale_label, scale_input]
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
}
