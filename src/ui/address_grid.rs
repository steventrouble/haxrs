use std::{
    convert::TryInto,
    sync::atomic::{AtomicUsize, Ordering},
};

use egui::RichText;
use egui_extras::{Size, TableRow};

use crate::windex::Process;

macro_rules! parse_bytes {
    ($value:expr, $tye:ty) => {
        <$tye>::from_le_bytes($value.try_into().expect("Invalid size")).to_string()
    };
}

#[derive(Default, PartialEq)]
pub enum DataType {
    #[default]
    FourBytes,
    EightBytes,
}

const ALL_DATA_TYPES: [DataType; 2] = [DataType::FourBytes, DataType::EightBytes];

impl DataType {
    /// Gets the display name of the given DataType.
    const fn name(&self) -> &str {
        match *self {
            DataType::FourBytes => "4 bytes",
            DataType::EightBytes => "8 bytes",
        }
    }

    /// Gets the size in bytes of the given DataType.
    const fn size_of(&self) -> usize {
        match *self {
            DataType::FourBytes => 4,
            DataType::EightBytes => 8,
        }
    }

    /// Parses the given DataType from a Vec of bytes.
    fn parse_value(&self, value: Vec<u8>) -> String {
        let value = match *self {
            DataType::FourBytes => {
                parse_bytes!(value, i32)
            }
            DataType::EightBytes => {
                parse_bytes!(value, i64)
            }
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
        egui_extras::TableBuilder::new(ui)
            .column(Size::relative(0.25).at_least(40.0))
            .column(Size::exact(100.0))
            .column(Size::remainder().at_least(40.0))
            .header(20.0, |mut header| {
                header.header_col("Address");
                header.header_col("Type");
                header.header_col("Value");
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
                    })
                }
            });
        if ui.button("+ Add Row").clicked() {
            self.addresses.push(UserAddress::new("".to_string()));
        }
    }
}

/// Allow text headers to be added with a single call.
trait HeaderCol {
    fn header_col(&mut self, text: impl Into<RichText>);
}

impl<'a, 'b> HeaderCol for TableRow<'a, 'b> {
    /// Add a header column with text.
    fn header_col(&mut self, text: impl Into<RichText>) {
        self.col(|ui| {
            ui.heading(text);
        });
    }
}

/// Returns the value at the given address as a string.
fn get_address_value(process: &Process, addr: &UserAddress) -> String {
    let address: Result<usize, _> = usize::from_str_radix(&addr.address, 16);
    if let Ok(address) = address {
        let mem = process.get_mem_at(address, addr.data_type.size_of());
        if let Ok(mem) = mem {
            return addr.data_type.parse_value(mem);
        }
    }
    "???".to_string()
}
