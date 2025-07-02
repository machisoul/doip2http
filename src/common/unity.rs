/// Converts a hex string starting with "0x" into a Vec<u8>.
/// Returns Ok(Vec<u8>) on success, or Err(String) with an error message.
pub fn parse_hex_string_to_bytes(hex_str: &str) -> Result<Vec<u8>, String> {
  if !hex_str.starts_with("0x") {
    return Err("Hex string must start with '0x'".to_string());
  }

  let hex_digits = &hex_str[2..];

  if hex_digits.is_empty() {
    return Err("Hex string after '0x' is empty".to_string());
  }

  if hex_digits.len() % 2 != 0 {
    return Err("Hex string must have an even number of digits".to_string());
  }

  let mut bytes = Vec::with_capacity(hex_digits.len() / 2);

  for i in (0..hex_digits.len()).step_by(2) {
    let byte_str = &hex_digits[i..i + 2];
    match u8::from_str_radix(byte_str, 16) {
      Ok(byte) => bytes.push(byte),
      Err(_) => {
        return Err(format!("Invalid hex byte: {}", byte_str));
      }
    }
  }

  Ok(bytes)
}
