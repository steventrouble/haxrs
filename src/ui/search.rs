use std::sync::atomic::Ordering;
use std::sync::{mpsc, atomic::AtomicBool};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::thread;

use crate::windex::scanner::SearchResult;
use crate::parser;
use crate::windex::{scanner, Process};
use cached::proc_macro::cached;
use egui::Layout;

use super::{address_grid::UserAddress, AddressGrid};

#[derive(Default)]
pub struct Search {
    results: SearchResults,
    tools: SearchTools,
}

impl Search {
    /// Display the search results
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        process: &Arc<Process>,
        address_grid: &mut AddressGrid,
    ) {
        ui.heading("Search");
        ui.horizontal(|ui| {
            // Search tools
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() / 2.0);
                self.tools.show(ui, &mut self.results, process);
            });

            // Search results
            ui.vertical(|ui| {
                ui.set_width(ui.available_width());
                self.results.show(ui, address_grid);
            });
        });
    }
}

/// UI component that displays the list of search results and lets users add
/// them to the address list.
#[derive(Default)]
struct SearchResults {
    /// The list of search results.
    results: Vec<SearchResult>,
    /// For each search result, whether or not the checkbox is checked.
    checked: Vec<bool>,
    /// Emits values whenever a search result is found.
    results_rx: Option<Receiver<SearchResult>>,
    /// True if the search is still loading.
    loading: Arc<AtomicBool>,
}

impl SearchResults {
    pub fn show(&mut self, ui: &mut egui::Ui, address_grid: &mut AddressGrid) {
        if let Some(results_rx) = &self.results_rx {
            for r in results_rx.try_iter() {
                self.results.push(r);
                self.checked.push(false);
            }
        }

        ui.vertical(|ui| {
            let num_results = self.results.len();
            self.show_progress(ui, num_results);
            egui::ScrollArea::vertical()
                .auto_shrink([false, true])
                .min_scrolled_height(150.0)
                .show(ui, |ui| {
                    for (idx, result) in self.results.iter().take(1000).enumerate() {
                        let addr = result.address;
                        let value = result.value_to_string();
                        ui.checkbox(&mut self.checked[idx], format!("{addr:x} - {value}"));
                    }
                });
            if ui.button("+ Add Selected").clicked() {
                for (idx, checked) in self.checked.iter().enumerate() {
                    if *checked {
                        let result = &self.results[idx];
                        let address = result.address;

                        let mut addr = UserAddress::new();
                        addr.address = format!("{address:x}");
                        addr.data_type = result.data_type;
                        address_grid.addresses.push(addr);
                    }
                }
                self.checked.fill(false);
            }
        });
    }

    fn show_progress(&mut self, ui: &mut egui::Ui, num_results: usize) {
        if self.loading.load(Ordering::Relaxed) {
            ui.label(format!("Scanning - {num_results} results"));
        } else {
            ui.label(format!("{num_results} results"));
        }
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
    pub fn show(&mut self, ui: &mut egui::Ui, results: &mut SearchResults, process: &Arc<Process>) {
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

                if !results.results.is_empty() {
                    if ui.button("Clear").clicked() {
                        results.results_rx = None;
                        results.results.clear();
                    }
                };
            });
        });
    }

    fn scan(&self, results: &mut SearchResults, process: &Arc<Process>) {
        let (tx, rx) = mpsc::channel();
        results.results_rx = Some(rx);

        let query = parser::parse(&self.search_text);

        let process = process.clone();
        let to_filter = results.results.clone();
        results.results.clear();

        let loading = results.loading.clone();
        loading.store(true, Ordering::Relaxed);

        if let Ok(query) = query {
            thread::spawn(move || {
                scanner::scan(tx, &process, query, &to_filter);
                loading.store(false, Ordering::Relaxed);
            });
        }
    }
}
