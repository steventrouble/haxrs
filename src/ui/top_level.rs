use std::sync::Arc;

use egui::{ScrollArea, TextEdit};

use super::AddressGrid;
use super::Search;
use crate::windex::Process;

/// The top-level app that shows up on startup.
#[derive(Default)]
pub struct MainApp {
    address_grid: AddressGrid,
    connect_menu: ConnectMenu,
    search: Search,

    selected_process: Option<Arc<Process>>,
}

impl eframe::App for MainApp {
    /// Called on each frame to render the UI.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        ctx.set_visuals(egui::Visuals::light());

        // Context menus
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.close();
                    }
                });
            });
        });

        // Main app
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered_justified(|ui| {
                // Process chooser
                ui.vertical_centered(|ui| {
                    ui.set_width(200.0);
                    ui.horizontal(|ui| {
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
                            self.selected_process = selected.map(|x| Arc::new(x));
                        }
                    });
                });

                if let Some(process) = &self.selected_process {
                    separator(ui);
                    self.search.show(ui, process, &mut self.address_grid);
                    separator(ui);
                    self.address_grid.show(ui, process);
                }
            });
        });
    }

    fn persist_native_window(&self) -> bool {
        true
    }
}

/// Add a custom styled separator.
fn separator(ui: &mut egui::Ui) {
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);
}

/// A generic popup menu like the command menu in VSCode.
///
/// Shows a list of options and allows the user to search and select one
/// using their mouse or keyboard.
#[derive(Default)]
struct ConnectMenu {
    search_text: String,
}

impl ConnectMenu {
    /// Display the menu. Returns the process that the user selected, if any.
    fn menu(&mut self, ui: &mut egui::Ui) -> Option<Process> {
        let mut user_selection: Option<Process> = None;

        ui.set_min_width(200.0);
        ui.vertical_centered_justified(|ui| {
            let mut processes = self.get_filtered_processes();

            let text = ui.add(TextEdit::singleline(&mut self.search_text));

            if text.lost_focus() && text.ctx.input().key_pressed(egui::Key::Enter) {
                // user pressed enter in the text area
                user_selection = processes.drain(0..1).next(); // get first process
                self.close_menu(ui);
                return;
            }

            text.request_focus();
            ui.separator();

            ScrollArea::vertical()
                .max_height(200.0)
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    for p in processes {
                        let name = p.get_name();
                        if ui.button(format!("{name}")).clicked() {
                            user_selection = Some(p);
                            self.close_menu(ui);
                            break;
                        }
                    }
                });
        });
        user_selection
    }

    /// Returns the list of process that match the current search text.
    fn get_filtered_processes(&self) -> Vec<Process> {
        let mut processes = Process::get_processes().unwrap_or_else(|_| vec![]);
        processes.retain(|p| p.get_name().to_lowercase().contains(&self.search_text));
        processes
    }

    /// Closes the context menu and resets the state.
    fn close_menu(&mut self, ui: &mut egui::Ui) {
        ui.close_menu();
        self.search_text = "".to_string();
    }
}
