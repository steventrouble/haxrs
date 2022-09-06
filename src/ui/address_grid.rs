use egui_extras::Size;

use crate::windex::Process;

#[derive(Default)]
pub struct Address {
    pub address: String,
}

#[derive(Default)]
pub struct AddressGrid {
    pub addresses: Vec<Address>,
}

impl AddressGrid {
    pub fn show(self: &mut Self, ui: &mut egui::Ui, process: &Process) {
        egui_extras::TableBuilder::new(ui)
            .column(Size::relative(0.25).at_least(40.0))
            .column(Size::remainder().at_least(40.0))
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.heading("Address");
                });
                header.col(|ui| {
                    ui.heading("Value");
                });
            })
            .body(|mut body| {
                for addr in self.addresses.iter_mut() {
                    body.row(20.0, |mut row| {
                        row.col(|ui| {
                            ui.text_edit_singleline(&mut addr.address);
                        });
                        row.col(|ui| {
                            let addr: Result<usize, _> = usize::from_str_radix(&addr.address, 16);
                            let value = if let Ok(addr) = addr {
                                process.get_mem_at(addr)
                            } else {
                                "???".to_string()
                            };
                            ui.label(value);
                        });
                    })
                }
            });
        if ui.button("+ Add Row").clicked() {
            self.addresses.push(Address {
                address: "123".to_string(),
            });
        }
    }
}
