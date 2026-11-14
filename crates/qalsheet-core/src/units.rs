//! Unit conversion data and state.
//!
//! Every unit maps to its category's base with the affine transform
//! `base = (value + offset) * factor`. Temperature is the only affine case.

use std::f64::consts::PI;

use crate::format;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Unit {
    pub symbol: &'static str,
    pub name: &'static str,
    pub factor: f64,
    pub offset: f64,
}

impl Unit {
    const fn linear(symbol: &'static str, name: &'static str, factor: f64) -> Self {
        Self {
            symbol,
            name,
            factor,
            offset: 0.0,
        }
    }

    pub fn to_base(self, value: f64) -> f64 {
        (value + self.offset) * self.factor
    }

    pub fn from_base(self, value: f64) -> f64 {
        value / self.factor - self.offset
    }
}

#[derive(Clone, Copy, Debug)]
pub struct UnitCategory {
    pub label: &'static str,
    pub icon: &'static str,
    pub units: &'static [Unit],
    pub default_from: &'static str,
    pub default_to: &'static str,
}

impl UnitCategory {
    pub fn unit(&self, symbol: &str) -> Option<&'static Unit> {
        self.units.iter().find(|unit| unit.symbol == symbol)
    }

    pub fn index_of(&self, symbol: &str) -> usize {
        self.units
            .iter()
            .position(|unit| unit.symbol == symbol)
            .unwrap_or(0)
    }

    pub fn default_from_index(&self) -> usize {
        self.index_of(self.default_from)
    }

    pub fn default_to_index(&self) -> usize {
        self.index_of(self.default_to)
    }
}

static LENGTH: UnitCategory = UnitCategory {
    label: "Length",
    icon: "↔",
    units: &[
        Unit::linear("mm", "Millimeter", 0.001),
        Unit::linear("cm", "Centimeter", 0.01),
        Unit::linear("dm", "Decimeter", 0.1),
        Unit::linear("m", "Meter", 1.0),
        Unit::linear("km", "Kilometer", 1_000.0),
        Unit::linear("in", "Inch", 0.0254),
        Unit::linear("ft", "Foot", 0.3048),
        Unit::linear("yd", "Yard", 0.9144),
        Unit::linear("mi", "Mile", 1_609.344),
    ],
    default_from: "cm",
    default_to: "mm",
};

static MASS: UnitCategory = UnitCategory {
    label: "Mass",
    icon: "⚖",
    units: &[
        Unit::linear("mg", "Milligram", 0.001),
        Unit::linear("g", "Gram", 1.0),
        Unit::linear("kg", "Kilogram", 1_000.0),
        Unit::linear("t", "Tonne", 1_000_000.0),
        Unit::linear("oz", "Ounce", 28.349523125),
        Unit::linear("lb", "Pound", 453.59237),
        Unit::linear("st", "Stone", 6_350.29318),
    ],
    default_from: "kg",
    default_to: "g",
};

static TEMPERATURE: UnitCategory = UnitCategory {
    label: "Temp",
    icon: "🌡",
    units: &[
        Unit {
            symbol: "°C",
            name: "Celsius",
            factor: 1.0,
            offset: 0.0,
        },
        Unit {
            symbol: "°F",
            name: "Fahrenheit",
            factor: 5.0 / 9.0,
            offset: -32.0,
        },
        Unit {
            symbol: "K",
            name: "Kelvin",
            factor: 1.0,
            offset: -273.15,
        },
        Unit {
            symbol: "°R",
            name: "Rankine",
            factor: 5.0 / 9.0,
            offset: -491.67,
        },
    ],
    default_from: "°C",
    default_to: "°F",
};

static VOLUME: UnitCategory = UnitCategory {
    label: "Volume",
    icon: "◫",
    units: &[
        Unit::linear("ml", "Milliliter", 0.001),
        Unit::linear("cl", "Centiliter", 0.01),
        Unit::linear("dl", "Deciliter", 0.1),
        Unit::linear("l", "Liter", 1.0),
        Unit::linear("m³", "Cubic meter", 1_000.0),
        Unit::linear("tsp", "Teaspoon (US)", 0.00492892159375),
        Unit::linear("tbsp", "Tablespoon (US)", 0.01478676478125),
        Unit::linear("fl oz", "Fluid ounce (US)", 0.0295735295625),
        Unit::linear("cup", "Cup (US)", 0.2365882365),
        Unit::linear("pint", "Pint (US)", 0.473176473),
        Unit::linear("gal", "Gallon (US)", 3.785411784),
    ],
    default_from: "l",
    default_to: "ml",
};

static AREA: UnitCategory = UnitCategory {
    label: "Area",
    icon: "▦",
    units: &[
        Unit::linear("mm²", "Square millimeter", 0.000001),
        Unit::linear("cm²", "Square centimeter", 0.0001),
        Unit::linear("dm²", "Square decimeter", 0.01),
        Unit::linear("m²", "Square meter", 1.0),
        Unit::linear("km²", "Square kilometer", 1_000_000.0),
        Unit::linear("ha", "Hectare", 10_000.0),
        Unit::linear("in²", "Square inch", 0.00064516),
        Unit::linear("ft²", "Square foot", 0.09290304),
        Unit::linear("yd²", "Square yard", 0.83612736),
        Unit::linear("acre", "Acre", 4_046.8564224),
    ],
    default_from: "m²",
    default_to: "cm²",
};

static SPEED: UnitCategory = UnitCategory {
    label: "Speed",
    icon: "➜",
    units: &[
        Unit::linear("m/s", "Meter / second", 1.0),
        Unit::linear("km/h", "Kilometer / hour", 1.0 / 3.6),
        Unit::linear("mph", "Mile / hour", 0.44704),
        Unit::linear("kn", "Knot", 0.5144444444444445),
        Unit::linear("ft/s", "Foot / second", 0.3048),
    ],
    default_from: "km/h",
    default_to: "m/s",
};

static TIME: UnitCategory = UnitCategory {
    label: "Time",
    icon: "◷",
    units: &[
        Unit::linear("ms", "Millisecond", 0.001),
        Unit::linear("s", "Second", 1.0),
        Unit::linear("min", "Minute", 60.0),
        Unit::linear("h", "Hour", 3_600.0),
        Unit::linear("day", "Day", 86_400.0),
        Unit::linear("wk", "Week", 604_800.0),
        Unit::linear("yr", "Year (365.25 d)", 31_557_600.0),
    ],
    default_from: "h",
    default_to: "min",
};

static DATA: UnitCategory = UnitCategory {
    label: "Data",
    icon: "⛁",
    units: &[
        Unit::linear("bit", "Bit", 0.125),
        Unit::linear("B", "Byte", 1.0),
        Unit::linear("kB", "Kilobyte (1000 B)", 1_000.0),
        Unit::linear("MB", "Megabyte (1000² B)", 1_000_000.0),
        Unit::linear("GB", "Gigabyte (1000³ B)", 1_000_000_000.0),
        Unit::linear("TB", "Terabyte (1000⁴ B)", 1_000_000_000_000.0),
        Unit::linear("KiB", "Kibibyte (1024 B)", 1_024.0),
        Unit::linear("MiB", "Mebibyte (1024² B)", 1_048_576.0),
        Unit::linear("GiB", "Gibibyte (1024³ B)", 1_073_741_824.0),
        Unit::linear("TiB", "Tebibyte (1024⁴ B)", 1_099_511_627_776.0),
    ],
    default_from: "MB",
    default_to: "GB",
};

static ANGLE: UnitCategory = UnitCategory {
    label: "Angle",
    icon: "∠",
    units: &[
        Unit::linear("°", "Degree", PI / 180.0),
        Unit::linear("rad", "Radian", 1.0),
        Unit::linear("grad", "Gradian", PI / 200.0),
        Unit::linear("turn", "Turn", 2.0 * PI),
        Unit::linear("′", "Arcminute", PI / 10_800.0),
        Unit::linear("″", "Arcsecond", PI / 648_000.0),
    ],
    default_from: "°",
    default_to: "rad",
};

static PRESSURE: UnitCategory = UnitCategory {
    label: "Pressure",
    icon: "◎",
    units: &[
        Unit::linear("Pa", "Pascal", 1.0),
        Unit::linear("kPa", "Kilopascal", 1_000.0),
        Unit::linear("MPa", "Megapascal", 1_000_000.0),
        Unit::linear("bar", "Bar", 100_000.0),
        Unit::linear("atm", "Atmosphere", 101_325.0),
        Unit::linear("psi", "Pound / square inch", 6_894.757293168),
        Unit::linear("torr", "Torr", 133.322368421),
        Unit::linear("mmHg", "Millimeter of mercury", 133.322387415),
    ],
    default_from: "atm",
    default_to: "psi",
};

static ENERGY: UnitCategory = UnitCategory {
    label: "Energy",
    icon: "⚡",
    units: &[
        Unit::linear("J", "Joule", 1.0),
        Unit::linear("kJ", "Kilojoule", 1_000.0),
        Unit::linear("cal", "Calorie", 4.184),
        Unit::linear("kcal", "Kilocalorie", 4_184.0),
        Unit::linear("Wh", "Watt-hour", 3_600.0),
        Unit::linear("kWh", "Kilowatt-hour", 3_600_000.0),
        Unit::linear("eV", "Electronvolt", 1.602_176_634e-19),
        Unit::linear("BTU", "British thermal unit", 1_055.055_852_62),
    ],
    default_from: "kcal",
    default_to: "kJ",
};

pub static CATEGORIES: &[UnitCategory] = &[
    LENGTH,
    MASS,
    TEMPERATURE,
    VOLUME,
    AREA,
    SPEED,
    TIME,
    DATA,
    ANGLE,
    PRESSURE,
    ENERGY,
];

pub fn convert(value: f64, from: Unit, to: Unit) -> f64 {
    to.from_base(from.to_base(value))
}

pub fn format_value(value: f64) -> String {
    format::grouped_number(value)
}

pub fn format_reference(from: Unit, to: Unit) -> String {
    let amount = convert(1.0, from, to);
    format!("1 {} = {} {}", from.symbol, format_value(amount), to.symbol)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnitTarget {
    From,
    To,
}

#[derive(Clone, Debug)]
pub struct ConverterState {
    category_index: usize,
    from_index: usize,
    to_index: usize,
    input: String,
}

impl Default for ConverterState {
    fn default() -> Self {
        Self::new()
    }
}

impl ConverterState {
    pub fn new() -> Self {
        let category = &CATEGORIES[0];
        Self {
            category_index: 0,
            from_index: category.default_from_index(),
            to_index: category.default_to_index(),
            input: "1".to_string(),
        }
    }

    pub fn category_index(&self) -> usize {
        self.category_index
    }

    pub fn category(&self) -> &'static UnitCategory {
        &CATEGORIES[self.category_index]
    }

    pub fn from_index(&self) -> usize {
        self.from_index
    }

    pub fn to_index(&self) -> usize {
        self.to_index
    }

    pub fn from(&self) -> Unit {
        self.category().units[self.from_index.min(self.category().units.len() - 1)]
    }

    pub fn to(&self) -> Unit {
        self.category().units[self.to_index.min(self.category().units.len() - 1)]
    }

    pub fn input(&self) -> &str {
        &self.input
    }

    pub fn input_text(&self) -> String {
        if self.input.is_empty() {
            "0".to_string()
        } else {
            self.input.clone()
        }
    }

    pub fn amount(&self) -> f64 {
        self.input.replace('\u{2212}', "-").parse().unwrap_or(0.0)
    }

    pub fn result(&self) -> f64 {
        convert(self.amount(), self.from(), self.to())
    }

    pub fn result_text(&self) -> String {
        format_value(self.result())
    }

    pub fn select_category(&mut self, index: usize) {
        if index >= CATEGORIES.len() {
            return;
        }
        self.category_index = index;
        let category = &CATEGORIES[index];
        self.from_index = category.default_from_index();
        self.to_index = category.default_to_index();
    }

    pub fn select_unit(&mut self, target: UnitTarget, index: usize) {
        if index >= self.category().units.len() {
            return;
        }
        match target {
            UnitTarget::From => self.from_index = index,
            UnitTarget::To => self.to_index = index,
        }
    }

    pub fn swap(&mut self) {
        let converted = self.result();
        std::mem::swap(&mut self.from_index, &mut self.to_index);
        self.input = format::plain_number(converted).replace('\u{2212}', "-");
    }

    pub fn set_input(&mut self, input: &str) {
        self.input = input.chars().take(24).collect();
    }

    /// Used by the text field; the converter recomputes live, so no validation
    /// is required while typing.
    pub fn input_mut(&mut self) -> &mut String {
        &mut self.input
    }

    pub fn append_digit(&mut self, digit: char) {
        if !digit.is_ascii_digit() {
            return;
        }
        match self.input.as_str() {
            "0" => self.input = digit.to_string(),
            "\u{2212}" | "" => {
                self.input = if self.input.is_empty() {
                    digit.to_string()
                } else {
                    format!("\u{2212}{digit}")
                }
            }
            _ if self.input.len() < 24 => self.input.push(digit),
            _ => {}
        }
    }

    pub fn append_decimal(&mut self) {
        if self.input.contains('.') {
            return;
        }
        match self.input.as_str() {
            "" => self.input = "0.".to_string(),
            "\u{2212}" => self.input = "\u{2212}0.".to_string(),
            _ => self.input.push('.'),
        }
    }

    pub fn backspace(&mut self) {
        self.input.pop();
        if self.input.is_empty() || self.input == "\u{2212}" {
            self.input = "0".to_string();
        }
    }

    pub fn clear_input(&mut self) {
        self.input = "0".to_string();
    }

    pub fn toggle_sign(&mut self) {
        if let Some(rest) = self.input.strip_prefix('\u{2212}') {
            self.input = if rest.is_empty() {
                "0".to_string()
            } else {
                rest.to_string()
            };
        } else if self.input == "0" || self.input.is_empty() {
            self.input = "\u{2212}".to_string();
        } else {
            self.input.insert(0, '\u{2212}');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn category(label: &str) -> &'static UnitCategory {
        CATEGORIES
            .iter()
            .find(|category| category.label == label)
            .unwrap()
    }

    #[test]
    fn converts_common_units() {
        let length = category("Length");
        let cm = *length.unit("cm").unwrap();
        let mm = *length.unit("mm").unwrap();
        assert!((convert(12.5, cm, mm) - 125.0).abs() < 1e-9);

        let mass = category("Mass");
        let kg = *mass.unit("kg").unwrap();
        let g = *mass.unit("g").unwrap();
        assert!((convert(2.0, kg, g) - 2_000.0).abs() < 1e-9);

        let temperature = category("Temp");
        let celsius = *temperature.unit("°C").unwrap();
        let fahrenheit = *temperature.unit("°F").unwrap();
        let kelvin = *temperature.unit("K").unwrap();
        assert!((convert(0.0, celsius, fahrenheit) - 32.0).abs() < 1e-9);
        assert!((convert(100.0, celsius, fahrenheit) - 212.0).abs() < 1e-9);
        assert!((convert(-40.0, celsius, fahrenheit) + 40.0).abs() < 1e-9);
        assert!((convert(0.0, celsius, kelvin) - 273.15).abs() < 1e-9);

        let data = category("Data");
        assert!(
            (convert(1.0, *data.unit("GB").unwrap(), *data.unit("MB").unwrap()) - 1_000.0).abs()
                < 1e-9
        );
        assert!(
            (convert(1.0, *data.unit("GiB").unwrap(), *data.unit("MiB").unwrap()) - 1_024.0).abs()
                < 1e-9
        );
    }

    #[test]
    fn added_categories_work() {
        let angle = category("Angle");
        let degree = *angle.unit("°").unwrap();
        let radian = *angle.unit("rad").unwrap();
        assert!((convert(180.0, degree, radian) - PI).abs() < 1e-12);

        let pressure = category("Pressure");
        let atm = *pressure.unit("atm").unwrap();
        let psi = *pressure.unit("psi").unwrap();
        assert!((convert(1.0, atm, psi) - 14.695_948).abs() < 1e-4);

        let energy = category("Energy");
        let kcal = *energy.unit("kcal").unwrap();
        let kj = *energy.unit("kJ").unwrap();
        assert!((convert(1.0, kcal, kj) - 4.184).abs() < 1e-9);
    }

    #[test]
    fn defaults_are_valid() {
        for category in CATEGORIES {
            assert!(category.units.len() >= 2);
            assert!(category.unit(category.default_from).is_some());
            assert!(category.unit(category.default_to).is_some());
        }
    }

    #[test]
    fn state_swaps_and_keeps_relationship() {
        let mut state = ConverterState::new();
        assert_eq!(state.from().symbol, "cm");
        assert_eq!(state.to().symbol, "mm");
        assert!((state.result() - 10.0).abs() < 1e-9);
        state.swap();
        assert_eq!(state.from().symbol, "mm");
        assert_eq!(state.to().symbol, "cm");
        assert_eq!(state.input(), "10");
        assert!((state.result() - 1.0).abs() < 1e-9);
    }
}
