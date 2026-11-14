//! Programmer mode: base conversion and bitwise operations.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BitOp {
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Not,
}

impl BitOp {
    pub fn label(self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
            Self::Xor => "XOR",
            Self::Shl => "SHL",
            Self::Shr => "SHR",
            Self::Not => "NOT",
        }
    }

    pub const ALL: [Self; 6] = [
        Self::And,
        Self::Or,
        Self::Xor,
        Self::Shl,
        Self::Shr,
        Self::Not,
    ];
}

#[derive(Clone, Debug)]
pub struct ProgrammerState {
    pub input: String,
    pub operand: String,
    pub operation: BitOp,
    pub width: u32,
}

impl Default for ProgrammerState {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgrammerState {
    pub fn new() -> Self {
        Self {
            input: "255".to_string(),
            operand: "15".to_string(),
            operation: BitOp::And,
            width: 32,
        }
    }

    pub fn value(&self) -> Option<u64> {
        parse_integer(&self.input).map(|value| self.mask(value))
    }

    pub fn operand_value(&self) -> Option<u64> {
        parse_integer(&self.operand).map(|value| self.mask(value))
    }

    pub fn result(&self) -> Option<u64> {
        apply(
            self.value()?,
            self.operand_value()?,
            self.operation,
            self.width,
        )
    }

    pub fn mask(&self, value: u64) -> u64 {
        if self.width >= 64 {
            value
        } else {
            value & ((1_u64 << self.width) - 1)
        }
    }

    pub fn signed(&self, value: u64) -> i64 {
        if self.width >= 64 {
            value as i64
        } else {
            let sign = 1_u64 << (self.width - 1);
            if value & sign != 0 {
                (value as i64) - (1_i64 << self.width)
            } else {
                value as i64
            }
        }
    }
}

pub fn apply(a: u64, b: u64, operation: BitOp, width: u32) -> Option<u64> {
    let mask = if width >= 64 {
        u64::MAX
    } else {
        (1_u64 << width) - 1
    };
    let result = match operation {
        BitOp::And => a & b,
        BitOp::Or => a | b,
        BitOp::Xor => a ^ b,
        BitOp::Shl => a.wrapping_shl((b % width as u64) as u32),
        BitOp::Shr => a.wrapping_shr((b % width as u64) as u32),
        BitOp::Not => !a,
    };
    Some(result & mask)
}

/// Parses decimal, 0x, 0b and 0o input. A leading minus becomes two's
/// complement in 64 bits.
pub fn parse_integer(input: &str) -> Option<u64> {
    let text = input.trim().replace(['_', ' ', ','], "");
    let text = text.replace('\u{2212}', "-");
    if text.is_empty() {
        return None;
    }
    let (negative, body) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.as_str()),
    };
    let (radix, digits) =
        if let Some(rest) = body.strip_prefix("0x").or_else(|| body.strip_prefix("0X")) {
            (16, rest)
        } else if let Some(rest) = body.strip_prefix("0b").or_else(|| body.strip_prefix("0B")) {
            (2, rest)
        } else if let Some(rest) = body.strip_prefix("0o").or_else(|| body.strip_prefix("0O")) {
            (8, rest)
        } else {
            (10, body)
        };
    if digits.is_empty() {
        return None;
    }
    let value = u64::from_str_radix(digits, radix).ok()?;
    if negative {
        Some((value as i64).wrapping_neg() as u64)
    } else {
        Some(value)
    }
}

pub fn format_radix(value: u64, radix: u32) -> String {
    match radix {
        2 => format!("{value:b}"),
        8 => format!("{value:o}"),
        10 => value.to_string(),
        16 => format!("{value:X}"),
        _ => value.to_string(),
    }
}

pub fn grouped_binary(value: u64, width: u32) -> String {
    let bits = format_radix(value, 2);
    let padded = if bits.len() as u32 >= width {
        bits
    } else {
        format!("{}{}", "0".repeat(width as usize - bits.len()), bits)
    };
    padded
        .as_bytes()
        .rchunks(4)
        .rev()
        .map(|chunk| std::str::from_utf8(chunk).unwrap_or(""))
        .collect::<Vec<_>>()
        .join(" ")
}

pub struct BaseRow {
    pub label: &'static str,
    pub value: String,
}

pub fn base_rows(value: u64, width: u32) -> Vec<BaseRow> {
    let mask = if width >= 64 {
        u64::MAX
    } else {
        (1_u64 << width) - 1
    };
    let value = value & mask;
    vec![
        BaseRow {
            label: "HEX",
            value: format!("0x{}", format_radix(value, 16)),
        },
        BaseRow {
            label: "DEC",
            value: value.to_string(),
        },
        BaseRow {
            label: "OCT",
            value: format!("0o{}", format_radix(value, 8)),
        },
        BaseRow {
            label: "BIN",
            value: format!("0b{}", format_radix(value, 2)),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_common_bases() {
        assert_eq!(parse_integer("255"), Some(255));
        assert_eq!(parse_integer("0xFF"), Some(255));
        assert_eq!(parse_integer("0b1010"), Some(10));
        assert_eq!(parse_integer("0o17"), Some(15));
        assert_eq!(parse_integer("1_000"), Some(1_000));
        assert_eq!(parse_integer("-1"), Some(u64::MAX));
    }

    #[test]
    fn bitwise_operations_respect_width() {
        assert_eq!(apply(0xF0, 0x0F, BitOp::Or, 32), Some(0xFF));
        assert_eq!(apply(0xF0, 0x0F, BitOp::And, 32), Some(0));
        assert_eq!(apply(1, 40, BitOp::Shl, 32), Some(1 << 8));
        assert_eq!(apply(0, 0, BitOp::Not, 32), Some(0xFFFF_FFFF));
    }

    #[test]
    fn state_and_rows() {
        let mut state = ProgrammerState::new();
        state.input = "0xFF".to_string();
        state.operand = "0x0F".to_string();
        assert_eq!(state.value(), Some(255));
        assert_eq!(state.result(), Some(15));
        let rows = base_rows(255, 32);
        assert_eq!(rows[0].value, "0xFF");
        assert_eq!(rows[1].value, "255");
        assert_eq!(rows[3].value, "0b11111111");
        assert_eq!(grouped_binary(255, 16), "0000 0000 1111 1111");
    }
}
