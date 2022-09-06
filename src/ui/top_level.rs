use egui::{Color32, ScrollArea, TextEdit};

use super::AddressGrid;
use crate::windex::Process;

#[derive(Default)]
pub struct MainApp {
    address_grid: AddressGrid,
    connect_menu: ConnectMenu,
    selected_process: Option<Process>,
}

impl MainApp {
    pub fn new(_: &eframe::CreationContext<'_>) -> Self {
        MainApp::default()
    }
}

impl eframe::App for MainApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut visuals = egui::Visuals::light();
        visuals.widgets.noninteractive.fg_stroke.color = Color32::BLACK;
        visuals.widgets.active.fg_stroke.color = Color32::BLACK;
        visuals.widgets.inactive.fg_stroke.color = Color32::BLACK;
        visuals.widgets.hovered.fg_stroke.color = Color32::BLACK;
        visuals.widgets.open.fg_stroke.color = Color32::BLACK;
        ctx.set_visuals(visuals);

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                ui.vertical_centered(|ui| {
                    if let Some(process) = &self.selected_process {
                        ui.label(process.get_name());
                    } else {
                        ui.label("Not connected");
                    }
                    let selected: Option<Process> = ui
                        .menu_button("Connect to Process", |ui| self.connect_menu.menu(ui))
                        .inner
                        .flatten();
                    if selected.is_some() {
                        self.selected_process = selected;
                    }
                });

                if let Some(process) = &self.selected_process {
                    ui.separator();
                    self.address_grid.show(ui, &process);
                }
            });
        });
    }
}

/// A generic popup menu like the command menu in VSCode.
///
/// Shows a list of options and allows the user to search and select one
/// using only their keyboard.
#[derive(Default)]
struct ConnectMenu {
    search_text: String,
}

impl ConnectMenu {
    fn menu(&mut self, ui: &mut egui::Ui) -> Option<Process> {
        let mut retval: Option<Process> = None;
        ui.set_min_width(200.0);
        ui.vertical_centered_justified(|ui| {
            let mut processes = Process::get_processes().unwrap_or_else(|_| vec![]);
            processes.retain(|p| p.get_name().to_lowercase().contains(&self.search_text));

            let text = ui.add(TextEdit::singleline(&mut self.search_text));
            if text.lost_focus() && text.ctx.input().key_pressed(egui::Key::Enter) {
                retval = processes.drain(0..1).next(); // get first process
                self.close_menu(ui);
                return;
            }
            text.request_focus();
            ui.separator();

            let scroll_area = ScrollArea::vertical()
                .max_height(200.0)
                .auto_shrink([false; 2]);

            scroll_area.show(ui, |ui| {
                for p in processes {
                    let name = p.get_name();
                    if ui.button(format!("{name}")).clicked() {
                        retval = Some(p);
                        self.close_menu(ui);
                        break;
                    }
                }
            });
        });
        retval
    }

    fn close_menu(&mut self, ui: &mut egui::Ui) {
        ui.close_menu();
        self.search_text = "".to_string();
    }
}
