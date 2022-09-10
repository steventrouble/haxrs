use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use cached::proc_macro::cached;
use egui::WidgetText;
use egui_extras::{Size, TableRow};

use crate::windex::{data_type, DataTypeTrait};
use crate::windex::Process;

static ADDRESS_ID: AtomicUsize = AtomicUsize::new(0);

/// Possible selections for the "data type" combo box.
#[derive(PartialEq)]
enum UserDataType {
    FourBytes,
    EightBytes,
    Float,
    Double,
}

/// All possible selections for the "data type" combo box.
const ALL_DATA_TYPES: [UserDataType; 4] = [
    UserDataType::FourBytes,
    UserDataType::EightBytes,
    UserDataType::Float,
    UserDataType::Double,
];

impl UserDataType {
    /// Get the associated info (byte sizes, etc) for a data type.
    fn info(&self) -> Box<dyn DataTypeTrait> {
        match *self {
            UserDataType::FourBytes => Box::new(data_type::FourBytes),
            UserDataType::EightBytes => Box::new(data_type::EightBytes),
            UserDataType::Float => Box::new(data_type::Float),
            UserDataType::Double => Box::new(data_type::Double),
        }
    }
}

/// The information the user provided for each address.
pub struct UserAddress {
    id: usize,
    address: String,
    data_type: UserDataType,
    requested_val: String,
}

impl UserAddress {
    fn new() -> Self {
        UserAddress {
            id: ADDRESS_ID.fetch_add(1, Ordering::Relaxed),
            address: "".to_string(),
            data_type: UserDataType::FourBytes,
            requested_val: "".to_string(),
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
                .column(Size::initial(100.0).at_least(40.0))
                .column(Size::remainder().at_least(40.0))
                .column(Size::remainder().at_least(40.0))
                .column(Size::initial(30.0).at_least(40.0))
                .header(20.0, |mut header| {
                    header_col(&mut header, "Address");
                    header_col(&mut header, "Type");
                    header_col(&mut header, "Value");
                    header_col(&mut header, "Edit");
                    header_col(&mut header, "✔");
                })
                .body(|mut body| {
                    for addr in self.addresses.iter_mut() {
                        body.row(20.0, |mut row| {
                            row.col(|ui| {
                                ui.text_edit_singleline(&mut addr.address);
                            });
                            row.col(|ui| {
                                let id = addr.id;
                                egui::ComboBox::from_id_source(id)
                                    .selected_text(format!("{}", addr.data_type.info().name()))
                                    .width(ui.available_width() - 8.0)
                                    .show_ui(ui, |ui| {
                                        for data_type in ALL_DATA_TYPES {
                                            let name = &data_type.info().name().to_owned();
                                            ui.selectable_value(
                                                &mut addr.data_type,
                                                data_type,
                                                name,
                                            );
                                        }
                                    });
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
                        })
                    }
                });
        });
        if ui.button("+ Add Row").clicked() {
            self.addresses.push(UserAddress::new());
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
            return data_type.from_bytes(mem);
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
