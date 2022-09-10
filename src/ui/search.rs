use crate::windex::{scanner, Process};
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
    pub fn show(&mut self, ui: &mut egui::Ui, process: &Process) {
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
                    let num_results = self.results.len();
                    ui.label(format!("{num_results} results"));
                    for addr in self.results.iter().take(1000) {
                        ui.label(&addr.address.to_string());
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
    pub fn show(&mut self, ui: &mut egui::Ui, results: &mut SearchResults, process: &Process) {
        ui.horizontal(|ui| {
            let text = ui.text_edit_singleline(&mut self.search_text);

            if text.lost_focus() && text.ctx.input().key_pressed(egui::Key::Enter) {
                let found = self.scan(process);
                results.results.extend(found.iter().map(|x| SearchAddress {
                    address: format!("{x:x}"),
                }));
                text.request_focus();
            }

            if ui.button("Search").clicked() {
                let found = self.scan(process);
                results.results.extend(found.iter().map(|x| SearchAddress {
                    address: format!("{x:x}"),
                }));
            }
        });
    }

    fn scan(&self, process: &Process) -> Vec<usize> {
        let search_val: Result<i32, _> = self.search_text.parse();
        if search_val.is_err() {
            return vec![];
        }
        let search_val = search_val.unwrap().to_ne_bytes();
        scanner::scan(process, &search_val)
    }
}
