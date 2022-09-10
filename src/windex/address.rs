use super::{data_type, DataTypeTrait};

/// Represents an address in memory.
pub struct Address {
    pub address: usize,
    pub data_type: Box<dyn DataTypeTrait>,
}

impl Default for Address {
    fn default() -> Self {
        Address {
            address: 0x0,
            data_type: Box::new(data_type::FourBytes),
        }
    }
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
