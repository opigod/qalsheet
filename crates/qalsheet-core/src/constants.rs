//! Physical constants and reference values.

#[derive(Clone, Copy, Debug)]
pub struct Constant {
    pub name: &'static str,
    pub symbol: &'static str,
    pub value: f64,
    pub unit: &'static str,
    pub note: &'static str,
}

pub static CONSTANTS: &[Constant] = &[
    Constant {
        name: "Pi",
        symbol: "π",
        value: std::f64::consts::PI,
        unit: "",
        note: "Circle circumference / diameter",
    },
    Constant {
        name: "Euler's number",
        symbol: "e",
        value: std::f64::consts::E,
        unit: "",
        note: "Base of the natural logarithm",
    },
    Constant {
        name: "Tau",
        symbol: "τ",
        value: std::f64::consts::TAU,
        unit: "",
        note: "Full turn in radians",
    },
    Constant {
        name: "Golden ratio",
        symbol: "φ",
        value: std::f64::consts::GOLDEN_RATIO,
        unit: "",
        note: "(1 + √5) / 2",
    },
    Constant {
        name: "Speed of light",
        symbol: "c",
        value: 299_792_458.0,
        unit: "m/s",
        note: "Exact in vacuum",
    },
    Constant {
        name: "Standard gravity",
        symbol: "g",
        value: 9.806_65,
        unit: "m/s²",
        note: "Mean sea level",
    },
    Constant {
        name: "Gravitational constant",
        symbol: "G",
        value: 6.674_30e-11,
        unit: "m³/(kg·s²)",
        note: "Newtonian constant",
    },
    Constant {
        name: "Planck constant",
        symbol: "h",
        value: 6.626_070_15e-34,
        unit: "J·s",
        note: "Quantum of action",
    },
    Constant {
        name: "Reduced Planck",
        symbol: "ħ",
        value: 1.054_571_817e-34,
        unit: "J·s",
        note: "h / 2π",
    },
    Constant {
        name: "Boltzmann constant",
        symbol: "k_B",
        value: 1.380_649e-23,
        unit: "J/K",
        note: "Entropy scale",
    },
    Constant {
        name: "Avogadro constant",
        symbol: "N_A",
        value: 6.022_140_76e23,
        unit: "1/mol",
        note: "Particles per mole",
    },
    Constant {
        name: "Gas constant",
        symbol: "R",
        value: 8.314_462_618,
        unit: "J/(mol·K)",
        note: "N_A · k_B",
    },
    Constant {
        name: "Elementary charge",
        symbol: "e⁻",
        value: 1.602_176_634e-19,
        unit: "C",
        note: "Charge of an electron",
    },
    Constant {
        name: "Electron mass",
        symbol: "mₑ",
        value: 9.109_383_701_5e-31,
        unit: "kg",
        note: "Rest mass",
    },
    Constant {
        name: "Proton mass",
        symbol: "mₚ",
        value: 1.672_621_923_69e-27,
        unit: "kg",
        note: "Rest mass",
    },
    Constant {
        name: "Neutron mass",
        symbol: "mₙ",
        value: 1.674_927_500_56e-27,
        unit: "kg",
        note: "Rest mass",
    },
    Constant {
        name: "Vacuum permittivity",
        symbol: "ε₀",
        value: 8.854_187_812_8e-12,
        unit: "F/m",
        note: "Electric constant",
    },
    Constant {
        name: "Vacuum permeability",
        symbol: "μ₀",
        value: 1.256_637_062_12e-6,
        unit: "H/m",
        note: "Magnetic constant",
    },
    Constant {
        name: "Standard atmosphere",
        symbol: "atm",
        value: 101_325.0,
        unit: "Pa",
        note: "Defined pressure",
    },
    Constant {
        name: "Earth radius",
        symbol: "R⊕",
        value: 6_371_000.0,
        unit: "m",
        note: "Mean radius",
    },
    Constant {
        name: "Earth mass",
        symbol: "M⊕",
        value: 5.972_2e24,
        unit: "kg",
        note: "Approximate",
    },
    Constant {
        name: "Astronomical unit",
        symbol: "au",
        value: 149_597_870_700.0,
        unit: "m",
        note: "Earth–Sun distance",
    },
    Constant {
        name: "Light year",
        symbol: "ly",
        value: 9.460_730_472_580_8e15,
        unit: "m",
        note: "Distance light travels in a year",
    },
];

pub fn search(query: &str) -> Vec<&'static Constant> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return CONSTANTS.iter().collect();
    }
    CONSTANTS
        .iter()
        .filter(|constant| {
            constant.name.to_lowercase().contains(&query)
                || constant.symbol.to_lowercase().contains(&query)
                || constant.unit.to_lowercase().contains(&query)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constants_are_sane() {
        assert!(CONSTANTS.len() > 15);
        let pi = CONSTANTS
            .iter()
            .find(|constant| constant.symbol == "π")
            .unwrap();
        assert!((pi.value - std::f64::consts::PI).abs() < 1e-15);
        assert!(search("planck").len() >= 2);
        assert!(!search("").is_empty());
    }
}
