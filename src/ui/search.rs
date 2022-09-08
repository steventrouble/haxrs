use super::address_grid::UserAddress;
use crate::windex::Process;
use cached::proc_macro::cached;

#[derive(Default)]
pub struct Search {
    results: SearchResults,
    tools: SearchTools,
}

impl Search {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.heading("Search");
        ui.horizontal(|ui| {
            // Search tools
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() / 2.0);
                self.tools.show(ui, &mut self.results);
            });

            // Search results
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                self.results.show(ui);
            });
        });
    }
}

#[derive(Default)]
struct SearchResults {
    results: Vec<UserAddress>,
}

impl SearchResults {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            for addr in self.results.iter().take(1000) {
                let address = &addr.address;
                ui.label(address.to_string());
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
}

impl SearchTools {
    pub fn show(&mut self, ui: &mut egui::Ui, results: &mut SearchResults) {
        ui.horizontal(|ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.text_edit_singleline(&mut self.search_text);
                if ui.button("Search").clicked() {
                    results
                        .results
                        .push(UserAddress::new("21195DF8408".to_string()))
                }
            });
        });
    }
}
