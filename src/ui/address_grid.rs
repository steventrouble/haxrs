use egui_extras::Size;

use crate::windex::Process;

/// Represents a user-input address.
#[derive(Default)]
pub struct UserAddress {
    pub address: String,
}

/// A table of addresses and values, and buttons for editing them.
#[derive(Default)]
pub struct AddressGrid {
    pub addresses: Vec<UserAddress>,
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
                            ui.label(get_address_value(process, addr));
                        });
                    })
                }
            });
        if ui.button("+ Add Row").clicked() {
            self.addresses.push(UserAddress {
                address: "".to_string(),
            });
        }
    }
}

/// Returns the value at the given address as a string.
fn get_address_value(process: &Process, addr: &UserAddress) -> String {
    let addr: Result<usize, _> = usize::from_str_radix(&addr.address, 16);
    if let Ok(addr) = addr {
        process.get_mem_at(addr)
    } else {
        "???".to_string()
    }
}
