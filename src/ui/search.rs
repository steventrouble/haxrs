use crate::windex::{scanner, Process};
use cached::proc_macro::cached;
use egui::Layout;

use super::{address_grid::UserAddress, type_combo::UserDataType, AddressGrid, TypeComboBox};

#[derive(Default)]
pub struct Search {
    results: SearchResults,
    tools: SearchTools,
}

impl Search {
    pub fn show(&mut self, ui: &mut egui::Ui, process: &Process, address_grid: &mut AddressGrid) {
        ui.heading("Search");
        ui.horizontal(|ui| {
            // Search tools
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() / 2.0);
                self.tools.show(ui, &mut self.results, &process);
            });

            // Search results
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                self.results.show(ui, address_grid);
            });
        });
    }
}

#[derive(Default)]
struct SearchResults {
    results: Vec<usize>,
    checked: Vec<bool>,
    data_type: UserDataType,
}

impl SearchResults {
    pub fn show(&mut self, ui: &mut egui::Ui, address_grid: &mut AddressGrid) {
        if self.checked.len() != self.results.len() {
            self.checked.clear();
            self.checked.resize(self.results.len(), false);
        }

        ui.vertical(|ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .min_scrolled_height(150.0)
                .show(ui, |ui| {
                    let num_results = self.results.len();
                    ui.label(format!("{num_results} results"));
                    for (idx, addr) in self.results.iter().take(1000).enumerate() {
                        ui.checkbox(&mut self.checked[idx], format!("{addr:x}"));
                    }
                });
            if ui.button("+ Add Selected").clicked() {
                for (idx, checked) in self.checked.iter().enumerate() {
                    let address = self.results[idx];
                    if *checked {
                        let mut addr = UserAddress::new();
                        addr.address = format!("{address:x}");
                        addr.data_type = self.data_type;
                        address_grid.addresses.push(addr);
                    }
                }
                self.checked.fill(false);
            }
        });
    }
}

#[cached(time = 5, key = "(usize, usize)", convert = r#"{ (address, size) }"#)]
fn get_mem_cached(process: &Process, address: usize, size: usize) -> Result<Vec<u8>, String> {
    process.get_mem_at(address, size)
}

#[derive(Default)]
struct SearchTools {
    search_text: String,
    data_type: UserDataType,
}

impl SearchTools {
    pub fn show(&mut self, ui: &mut egui::Ui, results: &mut SearchResults, process: &Process) {
        ui.with_layout(Layout::top_down(egui::Align::Min), |ui| {
            // Search bar and button
            ui.horizontal(|ui| {
                let text = ui.text_edit_singleline(&mut self.search_text);

                if text.lost_focus() && text.ctx.input().key_pressed(egui::Key::Enter) {
                    self.scan(results, process);
                    text.request_focus();
                }

                let label = if results.results.is_empty() {
                    "Search"
                } else {
                    "Filter"
                };
                if ui.button(label).clicked() {
                    self.scan(results, process);
                }
            });

            // Data type combo box
            self.data_type.show(ui, 9999999);
        });
    }

    fn scan(&self, results: &mut SearchResults, process: &Process) {
        let data_type = self.data_type.info();
        let bytes = data_type.to_bytes(&self.search_text);
        if let Ok(bytes) = bytes {
            scanner::scan(&mut results.results, process, &bytes, data_type);
        }
        results.data_type = self.data_type;
    }
}
