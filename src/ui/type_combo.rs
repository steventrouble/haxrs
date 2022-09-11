use crate::windex::{data_type, DataTypeTrait};

/// A combo box that can select from several data types.
pub trait TypeComboBox {
    fn show(&mut self, ui: &mut egui::Ui, id: usize);
}

impl TypeComboBox for UserDataType {
    fn show(&mut self, ui: &mut egui::Ui, id: usize) {
        egui::ComboBox::from_id_source(id)
        .selected_text(format!("{}", self.info().name()))
        .show_ui(ui, |ui| {
            for data_type in ALL_DATA_TYPES {
                let name = &data_type.info().name().to_owned();
                ui.selectable_value(
                    self,
                    data_type,
                    name,
                );
            }
        });
    }
}

/// Possible selections for the "data type" combo box.
#[derive(PartialEq, Default, Clone, Copy)]
pub enum UserDataType {
    #[default]
    FourBytes,
    EightBytes,
    Float,
    Double,
}

/// All possible selections for the "data type" combo box.
pub const ALL_DATA_TYPES: [UserDataType; 4] = [
    UserDataType::FourBytes,
    UserDataType::EightBytes,
    UserDataType::Float,
    UserDataType::Double,
];

impl UserDataType {
    /// Get the associated info (byte sizes, etc) for a data type.
    pub fn info(&self) -> Box<dyn DataTypeTrait> {
        match *self {
            UserDataType::FourBytes => Box::new(data_type::FourBytes),
            UserDataType::EightBytes => Box::new(data_type::EightBytes),
            UserDataType::Float => Box::new(data_type::Float),
            UserDataType::Double => Box::new(data_type::Double),
        }
    }
}
