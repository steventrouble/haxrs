use crate::windex::Process;
use cached::proc_macro::cached;

struct SearchAddress {
    address: String,
}

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
    results: Vec<SearchAddress>,
}

impl SearchResults {
    pub fn show(&mut self, ui: &mut egui::Ui) {
        ui.vertical(|ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .min_scrolled_height(150.0)
                .show(ui, |ui| {
                    for addr in self.results.iter().take(1000) {
                        let address = &addr.address;
                        ui.label(address.to_string());
                    }
                });
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
            let text = ui.text_edit_singleline(&mut self.search_text);

            if text.lost_focus() && text.ctx.input().key_pressed(egui::Key::Enter) {
                // user pressed enter in the text area
                results.results.push(SearchAddress {
                    address: "aaaa".to_string(),
                });
                text.request_focus();
            }

            if ui.button("Search").clicked() {
                results.results.push(SearchAddress {
                    address: "aaaa".to_string(),
                });
            }
        });
    }
}
