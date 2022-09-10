
// I wish the standard types implemented traits for things like
// from_ne_bytes and such...
macro_rules! default_data_type_fns {
  ($tye:ty) => {
      fn size_of(&self) -> usize {
          std::mem::size_of::<$tye>()
      }

      fn from_bytes(&self, value: Vec<u8>) -> String {
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
  fn from_bytes(&self, value: Vec<u8>) -> String;
  fn to_bytes(&self, value: &String) -> Result<Vec<u8>, String>;
}

/// Info about four-byte words.
pub struct FourBytes;
impl DataTypeTrait for FourBytes {
  fn name(&self) -> &str {
      "4 bytes"
  }

  default_data_type_fns!(i32);
}

/// Info about eight-byte dwords.
pub struct EightBytes;
impl DataTypeTrait for EightBytes {
  fn name(&self) -> &str {
      "8 bytes"
  }

  default_data_type_fns!(i64);
}

/// Info about four-byte floats.
pub struct Float;
impl DataTypeTrait for Float {
  fn name(&self) -> &str {
      "Float"
  }

  default_data_type_fns!(f32);
}

/// Info about eight-byte doubles.
pub struct Double;
impl DataTypeTrait for Double {
  fn name(&self) -> &str {
      "Double"
  }

  default_data_type_fns!(f64);
}