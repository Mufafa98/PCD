#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub enum Size {
    Byte(usize),
    KB(usize),
    MB(usize),
    GB(usize),
}

impl Size {
    pub fn to_bytes(self) -> usize {
        match self {
            Size::Byte(value) => value,
            Size::KB(value) => 1024 * value,
            Size::MB(value) => 1024 * 1024 * value,
            Size::GB(value) => 1024 * 1024 * 1024 * value,
        }
    }
}

use std::str::FromStr;

impl FromStr for Size {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split_idx = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
        let (num_str, unit) = s.split_at(split_idx);

        let num: usize = num_str.parse().map_err(|_| "Invalid number format")?;

        match unit.trim().to_uppercase().as_str() {
            "" | "B" | "BYTE" => Ok(Size::Byte(num)),
            "K" | "KB" => Ok(Size::KB(num)),
            "M" | "MB" => Ok(Size::MB(num)),
            "G" | "GB" => Ok(Size::GB(num)),
            _ => Err("Invalid unit (use B, KB, MB, GB)".to_string()),
        }
    }
}
