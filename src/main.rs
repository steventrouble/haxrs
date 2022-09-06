use hax::ui::top_level::MainApp;

fn main() {
    eframe::run_native(
        "hax0rs",
        eframe::NativeOptions::default(),
        Box::new(|_| Box::new(MainApp::default())),
    );
}
