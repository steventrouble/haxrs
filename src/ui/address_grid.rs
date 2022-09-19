use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use cached::proc_macro::cached;
use egui::WidgetText;
use egui_extras::TableBody;
use egui_extras::{Size, TableRow};

use super::TypeComboBox;
use crate::windex::DataTypeEnum;
use crate::windex::Process;

static ADDRESS_ID: AtomicUsize = AtomicUsize::new(0);

/// The information the user provided for each address.
pub struct UserAddress {
    pub id: usize,
    pub description: String,
    pub address: String,
    pub data_type: DataTypeEnum,
    pub requested_val: String,
}

impl UserAddress {
    pub fn new() -> Self {
        UserAddress {
            id: ADDRESS_ID.fetch_add(1, Ordering::Relaxed),
            description: String::new(),
            address: String::new(),
            data_type: DataTypeEnum::default(),
            requested_val: String::new(),
        }
    }
}

/// A table of addresses and values, and buttons for editing them.
#[derive(Default)]
pub struct AddressGrid {
    pub addresses: Vec<UserAddress>,
}

impl AddressGrid {
    pub fn show(self: &mut Self, ui: &mut egui::Ui, process: &Process) {
        ui.heading("Address List");
        ui.vertical_centered_justified(|ui| {
            ui.set_height(ui.available_height() - 20.0);
            egui_extras::TableBuilder::new(ui)
                .resizable(true)
                .column(Size::relative(0.25).at_least(40.0))
                .column(Size::relative(0.25).at_least(40.0))
                .column(Size::initial(100.0).at_least(40.0))
                .column(Size::remainder().at_least(40.0))
                .column(Size::remainder().at_least(40.0))
                .column(Size::initial(30.0).at_least(20.0))
                .column(Size::initial(30.0).at_least(20.0))
                .header(20.0, |mut header| {
                    header_col(&mut header, "Description");
                    header_col(&mut header, "Address");
                    header_col(&mut header, "Type");
                    header_col(&mut header, "Value");
                    header_col(&mut header, "Edit");
                    header_col(&mut header, "✔");
                    header_col(&mut header, "❌");
                })
                .body(|mut body| {
                    let mut to_remove: Vec<usize> = vec![];
                    self.render_address_row(&mut body, &mut to_remove, process);
                    for idx in to_remove {
                        self.addresses.remove(idx);
                    }
                });
        });
        if ui.button("+ Add Row").clicked() {
            self.addresses.push(UserAddress::new());
        }
    }

    fn render_address_row(
        &mut self,
        body: &mut TableBody,
        to_remove: &mut Vec<usize>,
        process: &Process,
    ) {
        for (idx, addr) in self.addresses.iter_mut().enumerate() {
            body.row(20.0, |mut row| {
                row.col(|ui| {
                    ui.text_edit_singleline(&mut addr.description);
                });
                row.col(|ui| {
                    ui.text_edit_singleline(&mut addr.address);
                });
                row.col(|ui| {
                    addr.data_type.show(ui, addr.id);
                });
                row.col(|ui| {
                    ui.label(get_address_value(process, addr));
                });
                row.col(|ui| {
                    ui.text_edit_singleline(&mut addr.requested_val);
                });
                row.col(|ui| {
                    if ui.button("Set").clicked() {
                        set_address_value(process, addr);
                    }
                });
                row.col(|ui| {
                    if ui.button("Del").clicked() {
                        to_remove.push(idx);
                    }
                });
            })
        }
    }
}

/// Returns the value at the given address as a string.
fn get_address_value(process: &Process, addr: &UserAddress) -> String {
    let data_type = addr.data_type.info();
    let address: Result<usize, _> = usize::from_str_radix(&addr.address, 16);
    if let Ok(address) = address {
        let mem = get_mem_cached(process, address, data_type.size_of());
        if let Ok(mem) = mem {
            return data_type.display(&mem);
        }
    }
    "???".to_string()
}

#[cached(time = 1, key = "(usize, usize)", convert = r#"{ (address, size) }"#)]
fn get_mem_cached(process: &Process, address: usize, size: usize) -> Result<Vec<u8>, String> {
    process.get_mem_at(address, size)
}

/// Set the value at the given address.
fn set_address_value(process: &Process, addr: &UserAddress) {
    let bytes = addr.data_type.info().to_bytes(&addr.requested_val);
    let address: Result<usize, _> = usize::from_str_radix(&addr.address, 16);

    if let (Ok(bytes), Ok(address)) = (bytes, address) {
        let result = process.set_mem_at(address, bytes);
        if result.is_err() {
            panic!("Error writing.")
        }
    }
}

/// Add a header column with text.
fn header_col(header: &mut TableRow, text: impl Into<WidgetText>) {
    header.col(|ui| {
        ui.label(text);
    });
}
