use std::{
    convert::TryInto,
    sync::atomic::{AtomicUsize, Ordering},
};

use egui::WidgetText;
use egui_extras::{Size, TableRow};
use cached::proc_macro::cached;

use crate::windex::Process;

macro_rules! from_bytes {
    ($value:expr, $tye:ty) => {
        <$tye>::from_le_bytes($value.try_into().expect("Invalid size")).to_string()
    };
}

macro_rules! to_bytes {
    ($value:expr, $tye:ty) => {
        match $value.parse::<$tye>() {
            Ok(parsed) => Ok(parsed.to_le_bytes().to_vec()),
            _ => Err("Parse Error".to_string()),
        }
    };
}

#[derive(Default, PartialEq)]
pub enum DataType {
    #[default]
    FourBytes,
    EightBytes,
    Float,
    Double,
}

const ALL_DATA_TYPES: [DataType; 4] = [
    DataType::FourBytes,
    DataType::EightBytes,
    DataType::Float,
    DataType::Double,
];

impl DataType {
    /// Gets the display name of the given DataType.
    const fn name(&self) -> &str {
        match *self {
            DataType::FourBytes => "4 bytes",
            DataType::EightBytes => "8 bytes",
            DataType::Float => "Float",
            DataType::Double => "Double",
        }
    }

    /// Gets the size in bytes of the given DataType.
    const fn size_of(&self) -> usize {
        match *self {
            DataType::FourBytes => 4,
            DataType::EightBytes => 8,
            DataType::Float => 4,
            DataType::Double => 8,
        }
    }

    /// Parses the given DataType from a Vec of bytes.
    fn from_bytes(&self, value: Vec<u8>) -> String {
        let value = match *self {
            DataType::FourBytes => from_bytes!(value, i32),
            DataType::EightBytes => from_bytes!(value, i64),
            DataType::Float => from_bytes!(value, f32),
            DataType::Double => from_bytes!(value, f64),
        };
        value
    }

    /// Parses the given String from a Vec of bytes.
    fn to_bytes(&self, value: &String) -> Result<Vec<u8>, String> {
        let value = match *self {
            DataType::FourBytes => to_bytes!(value, i32),
            DataType::EightBytes => to_bytes!(value, i64),
            DataType::Float => to_bytes!(value, f32),
            DataType::Double => to_bytes!(value, f64),
        };
        value
    }
}

/// Represents a user-input address.
#[derive(Default)]
pub struct UserAddress {
    id: usize,
    pub address: String,
    pub data_type: DataType,
    pub requested_val: String,
}

impl UserAddress {
    /// Create a new UserAddress.
    pub fn new(address: String) -> UserAddress {
        static ID: AtomicUsize = AtomicUsize::new(0);
        let mut addr = UserAddress::default();
        addr.address = address;
        addr.id = ID.fetch_add(1, Ordering::Relaxed);
        addr
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
                                .selected_text(format!("{}", addr.data_type.name()))
                                .width(ui.available_width() - 8.0)
                                .show_ui(ui, |ui| {
                                    for data_type in ALL_DATA_TYPES {
                                        let name = &data_type.name().to_owned();
                                        ui.selectable_value(&mut addr.data_type, data_type, name);
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
        if ui.button("+ Add Row").clicked() {
            self.addresses.push(UserAddress::new("".to_string()));
        }
    }
}

/// Returns the value at the given address as a string.
fn get_address_value(process: &Process, addr: &UserAddress) -> String {
    let address: Result<usize, _> = usize::from_str_radix(&addr.address, 16);
    if let Ok(address) = address {
        let mem = get_mem_cached(process, address, addr.data_type.size_of());
        if let Ok(mem) = mem {
            return addr.data_type.from_bytes(mem);
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
    let bytes = addr.data_type.to_bytes(&addr.requested_val);
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
