use crate::windex::{data_type::ALL_DATA_TYPES, DataTypeEnum};

/// A combo box that can select from several data types.
pub trait TypeComboBox {
    fn show(&mut self, ui: &mut egui::Ui, id: usize);
}

impl TypeComboBox for DataTypeEnum {
    fn show(&mut self, ui: &mut egui::Ui, id: usize) {
        egui::ComboBox::from_id_source(id)
            .selected_text(format!("{}", self.info().name()))
            .show_ui(ui, |ui| {
                for data_type in ALL_DATA_TYPES {
                    let name = &data_type.info().name().to_owned();
                    ui.selectable_value(self, data_type, name);
                }
            });
    }
}
