use eframe::{egui, epi};
use egui::{
    Align, Align2, Button, Context, Label, Layout, Rgba, Style, TopBottomPanel, Ui, Vec2, Window,
};
use epi::{Frame, Storage};
use serialport::{DataBits, FlowControl, Parity, StopBits};
use std::path::{Path, PathBuf};
use std::time::Duration;

pub const PRESET_BAUDRATE_LIST: &[u32] = &[2400, 4800, 9600, 19200, 115200, 230400, 460800];

#[derive(Copy, Clone, Debug, Default)]
pub struct MenuBar {
    show_settings_window: bool,
    show_about_window: bool,
}

#[derive(Clone, Debug)]
pub struct DeviceOpenOptions {
    available_ports: Vec<String>,
    connected_device: Option<String>,
    baudrate: u32,
    parity: Parity,
    data_bits: DataBits,
    stop_bits: StopBits,
    flow_control: FlowControl,
}

impl DeviceOpenOptions {
    fn new() -> Self {
        Self {
            available_ports: Vec::new(),
            connected_device: None,
            baudrate: 9600,
            parity: Parity::None,
            data_bits: DataBits::Eight,
            stop_bits: StopBits::One,
            flow_control: FlowControl::None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SearchBar {
    string_to_search: String,
    search_area_index: usize,
    search_results: Vec<usize>,
    current_cursor: usize,
}

impl SearchBar {
    fn new() -> Self {
        Self {
            string_to_search: String::new(),
            search_area_index: 0,
            search_results: Vec::new(),
            current_cursor: 0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum DisplayMode {
    ASCII,
    HEX,
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self::ASCII
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct DisplayOptions {
    display_data: bool,
    display_mode: DisplayMode,
}

#[derive(Clone, Debug)]
pub struct DisplayPanel {
    data_on_display: String,
    search_bar: SearchBar,
    display_options: DisplayOptions,
}

impl DisplayPanel {
    fn new() -> Self {
        Self {
            data_on_display: String::new(),
            search_bar: SearchBar::new(),
            display_options: DisplayOptions::default(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct CommandPanel {
    commands_history: Vec<String>,
    command_to_send: String,
    file_history: Vec<String>,
    file_to_send: Option<PathBuf>,
    char_delay: usize,
}

impl CommandPanel {
    fn new() -> Self {
        Self {
            commands_history: Vec::new(),
            command_to_send: String::new(),
            file_history: Vec::new(),
            file_to_send: None,
            char_delay: 1,
        }
    }
}

#[derive(Clone, Default, Debug)]
pub struct StatusBar {
    opened_port: String,
    port_status: String,
    bytes_received: usize,
    bytes_sent: usize,
}

pub struct BCom {
    menu_bar: MenuBar,
    device_open_options: DeviceOpenOptions,
    display_panel: DisplayPanel,
    command_panel: CommandPanel,
    status_bar: StatusBar,
}

impl BCom {
    pub fn new() -> Self {
        Self {
            menu_bar: MenuBar::default(),
            device_open_options: DeviceOpenOptions::new(),
            display_panel: DisplayPanel::new(),
            command_panel: CommandPanel::new(),
            status_bar: StatusBar::default(),
        }
    }

    pub fn render_menu_bar(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        TopBottomPanel::top("menu bar").show(ctx, |ui| {
            ui.add_space(10.0);
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Settings", |ui| {
                    if ui.button("baudrate").clicked() {
                        println!("Config baudrate clicked");
                        self.menu_bar.show_settings_window = true;
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        println!("About BCom clicked");
                        self.menu_bar.show_about_window = true;
                    }
                });
            });
        });
    }

    pub fn render_status_bar(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        TopBottomPanel::bottom("status bar").show(ctx, |ui| {
            ui.with_layout(Layout::left_to_right(), |ui| {
                ui.add_space(3.0);
                let reset_button = ui
                    .add(Button::new("Reset"))
                    .on_hover_text("Reset RX and TX count");
                if reset_button.clicked() {
                    self.status_bar.bytes_received = 0;
                    self.status_bar.bytes_sent = 0;
                }

                ui.add_space(50.0);
                let rx_count = ui.add(Label::new(format!(
                    "RX: {}",
                    self.status_bar.bytes_received
                )));
                ui.add_space(50.0);
                let tx_count = ui.add(Label::new(format!("TX: {}", self.status_bar.bytes_sent)));
            });

            self.status_bar.bytes_received += 1;
            self.status_bar.bytes_sent += 1;
        });
    }
}

impl epi::App for BCom {
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        self.render_menu_bar(ctx, frame);
        self.render_status_bar(ctx, frame);
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            if self.menu_bar.show_about_window {
                Window::new("About")
                    .fixed_size((300.0, 300.0))
                    .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                            let confirm_button = ui.add(Button::new("Close"));
                            if confirm_button.clicked() {
                                self.menu_bar.show_about_window = false;
                            }
                            ui.vertical_centered(|ui| ui.add(Label::new("Add something here")));
                        });
                    });
            }

            if self.menu_bar.show_settings_window {
                Window::new("Settings")
                    .fixed_size((300.0, 300.0))
                    .anchor(Align2::CENTER_CENTER, (0.0, 0.0))
                    .show(ctx, |ui| {
                        ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                            let confirm_button = ui.add(Button::new("Close"));
                            if confirm_button.clicked() {
                                self.menu_bar.show_settings_window = false;
                            }
                            ui.vertical_centered(|ui| ui.add(Label::new("Add something here")));
                        });
                    });
            }
        });
    }

    fn setup(&mut self, _ctx: &Context, _frame: &Frame, storage: Option<&dyn Storage>) {
        if let Some(data) = storage {
            if let Some(baudrate) = data.get_string("baudrate") {
                self.device_open_options.baudrate = baudrate.parse().unwrap_or(9600);
            }
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        // Save the last used baudrate.
        storage.set_string("baudrate", self.device_open_options.baudrate.to_string());
    }

    fn name(&self) -> &str {
        "BCom"
    }
}

fn main() {
    let app = BCom::new();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}
