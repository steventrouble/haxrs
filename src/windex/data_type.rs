/// Possible selections for the "data type" combo box.
#[derive(PartialEq, Default, Clone, Copy)]
pub enum DataTypeEnum {
    #[default]
    FourBytes,
    EightBytes,
    Float,
    Double,
}

/// All possible selections for the "data type" combo box.
pub const ALL_DATA_TYPES: [DataTypeEnum; 4] = [
    DataTypeEnum::FourBytes,
    DataTypeEnum::EightBytes,
    DataTypeEnum::Float,
    DataTypeEnum::Double,
];

impl DataTypeEnum {
    /// Get the associated info (byte sizes, etc) for a data type.
    pub fn info(&self) -> Box<dyn DataTypeTrait> {
        match *self {
            DataTypeEnum::FourBytes => Box::new(FourBytes),
            DataTypeEnum::EightBytes => Box::new(EightBytes),
            DataTypeEnum::Float => Box::new(Float),
            DataTypeEnum::Double => Box::new(Double),
        }
    }
}

// I wish the standard types implemented traits for things like
// from_ne_bytes and such...
macro_rules! default_data_type_fns {
  ($tye:ty) => {
      fn size_of(&self) -> usize {
          std::mem::size_of::<$tye>()
      }

      fn from_bytes(&self, value: &[u8]) -> String {
          <$tye>::from_ne_bytes(value.try_into().expect("Invalid size")).to_string()
      }

      fn to_bytes(&self, value: &String) -> Result<Vec<u8>, String> {
          match value.parse::<$tye>() {
              Ok(parsed) => Ok(parsed.to_ne_bytes().to_vec()),
              _ => Err("Parse Error".to_string()),
          }
      }
  };
}

/// All the info needed to perform operations on data types in memory.
pub trait DataTypeTrait {
  fn name(&self) -> &str;

  fn size_of(&self) -> usize;
  fn from_bytes(&self, value: &[u8]) -> String;
  fn to_bytes(&self, value: &String) -> Result<Vec<u8>, String>;
  fn display(&self, value: &[u8]) -> String;
}

/// Info about four-byte words.
pub struct FourBytes;
impl DataTypeTrait for FourBytes {
  fn name(&self) -> &str {
      "4 bytes"
  }

  fn display(&self, value: &[u8]) -> String {
    format!("{}", i32::from_ne_bytes(value.try_into().expect("Invalid size")))
  }

  default_data_type_fns!(i32);
}

/// Info about eight-byte dwords.
pub struct EightBytes;
impl DataTypeTrait for EightBytes {
  fn name(&self) -> &str {
      "8 bytes"
  }

  fn display(&self, value: &[u8]) -> String {
    format!("{}", i64::from_ne_bytes(value.try_into().expect("Invalid size")))
  }

  default_data_type_fns!(i64);
}

/// Info about four-byte floats.
pub struct Float;
impl DataTypeTrait for Float {
  fn name(&self) -> &str {
      "Float"
  }

  fn display(&self, value: &[u8]) -> String {
    let val = f32::from_ne_bytes(value.try_into().expect("Invalid size"));
    if val.log2() < 10.0 && val.log2() > -10.0 {
      format!("{:.2}", val)
    } else {
      format!("{:e}", val)
    }
  }

  default_data_type_fns!(f32);
}

/// Info about eight-byte doubles.
pub struct Double;
impl DataTypeTrait for Double {
  fn name(&self) -> &str {
      "Double"
  }

  fn display(&self, value: &[u8]) -> String {
    let val = f64::from_ne_bytes(value.try_into().expect("Invalid size"));
    if val.log2() < 10.0 && val.log2() > -10.0 {
      format!("{:.2}", val)
    } else {
      format!("{:.2e}", val)
    }
  }

  default_data_type_fns!(f64);
}
