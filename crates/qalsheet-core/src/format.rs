//! Number formatting shared by every screen.
//!
//! Twelve significant digits are enough for a handheld calculator and keep
//! binary floating point noise (`0.1 + 0.2`) out of the display. Values beyond
//! 1e12 or below 1e-8 switch to scientific notation.

/// Formats a value for the display. `grouping` uses NBSP thousands separators.
pub fn number(value: f64, grouping: bool) -> String {
    if !value.is_finite() {
        return "Error".to_string();
    }
    let abs = value.abs();
    let body = if abs != 0.0 && (abs >= 1_000_000_000_000.0 || abs < 0.000_000_01) {
        scientific(value, 8)
    } else {
        plain(value, 12)
    };
    if grouping {
        group_thousands(&body)
    } else {
        body
    }
}

/// Same as [`number`] without grouping.
pub fn plain_number(value: f64) -> String {
    number(value, false)
}

/// Formats a value with grouping, used by the converter.
pub fn grouped_number(value: f64) -> String {
    number(value, true)
}

/// Six decimals, trimmed. Used for algebraic roots and symbolic coefficients.
pub fn solution_number(value: f64) -> String {
    if !value.is_finite() {
        return "Error".to_string();
    }
    let abs = value.abs();
    if abs >= 1_000_000_000.0 || (abs > 0.0 && abs < 1e-6) {
        return number(value, false);
    }
    let text = format!("{value:.6}");
    let text = trim_decimal(&text);
    let text = if text == "-0" { "0" } else { &text };
    text.replace('-', "\u{2212}")
}

/// A value that can be typed back into the parser. Scientific results become
/// `1.2×10^9` instead of an `e` notation the parser would reject.
pub fn as_expression(value: f64) -> String {
    let text = plain_number(value);
    if let Some((mantissa, exponent)) = text.split_once('e') {
        let exponent = exponent.replace('-', "\u{2212}");
        format!("{mantissa}\u{00D7}10^{exponent}")
    } else {
        text
    }
}

fn plain(value: f64, significant: i32) -> String {
    if value == 0.0 {
        return "0".to_string();
    }
    let abs = value.abs();
    let exponent = abs.log10().floor() as i32;
    let decimals = (significant - (exponent + 1)).clamp(0, 17) as usize;
    let text = format!("{value:.decimals$}");
    let text = trim_decimal(&text);
    let text = if text == "-0" { "0".to_string() } else { text };
    text.replace('-', "\u{2212}")
}

fn scientific(value: f64, significant: usize) -> String {
    let decimals = significant.saturating_sub(1);
    let text = format!("{value:.decimals$e}");
    let (mantissa, exponent) = text.split_once('e').unwrap_or((&text, "0"));
    let mantissa = trim_decimal(mantissa);
    let mantissa = if mantissa == "-0" {
        "0".to_string()
    } else {
        mantissa
    };
    let exponent = exponent.trim_start_matches('+');
    let exponent = if exponent == "-0" { "0" } else { exponent };
    format!("{mantissa}e{exponent}").replace('-', "\u{2212}")
}

fn trim_decimal(text: &str) -> String {
    if !text.contains('.') {
        return text.to_string();
    }
    let mut result = text.trim_end_matches('0').trim_end_matches('.').to_string();
    if result.is_empty() || result == "-" {
        result = "0".to_string();
    }
    result
}

fn group_thousands(text: &str) -> String {
    if text.contains('e') || text.contains('E') {
        return text.to_string();
    }
    let (negative, body) = match text.strip_prefix('\u{2212}') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    let (integer, fraction) = match body.split_once('.') {
        Some((i, f)) => (i, Some(f)),
        None => (body, None),
    };
    if integer.len() <= 3 {
        return text.to_string();
    }
    let mut grouped = String::with_capacity(integer.len() + integer.len() / 3);
    let first = integer.len() % 3;
    for (index, ch) in integer.chars().enumerate() {
        if index > 0 && (index == first || (index > first && (index - first) % 3 == 0)) {
            grouped.push('\u{00A0}');
        }
        grouped.push(ch);
    }
    let mut result = String::new();
    if negative {
        result.push('\u{2212}');
    }
    result.push_str(&grouped);
    if let Some(fraction) = fraction {
        result.push('.');
        result.push_str(fraction);
    }
    result
}

/// Compact graph-axis labels: no grouping, up to four significant digits.
pub fn axis_label(value: f64) -> String {
    if !value.is_finite() {
        return String::new();
    }
    if value == 0.0 {
        return "0".to_string();
    }
    let abs = value.abs();
    if abs >= 1_000_000.0 || abs < 0.0001 {
        let text = format!("{value:.2e}");
        let (mantissa, exponent) = text.split_once('e').unwrap_or((&text, "0"));
        let mantissa = trim_decimal(mantissa);
        let exponent = exponent.trim_start_matches('+');
        return format!("{mantissa}e{exponent}").replace('-', "\u{2212}");
    }
    let decimals = if abs >= 100.0 {
        0
    } else if abs >= 10.0 {
        1
    } else if abs >= 1.0 {
        2
    } else {
        3
    };
    let text = format!("{value:.decimals$}");
    trim_decimal(&text).replace('-', "\u{2212}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_like_the_watch() {
        assert_eq!(plain_number(0.0), "0");
        assert_eq!(plain_number(15.000_000_000_000_1), "15");
        assert_eq!(plain_number(0.1 + 0.2), "0.3");
        assert_eq!(plain_number(-12.5), "\u{2212}12.5");
        assert_eq!(grouped_number(1_234_567.0), "1\u{00A0}234\u{00A0}567");
        assert!(plain_number(0.000_000_001).contains("e\u{2212}"));
    }

    #[test]
    fn scientific_results_round_trip() {
        assert_eq!(as_expression(1e20), "1\u{00D7}10^20");
        assert_eq!(as_expression(1.5e-9), "1.5\u{00D7}10^\u{2212}9");
    }

    #[test]
    fn solution_rounding() {
        assert_eq!(solution_number(1.0 / 3.0), "0.333333");
        assert_eq!(solution_number(-2.5), "\u{2212}2.5");
        assert_eq!(solution_number(0.0), "0");
    }
}
