use eframe::{egui, epi};
use egui::{
    Align, Align2, Button, Color32, Context, Label, Layout, Rgba, RichText, ScrollArea, Style,
    TopBottomPanel, Ui, Vec2, Widget, Window,
};
use epi::{Frame, Storage};
use serialport::{
    ClearBuffer, DataBits, FlowControl, Parity, SerialPort, SerialPortInfo, StopBits,
};
use std::fmt::format;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Copy, Clone, Debug, Default)]
pub struct MenuBar {
    show_settings_window: bool,
    show_about_window: bool,
}

#[derive(Clone, Debug)]
pub struct DeviceOpenOptions {
    available_ports: Vec<SerialPortInfo>,
    selected_device: String,
    baudrate_list: Vec<u32>,
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
            selected_device: String::new(),
            baudrate_list: Vec::new(),
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

pub struct DeviceUnavailable;

pub struct BCom {
    connected_device: Option<Box<dyn SerialPort>>,
    menu_bar: MenuBar,
    device_open_options: DeviceOpenOptions,
    display_panel: DisplayPanel,
    command_panel: CommandPanel,
    status_bar: StatusBar,
    app_message: String,
}

impl BCom {
    pub fn new() -> Self {
        Self {
            connected_device: None,
            menu_bar: MenuBar::default(),
            device_open_options: DeviceOpenOptions::new(),
            display_panel: DisplayPanel::new(),
            command_panel: CommandPanel::new(),
            status_bar: StatusBar::default(),
            app_message: String::new(),
        }
    }

    pub fn handle_menu_bar_actions(&mut self, ctx: &Context, ui: &mut Ui) {
        if self.menu_bar.show_about_window {
            Window::new("About").collapsible(false).show(ctx, |ui| {
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
            Window::new("Settings").collapsible(false).show(ctx, |ui| {
                ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
                    let confirm_button = ui.add(Button::new("Close"));
                    if confirm_button.clicked() {
                        self.menu_bar.show_settings_window = false;
                    }
                    ui.vertical_centered(|ui| ui.add(Label::new("Add something here")));
                });
            });
        }
    }

    pub fn render_menu_bar(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        TopBottomPanel::top("menu_bar").show(ctx, |ui| {
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
        TopBottomPanel::bottom("status_bar")
            .min_height(30.0)
            .show(ctx, |ui| {
                ui.with_layout(Layout::left_to_right(), |ui| {
                    ui.add_space(3.0);
                    let reset_button = ui
                        .add(Button::new("Reset"))
                        .on_hover_text("Reset RX and TX count");
                    if reset_button.clicked() {
                        self.status_bar.bytes_received = 0;
                        self.status_bar.bytes_sent = 0;
                        self.app_message.clear();
                    }

                    ui.add_space(50.0);
                    let rx_count = ui.add(Label::new(format!(
                        "RX: {}",
                        self.status_bar.bytes_received
                    )));
                    ui.add_space(50.0);
                    let tx_count =
                        ui.add(Label::new(format!("TX: {}", self.status_bar.bytes_sent)));

                    ui.add_space(50.0);
                    ScrollArea::new([true, false]).show(ui, |ui| {
                        ui.label(RichText::new(&self.app_message).color(Color32::RED));
                    });
                });

                self.status_bar.bytes_received += 1;
                self.status_bar.bytes_sent += 1;
            });
    }

    pub fn render_open_options(&mut self, ctx: &Context, ui: &mut Ui) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label("Devices");
                let device_selected = if self.device_open_options.selected_device.is_empty() {
                    "None"
                } else {
                    &self.device_open_options.selected_device
                };
                egui::ComboBox::from_id_source("device_combobox")
                    .selected_text(device_selected)
                    .show_ui(ui, |ui| {
                        // todo: should call only once
                        match serialport::available_ports() {
                            Ok(ports) => {
                                self.device_open_options.available_ports = ports;
                                for p in self.device_open_options.available_ports.iter() {
                                    ui.selectable_value(
                                        &mut self.device_open_options.selected_device,
                                        p.port_name.clone(),
                                        p.port_name.clone(),
                                    );
                                }
                            }
                            Err(e) => {
                                self.app_message = "Cannot enumerate available devices".to_string();
                            }
                        };
                    });
                let status_color = if self.connected_device.is_some() {
                    Color32::GREEN
                } else {
                    Color32::RED
                };
                let open_button = ui.button("Open");
                let device_status = ui.add_enabled(false, Button::new("    ").fill(status_color));
                let close_button = ui.button("Close");
                if open_button.clicked() {
                    match serialport::new(
                        &self.device_open_options.selected_device,
                        self.device_open_options.baudrate,
                    )
                    .parity(self.device_open_options.parity)
                    .data_bits(self.device_open_options.data_bits)
                    .stop_bits(self.device_open_options.stop_bits)
                    .flow_control(self.device_open_options.flow_control)
                    .open()
                    {
                        Ok(port) => {
                            self.connected_device = Some(port);
                        }
                        Err(e) => {
                            self.app_message = format!(
                                "Cannot open device {}",
                                self.device_open_options.selected_device
                            );
                        }
                    }
                }
                if close_button.clicked() {
                    self.connected_device = None;
                }

                ui.add_space(20.0);
                ui.label("Baudrate");
                egui::ComboBox::from_id_source("baudrate_combobox")
                    .selected_text(self.device_open_options.baudrate.to_string())
                    .show_ui(ui, |ui| {
                        for &b in self.device_open_options.baudrate_list.iter() {
                            ui.selectable_value(
                                &mut self.device_open_options.baudrate,
                                b,
                                b.to_string(),
                            );
                        }
                    });
            });
            ui.horizontal(|ui| {
                ui.label("Parity");
            });
            ui.horizontal(|ui| {
                ui.label("Flow Control");
            });
        });
    }
}

impl epi::App for BCom {
    fn update(&mut self, ctx: &egui::Context, frame: &epi::Frame) {
        self.render_menu_bar(ctx, frame);
        self.render_status_bar(ctx, frame);
        egui::CentralPanel::default().show(ctx, |ui| {
            self.handle_menu_bar_actions(ctx, ui);
            ui.horizontal(|ui| {
                self.render_open_options(ctx, ui);
            });
        });
    }

    fn setup(&mut self, _ctx: &Context, _frame: &Frame, storage: Option<&dyn Storage>) {
        if let Some(data) = storage {
            if let Some(baudrate) = data.get_string("favorite_baudrate") {
                self.device_open_options.baudrate = baudrate.parse().unwrap_or(9600);
            }

            self.device_open_options.baudrate_list =
                vec![2400, 4800, 9600, 19200, 115200, 230400, 460800];
            if let Some(list) = data.get_string("baudrate_list") {
                for b in list.split_whitespace() {
                    match b.parse() {
                        Ok(n) => {
                            self.device_open_options.baudrate_list.push(n);
                        }
                        Err(_) => {
                            self.app_message = "Cannot load baudrate list from config".to_string();
                            break;
                        }
                    }
                }
            }
            self.device_open_options.baudrate_list.sort();
            self.device_open_options.baudrate_list.dedup();
        }
    }

    fn save(&mut self, storage: &mut dyn Storage) {
        // Save the last used baudrate.
        storage.set_string(
            "favorite_baudrate",
            self.device_open_options.baudrate.to_string(),
        );
        let baud_list = self
            .device_open_options
            .baudrate_list
            .iter()
            .map(|b| format!("{} ", b))
            .collect::<String>();
        storage.set_string("baudrate_list", baud_list);
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
