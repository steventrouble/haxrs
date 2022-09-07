#[derive(Default)]
pub struct Search {
    results: SearchResults,
    tools: SearchTools,
}

impl Search {
    pub fn show(&self, ui: &mut egui::Ui) {
        ui.heading("Search");
        ui.horizontal(|ui| {
            // Search tools
            ui.vertical(|ui| {
                ui.set_width(ui.available_width() / 2.0);
                self.tools.show(ui);
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
struct SearchResults {}

impl SearchResults {
    pub fn show(&self, ui: &mut egui::Ui) {
        ui.label("Seach results");
    }
}

#[derive(Default)]
struct SearchTools {}

impl SearchTools {
    pub fn show(&self, ui: &mut egui::Ui) {
        ui.label("Seach tools");
    }
}
