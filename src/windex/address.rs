use std::convert::TryInto;

macro_rules! from_bytes {
    ($value:expr, $tye:ty) => {
        <$tye>::from_ne_bytes($value.try_into().expect("Invalid size")).to_string()
    };
}

macro_rules! to_bytes {
    ($value:expr, $tye:ty) => {
        match $value.parse::<$tye>() {
            Ok(parsed) => Ok(parsed.to_ne_bytes().to_vec()),
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

pub const ALL_DATA_TYPES: [DataType; 4] = [
    DataType::FourBytes,
    DataType::EightBytes,
    DataType::Float,
    DataType::Double,
];

impl DataType {
    /// Gets the display name of the given DataType.
    pub const fn name(&self) -> &str {
        match *self {
            DataType::FourBytes => "4 bytes",
            DataType::EightBytes => "8 bytes",
            DataType::Float => "Float",
            DataType::Double => "Double",
        }
    }

    /// Gets the size in bytes of the given DataType.
    pub const fn size_of(&self) -> usize {
        match *self {
            DataType::FourBytes => 4,
            DataType::EightBytes => 8,
            DataType::Float => 4,
            DataType::Double => 8,
        }
    }

    /// Parses the given DataType from a Vec of bytes.
    pub fn from_bytes(&self, value: Vec<u8>) -> String {
        let value = match *self {
            DataType::FourBytes => from_bytes!(value, i32),
            DataType::EightBytes => from_bytes!(value, i64),
            DataType::Float => from_bytes!(value, f32),
            DataType::Double => from_bytes!(value, f64),
        };
        value
    }

    /// Parses the given String from a Vec of bytes.
    pub fn to_bytes(&self, value: &String) -> Result<Vec<u8>, String> {
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
pub struct Address {
    pub address: usize,
    pub data_type: DataType,
}

impl From<String> for Address {
    fn from(address: String) -> Self {
        let mut addr = Address::default();
        addr.address = usize::from_str_radix(&address, 16).unwrap_or(0);
        addr
    }
}

impl From<usize> for Address {
    fn from(address: usize) -> Self {
        let mut addr = Address::default();
        addr.address = address;
        addr
    }
}
