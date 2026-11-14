//! Multivariate polynomial and rational-equation solver.
//!
//! The parser expands integer powers while parsing, so `(x+1)^2` becomes
//! `x² + 2x + 1`. Linear equations with several letters are solved
//! symbolically for the selected letter. One-letter equations are solved
//! exactly up to degree two and numerically (Durand–Kerner) up to degree 12.

use std::collections::{BTreeMap, BTreeSet};

use crate::format;

const EPS: f64 = 1e-10;
const MAX_POWER: i32 = 12;

#[derive(Clone, Debug, PartialEq)]
pub enum AlgebraResult {
    Solved { variable: char, value: f64 },
    SolvedRoots { variable: char, values: Vec<f64> },
    Expression { variable: char, expression: String },
    NoSolution,
    NoRealSolution,
    InfiniteSolutions,
    Error(String),
}

impl AlgebraResult {
    pub fn is_solution(&self) -> bool {
        matches!(
            self,
            Self::Solved { .. } | Self::SolvedRoots { .. } | Self::Expression { .. }
        )
    }

    pub fn is_warning(&self) -> bool {
        matches!(
            self,
            Self::NoSolution | Self::NoRealSolution | Self::InfiniteSolutions
        )
    }

    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error(_))
    }

    pub fn text(&self) -> String {
        match self {
            Self::Solved { variable, value } => {
                format!("{variable} = {}", format::solution_number(*value))
            }
            Self::SolvedRoots { variable, values } => format!(
                "{variable} = {}",
                values
                    .iter()
                    .map(|value| format::solution_number(*value))
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            Self::Expression {
                variable,
                expression,
            } => format!("{variable} = {expression}"),
            Self::NoSolution => "No solution".to_string(),
            Self::NoRealSolution => "No real solution".to_string(),
            Self::InfiniteSolutions => "Infinite solutions".to_string(),
            Self::Error(message) => message.clone(),
        }
    }
}

pub fn solve(raw: &str, variable: char) -> AlgebraResult {
    let equation = normalize_superscripts(raw.trim());
    if equation.is_empty() {
        return AlgebraResult::Error("Enter an equation".to_string());
    }

    let parts: Vec<&str> = equation.split('=').collect();
    if parts.len() == 1 {
        return AlgebraResult::Error("Add an = sign".to_string());
    }
    if parts.len() > 2 {
        return AlgebraResult::Error("Use one = sign".to_string());
    }
    if parts[0].trim().is_empty() || parts[1].trim().is_empty() {
        return AlgebraResult::Error("Complete both sides".to_string());
    }

    let left = match AlgebraParser::new(parts[0]).parse() {
        Ok(value) => value,
        Err(error) => return AlgebraResult::Error(error),
    };
    let right = match AlgebraParser::new(parts[1]).parse() {
        Ok(value) => value,
        Err(error) => return AlgebraResult::Error(error),
    };
    solve_difference(&(left - right), variable)
}

fn solve_difference(difference: &Rational, selected: char) -> AlgebraResult {
    let numerator = &difference.numerator;
    if numerator.is_zero() {
        return AlgebraResult::InfiniteSolutions;
    }
    if numerator.is_constant() {
        return AlgebraResult::NoSolution;
    }

    let letters = numerator.letters();
    if letters.len() > 1 {
        if numerator.degree() > 1 {
            return AlgebraResult::Error("Use one letter with powers".to_string());
        }
        return solve_linear_with_letters(difference, selected);
    }

    let letter = *letters.iter().next().unwrap();
    let coefficients = trim(&numerator.coefficients_for(letter));
    let degree = coefficients.len() as i32 - 1;
    match degree {
        0 => AlgebraResult::NoSolution,
        1 => {
            let root = -coefficients[0] / coefficients[1];
            if !denominator_allows(
                &difference.denominator,
                &difference.constraints,
                letter,
                root,
            ) {
                AlgebraResult::NoSolution
            } else {
                AlgebraResult::Solved {
                    variable: letter,
                    value: clean(root),
                }
            }
        }
        2 => match quadratic_roots(&coefficients) {
            None => AlgebraResult::NoRealSolution,
            Some(roots) => finish_roots(
                &roots,
                letter,
                &difference.denominator,
                &difference.constraints,
                false,
            ),
        },
        3..=MAX_POWER => {
            let roots = real_roots(&coefficients);
            if roots.is_empty() {
                return AlgebraResult::NoRealSolution;
            }
            finish_roots(
                &roots,
                letter,
                &difference.denominator,
                &difference.constraints,
                true,
            )
        }
        _ => AlgebraResult::Error("Power too high".to_string()),
    }
}

fn solve_linear_with_letters(difference: &Rational, selected: char) -> AlgebraResult {
    let numerator = &difference.numerator;
    let constant = clean(numerator.constant_value());
    let mut terms: BTreeMap<char, f64> = BTreeMap::new();
    for (monomial, coefficient) in &numerator.terms {
        if monomial.chars().count() == 1 {
            let letter = monomial.chars().next().unwrap();
            let value = clean(*coefficient);
            if value.abs() > EPS {
                terms.insert(letter, value);
            }
        }
    }

    if terms.is_empty() {
        return if constant.abs() < EPS {
            AlgebraResult::InfiniteSolutions
        } else {
            AlgebraResult::NoSolution
        };
    }

    let solve_for = if terms.contains_key(&selected) {
        selected
    } else if terms.len() == 1 {
        *terms.keys().next().unwrap()
    } else {
        return AlgebraResult::Error("Choose a letter in the equation".to_string());
    };
    let coefficient = terms[&solve_for];
    let solved_constant = -constant / coefficient;
    let mut others = BTreeMap::new();
    for (letter, value) in terms.iter().filter(|(letter, _)| **letter != solve_for) {
        let scaled = -value / coefficient;
        if scaled.abs() > EPS {
            others.insert(*letter, scaled);
        }
    }

    if others.is_empty() {
        if !denominator_allows(
            &difference.denominator,
            &difference.constraints,
            solve_for,
            solved_constant,
        ) {
            return AlgebraResult::NoSolution;
        }
        return AlgebraResult::Solved {
            variable: solve_for,
            value: clean(solved_constant),
        };
    }

    AlgebraResult::Expression {
        variable: solve_for,
        expression: format_expression(solved_constant, &others),
    }
}

fn finish_roots(
    roots: &[f64],
    letter: char,
    denominator: &Poly,
    constraints: &[Poly],
    group_nearby: bool,
) -> AlgebraResult {
    let valid: Vec<f64> = roots
        .iter()
        .copied()
        .filter(|root| denominator_allows(denominator, constraints, letter, *root))
        .collect();
    if valid.is_empty() {
        return AlgebraResult::NoSolution;
    }
    let values = if group_nearby {
        distinct_roots(&valid)
    } else {
        let mut sorted = valid;
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mut unique = Vec::new();
        for value in sorted {
            let text = format::solution_number(value);
            if unique
                .iter()
                .all(|existing| format::solution_number(*existing) != text)
            {
                unique.push(value);
            }
        }
        unique
    };
    if values.len() == 1 {
        AlgebraResult::Solved {
            variable: letter,
            value: values[0],
        }
    } else {
        AlgebraResult::SolvedRoots {
            variable: letter,
            values,
        }
    }
}

fn quadratic_roots(coefficients: &[f64]) -> Option<Vec<f64>> {
    let a = coefficients[2];
    let b = coefficients[1];
    let c = coefficients[0];
    let discriminant = b * b - 4.0 * a * c;
    if discriminant < -EPS {
        return None;
    }
    if discriminant.abs() <= EPS {
        return Some(vec![clean(-b / (2.0 * a))]);
    }
    let root = discriminant.sqrt();
    let mut values = vec![
        clean((-b - root) / (2.0 * a)),
        clean((-b + root) / (2.0 * a)),
    ];
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Some(values)
}

fn real_roots(coefficients: &[f64]) -> Vec<f64> {
    let degree = coefficients.len() as i32 - 1;
    let leading = coefficients[coefficients.len() - 1];
    if leading.abs() < EPS {
        return Vec::new();
    }
    let monic: Vec<f64> = coefficients.iter().map(|value| value / leading).collect();

    let mut roots: Vec<Complex> = (0..degree as usize)
        .map(|index| {
            let mut seed = Complex::one();
            for _ in 0..index {
                seed *= Complex::new(0.4, 0.9);
            }
            seed
        })
        .collect();

    for _ in 0..240 {
        let mut largest_delta = 0.0_f64;
        for i in 0..roots.len() {
            let mut denominator = Complex::one();
            for j in 0..roots.len() {
                if i != j {
                    denominator *= roots[i] - roots[j];
                }
            }
            if denominator.abs() < 1e-12 {
                denominator = Complex::new(1e-12, 0.0);
            }
            let delta = evaluate_complex(&monic, roots[i]) / denominator;
            roots[i] -= delta;
            largest_delta = largest_delta.max(delta.abs());
        }
        if largest_delta < 1e-12 {
            break;
        }
    }

    let mut result: Vec<f64> = roots
        .iter()
        .filter(|root| root.im.abs() <= 1e-5 * 1.0_f64.max(root.re.abs()))
        .map(|root| clean(root.re))
        .collect();
    result.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    result
}

fn evaluate_complex(coefficients: &[f64], value: Complex) -> Complex {
    let mut result = Complex::new(*coefficients.last().unwrap(), 0.0);
    for coefficient in coefficients.iter().rev().skip(1) {
        result = result * value + Complex::new(*coefficient, 0.0);
    }
    result
}

fn trim(coefficients: &[f64]) -> Vec<f64> {
    let mut degree = coefficients.len() - 1;
    while degree > 0 && coefficients[degree].abs() < EPS {
        degree -= 1;
    }
    coefficients[..=degree].to_vec()
}

fn distinct_roots(values: &[f64]) -> Vec<f64> {
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mut groups: Vec<Vec<f64>> = Vec::new();
    for value in sorted {
        let close = groups.last().is_some_and(|group| {
            let average = group.iter().sum::<f64>() / group.len() as f64;
            (average - value).abs() <= 1e-4 * 1.0_f64.max(value.abs())
        });
        if close {
            groups.last_mut().unwrap().push(value);
        } else {
            groups.push(vec![value]);
        }
    }
    groups
        .into_iter()
        .map(|group| {
            let average = group.iter().sum::<f64>() / group.len() as f64;
            if average.abs() < 1e-6 {
                0.0
            } else {
                clean(average)
            }
        })
        .collect()
}

fn normalize_superscripts(text: &str) -> String {
    if !text.chars().any(|ch| superscript_digit(ch).is_some()) {
        return text.to_string();
    }
    let mut result = String::with_capacity(text.len() + 8);
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if superscript_digit(ch).is_none() {
            result.push(ch);
            continue;
        }
        if !result.ends_with('^') {
            result.push('^');
        }
        result.push(superscript_digit(ch).unwrap());
        while let Some(next) = chars.peek() {
            if let Some(digit) = superscript_digit(*next) {
                result.push(digit);
                chars.next();
            } else {
                break;
            }
        }
    }
    result
}

fn superscript_digit(ch: char) -> Option<char> {
    Some(match ch {
        '⁰' => '0',
        '¹' => '1',
        '²' => '2',
        '³' => '3',
        '⁴' => '4',
        '⁵' => '5',
        '⁶' => '6',
        '⁷' => '7',
        '⁸' => '8',
        '⁹' => '9',
        _ => return None,
    })
}

pub fn format_expression(constant: f64, terms: &BTreeMap<char, f64>) -> String {
    let mut pieces: Vec<String> = Vec::new();
    if constant.abs() > EPS {
        pieces.push(if constant < 0.0 {
            format!("\u{2212}{}", format::solution_number(constant.abs()))
        } else {
            format::solution_number(constant)
        });
    }
    for (letter, coefficient) in terms {
        if coefficient.abs() <= EPS {
            continue;
        }
        let magnitude = coefficient.abs();
        let mut body = String::new();
        if (magnitude - 1.0).abs() > EPS {
            body.push_str(&format::solution_number(magnitude));
        }
        body.push(*letter);
        if pieces.is_empty() {
            pieces.push(if *coefficient < 0.0 {
                format!("\u{2212}{body}")
            } else {
                body
            });
        } else if *coefficient < 0.0 {
            pieces.push(format!("\u{2212} {body}"));
        } else {
            pieces.push(format!("+ {body}"));
        }
    }
    if pieces.is_empty() {
        "0".to_string()
    } else {
        pieces.join(" ")
    }
}

fn clean(value: f64) -> f64 {
    if value.abs() < EPS { 0.0 } else { value }
}

fn denominator_allows(
    denominator: &Poly,
    constraints: &[Poly],
    variable: char,
    value: f64,
) -> bool {
    constraints
        .iter()
        .chain(std::iter::once(denominator))
        .all(|constraint| {
            if constraint.is_zero() {
                return false;
            }
            if constraint.is_constant() {
                return constraint.constant_value().abs() > EPS;
            }
            let letters = constraint.letters();
            if !letters.iter().all(|letter| *letter == variable) {
                return true;
            }
            let value = constraint.evaluate(&BTreeMap::from([(variable, value)]));
            value.abs() > 1e-7
        })
}

#[derive(Clone, Debug, Default)]
struct Poly {
    terms: BTreeMap<String, f64>,
}

impl Poly {
    fn zero() -> Self {
        Self::default()
    }

    fn one() -> Self {
        Self::constant(1.0)
    }

    fn constant(value: f64) -> Self {
        if value.abs() < EPS {
            Self::zero()
        } else {
            Self {
                terms: BTreeMap::from([(String::new(), value)]),
            }
        }
    }

    fn variable(letter: char) -> Self {
        Self {
            terms: BTreeMap::from([(letter.to_string(), 1.0)]),
        }
    }

    fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    fn is_constant(&self) -> bool {
        self.terms.keys().all(|monomial| monomial.is_empty())
    }

    fn degree(&self) -> i32 {
        self.terms
            .keys()
            .map(|monomial| monomial.chars().count() as i32)
            .max()
            .unwrap_or(0)
    }

    fn constant_value(&self) -> f64 {
        *self.terms.get("").unwrap_or(&0.0)
    }

    fn letters(&self) -> BTreeSet<char> {
        self.terms
            .keys()
            .flat_map(|monomial| monomial.chars())
            .collect()
    }

    fn coefficients_for(&self, letter: char) -> Vec<f64> {
        let mut coefficients = vec![0.0; self.degree() as usize + 1];
        for (monomial, coefficient) in &self.terms {
            let mut power = 0;
            let mut valid = true;
            for ch in monomial.chars() {
                if ch != letter {
                    valid = false;
                    break;
                }
                power += 1;
            }
            if !valid {
                return Vec::new();
            }
            coefficients[power] += coefficient;
        }
        coefficients
    }

    fn pow(&self, exponent: i32) -> Self {
        let mut result = Self::one();
        let mut base = self.clone();
        let mut remaining = exponent;
        while remaining > 0 {
            if remaining & 1 == 1 {
                result = &result * &base;
            }
            remaining >>= 1;
            if remaining > 0 {
                base = &base * &base;
            }
        }
        result
    }

    fn scale(&self, factor: f64) -> Self {
        if factor == 0.0 || self.is_zero() {
            return Self::zero();
        }
        Self {
            terms: self
                .terms
                .iter()
                .map(|(monomial, coefficient)| (monomial.clone(), coefficient * factor))
                .filter(|(_, coefficient)| coefficient.abs() > EPS)
                .collect(),
        }
    }

    fn evaluate(&self, values: &BTreeMap<char, f64>) -> f64 {
        self.terms
            .iter()
            .map(|(monomial, coefficient)| {
                let mut result = *coefficient;
                for letter in monomial.chars() {
                    result *= values.get(&letter).copied().unwrap_or(0.0);
                }
                result
            })
            .sum()
    }
}

impl std::ops::Add for &Poly {
    type Output = Poly;

    fn add(self, other: &Poly) -> Poly {
        if other.is_zero() {
            return self.clone();
        }
        if self.is_zero() {
            return other.clone();
        }
        let mut merged = self.terms.clone();
        for (monomial, coefficient) in &other.terms {
            *merged.entry(monomial.clone()).or_insert(0.0) += coefficient;
        }
        Poly {
            terms: merged
                .into_iter()
                .filter(|(_, coefficient)| coefficient.abs() > EPS)
                .collect(),
        }
    }
}

impl std::ops::Sub for Rational {
    type Output = Rational;

    fn sub(self, other: Rational) -> Rational {
        self.subtract(&other)
    }
}

impl std::ops::Sub for &Poly {
    type Output = Poly;

    fn sub(self, other: &Poly) -> Poly {
        self + &other.scale(-1.0)
    }
}

impl std::ops::Mul for &Poly {
    type Output = Poly;

    fn mul(self, other: &Poly) -> Poly {
        if self.is_zero() || other.is_zero() {
            return Poly::zero();
        }
        let mut product: BTreeMap<String, f64> = BTreeMap::new();
        for (left, left_coefficient) in &self.terms {
            for (right, right_coefficient) in &other.terms {
                let mut chars: Vec<char> = (left.clone() + right).chars().collect();
                chars.sort_unstable();
                let monomial: String = chars.into_iter().collect();
                *product.entry(monomial).or_insert(0.0) += left_coefficient * right_coefficient;
            }
        }
        Poly {
            terms: product
                .into_iter()
                .filter(|(_, coefficient)| coefficient.abs() > EPS)
                .collect(),
        }
    }
}

#[derive(Clone, Debug)]
struct Rational {
    numerator: Poly,
    denominator: Poly,
    constraints: Vec<Poly>,
}

impl Rational {
    fn of(poly: Poly) -> Self {
        Self {
            numerator: poly,
            denominator: Poly::one(),
            constraints: Vec::new(),
        }
    }

    fn constant(value: f64) -> Self {
        Self::of(Poly::constant(value))
    }

    fn add(&self, other: &Self) -> Self {
        Self {
            numerator: &(&self.numerator * &other.denominator)
                + &(&other.numerator * &self.denominator),
            denominator: &self.denominator * &other.denominator,
            constraints: self
                .constraints
                .iter()
                .chain(other.constraints.iter())
                .cloned()
                .chain([self.denominator.clone(), other.denominator.clone()])
                .collect(),
        }
    }

    fn negate(&self) -> Self {
        Self {
            numerator: self.numerator.scale(-1.0),
            denominator: self.denominator.clone(),
            constraints: self.constraints.clone(),
        }
    }

    fn subtract(&self, other: &Self) -> Self {
        self.add(&other.negate())
    }

    fn mul(&self, other: &Self) -> Self {
        Self {
            numerator: &self.numerator * &other.numerator,
            denominator: &self.denominator * &other.denominator,
            constraints: self
                .constraints
                .iter()
                .chain(other.constraints.iter())
                .cloned()
                .chain([self.denominator.clone(), other.denominator.clone()])
                .collect(),
        }
    }

    fn div(&self, other: &Self) -> Result<Self, String> {
        if other.numerator.is_zero() {
            return Err("Division by zero".to_string());
        }
        Ok(Self {
            numerator: &self.numerator * &other.denominator,
            denominator: &self.denominator * &other.numerator,
            constraints: self
                .constraints
                .iter()
                .chain(other.constraints.iter())
                .cloned()
                .chain([
                    self.denominator.clone(),
                    other.denominator.clone(),
                    other.numerator.clone(),
                ])
                .collect(),
        })
    }

    fn pow(&self, exponent: i32) -> Self {
        Self {
            numerator: self.numerator.pow(exponent),
            denominator: self.denominator.pow(exponent),
            constraints: self.constraints.clone(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Complex {
    re: f64,
    im: f64,
}

impl Complex {
    fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }

    fn one() -> Self {
        Self::new(1.0, 0.0)
    }

    fn abs(self) -> f64 {
        self.re.hypot(self.im)
    }
}

impl std::ops::Add for Complex {
    type Output = Complex;
    fn add(self, other: Complex) -> Complex {
        Complex::new(self.re + other.re, self.im + other.im)
    }
}

impl std::ops::Sub for Complex {
    type Output = Complex;
    fn sub(self, other: Complex) -> Complex {
        Complex::new(self.re - other.re, self.im - other.im)
    }
}

impl std::ops::SubAssign for Complex {
    fn sub_assign(&mut self, other: Complex) {
        *self = *self - other;
    }
}

impl std::ops::MulAssign for Complex {
    fn mul_assign(&mut self, other: Complex) {
        *self = *self * other;
    }
}

impl std::ops::Mul for Complex {
    type Output = Complex;
    fn mul(self, other: Complex) -> Complex {
        Complex::new(
            self.re * other.re - self.im * other.im,
            self.re * other.im + self.im * other.re,
        )
    }
}

impl std::ops::Div for Complex {
    type Output = Complex;
    fn div(self, other: Complex) -> Complex {
        let denominator = other.re * other.re + other.im * other.im;
        Complex::new(
            (self.re * other.re + self.im * other.im) / denominator,
            (self.im * other.re - self.re * other.im) / denominator,
        )
    }
}

struct AlgebraParser {
    source: Vec<char>,
    position: usize,
}

impl AlgebraParser {
    fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
        }
    }

    fn parse(&mut self) -> Result<Rational, String> {
        let value = self.expression()?;
        self.skip_spaces();
        if self.position != self.source.len() {
            return Err("Check the expression".to_string());
        }
        Ok(value)
    }

    fn expression(&mut self) -> Result<Rational, String> {
        let mut value = self.term()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some('+') => {
                    self.position += 1;
                    value = value.add(&self.term()?);
                }
                Some('-') | Some('\u{2212}') => {
                    self.position += 1;
                    value = value.subtract(&self.term()?);
                }
                _ => return Ok(value),
            }
        }
    }

    fn term(&mut self) -> Result<Rational, String> {
        let mut value = self.factor()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some('*') | Some('\u{00D7}') => {
                    self.position += 1;
                    value = value.mul(&self.factor()?);
                }
                Some('/') | Some('\u{00F7}') => {
                    self.position += 1;
                    value = value.div(&self.factor()?)?;
                }
                Some(ch) if starts_implicit(ch) => {
                    value = value.mul(&self.factor()?);
                }
                _ => return Ok(value),
            }
        }
    }

    fn factor(&mut self) -> Result<Rational, String> {
        self.skip_spaces();
        match self.peek() {
            Some('+') => {
                self.position += 1;
                self.factor()
            }
            Some('-') | Some('\u{2212}') => {
                self.position += 1;
                Ok(self.factor()?.negate())
            }
            _ => self.power(),
        }
    }

    fn power(&mut self) -> Result<Rational, String> {
        let mut value = self.primary()?;
        loop {
            self.skip_spaces();
            if self.peek() != Some('^') {
                return Ok(value);
            }
            self.position += 1;
            let exponent = self.factor()?;
            value = value.pow(whole_exponent(&exponent)?);
        }
    }

    fn primary(&mut self) -> Result<Rational, String> {
        self.skip_spaces();
        let open = self.peek();
        if matches!(open, Some('(') | Some('[') | Some('{')) {
            let close = match open.unwrap() {
                '(' => ')',
                '[' => ']',
                _ => '}',
            };
            self.position += 1;
            let value = self.expression()?;
            self.skip_spaces();
            if self.peek() != Some(close) {
                return Err(format!("Missing {close}"));
            }
            self.position += 1;
            return Ok(value);
        }
        if let Some(ch) = self.peek() {
            if ch.is_alphabetic() {
                self.position += 1;
                return Ok(Rational::of(Poly::variable(
                    ch.to_lowercase().next().unwrap(),
                )));
            }
        }

        let start = self.position;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() || ch == '.' || ch == ',' {
                self.position += 1;
            } else {
                break;
            }
        }
        if start == self.position {
            return Err("Check the expression".to_string());
        }
        let token: String = self.source[start..self.position].iter().collect();
        if token.chars().filter(|ch| *ch == '.' || *ch == ',').count() > 1 {
            return Err("Check the expression".to_string());
        }
        let number: f64 = token
            .replace(',', ".")
            .parse()
            .map_err(|_| "Check the expression".to_string())?;
        Ok(Rational::constant(number))
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }

    fn skip_spaces(&mut self) {
        while self.peek().is_some_and(|ch| ch.is_whitespace()) {
            self.position += 1;
        }
    }
}

fn starts_implicit(ch: char) -> bool {
    ch.is_ascii_digit() || ch == '.' || ch.is_alphabetic() || matches!(ch, '(' | '[' | '{')
}

fn whole_exponent(value: &Rational) -> Result<i32, String> {
    if !value.numerator.is_constant() || !value.denominator.is_constant() {
        return Err("Use whole-number powers".to_string());
    }
    let denominator = value.denominator.constant_value();
    if denominator.abs() < EPS {
        return Err("Division by zero".to_string());
    }
    let number = value.numerator.constant_value() / denominator;
    if !number.is_finite() {
        return Err("Check the power".to_string());
    }
    let rounded = number.round();
    if (number - rounded).abs() > 1e-9 {
        return Err("Use whole-number powers".to_string());
    }
    if rounded < 0.0 {
        return Err("Use positive powers".to_string());
    }
    if rounded > MAX_POWER as f64 {
        return Err("Power too high".to_string());
    }
    Ok(rounded as i32)
}

const OPEN_GROUPS: [char; 3] = ['(', '[', '{'];

fn matching_close(open: char) -> char {
    match open {
        '(' => ')',
        '[' => ']',
        _ => '}',
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct AlgebraHistoryEntry {
    pub id: u64,
    pub equation: String,
    pub result: Option<AlgebraResult>,
    pub variable: char,
    pub note: String,
}

pub struct AlgebraState {
    pub equation: String,
    pub result: Option<AlgebraResult>,
    pub variable: char,
    pub history: Vec<AlgebraHistoryEntry>,
    next_history_id: u64,
}

impl Default for AlgebraState {
    fn default() -> Self {
        Self::new()
    }
}

impl AlgebraState {
    pub fn new() -> Self {
        Self {
            equation: String::new(),
            result: None,
            variable: 'x',
            history: Vec::new(),
            next_history_id: 1,
        }
    }

    pub fn display_text(&self) -> String {
        match &self.result {
            Some(result) => result.text(),
            None => self.equation.clone().if_empty("0"),
        }
    }

    pub fn hint_text(&self) -> String {
        match &self.result {
            Some(AlgebraResult::Error(message)) => message.clone(),
            Some(result) if result.is_solution() || result.is_warning() => self.equation.clone(),
            Some(_) => self.equation.clone(),
            None => {
                if self.equation.is_empty() {
                    "Type an equation".to_string()
                } else if self.equation.contains('=') {
                    "Tap Solve".to_string()
                } else {
                    "Use = then Solve".to_string()
                }
            }
        }
    }

    pub fn is_error(&self) -> bool {
        self.result.as_ref().is_some_and(AlgebraResult::is_error)
    }

    pub fn is_solution(&self) -> bool {
        self.result.as_ref().is_some_and(AlgebraResult::is_solution)
    }

    pub fn is_warning(&self) -> bool {
        self.result.as_ref().is_some_and(AlgebraResult::is_warning)
    }

    pub fn next_open_group(&self) -> char {
        OPEN_GROUPS[self.unmatched_open_groups().len() % OPEN_GROUPS.len()]
    }

    pub fn next_close_group(&self) -> char {
        self.unmatched_open_groups()
            .last()
            .copied()
            .map(matching_close)
            .unwrap_or(')')
    }

    pub fn set_equation(&mut self, equation: &str) {
        self.equation = equation.chars().take(120).collect();
        self.result = None;
    }

    pub fn press(&mut self, key: &str) {
        self.result = None;
        match key {
            "C" | "clear" => self.clear(),
            "\u{232B}" | "\u{8}" | "backspace" => self.backspace(),
            "=" => {
                if self.equation.contains('=') {
                    self.solve();
                } else {
                    self.input_equals();
                }
            }
            "+" | "\u{2212}" | "-" | "\u{00D7}" | "*" | "\u{00F7}" | "/" => {
                self.input_operator(key)
            }
            "." | "," => self.input_decimal(),
            "(" | ")" | "[" | "]" | "{" | "}" => self.input_paren(key),
            "^" => self.insert_power(),
            other if other.chars().count() == 1 => {
                let ch = other.chars().next().unwrap();
                if ch.is_alphabetic() {
                    self.insert_variable();
                } else {
                    self.input_digit(ch);
                }
            }
            _ => {}
        }
    }

    pub fn insert_power(&mut self) {
        self.result = None;
        if self.equation.len() >= 120 {
            return;
        }
        if let Some(last) = self.equation.chars().last() {
            if last.is_ascii_digit() || last.is_alphabetic() || matches!(last, ')' | ']' | '}') {
                self.equation.push('^');
            }
        }
    }

    pub fn insert_variable(&mut self) {
        self.result = None;
        if self.equation.len() >= 120 {
            return;
        }
        if self.equation.ends_with('.') {
            return;
        }
        self.equation.push(self.variable);
    }

    pub fn select_variable(&mut self, letter: char) {
        if letter.is_alphabetic() {
            self.variable = letter.to_lowercase().next().unwrap_or('x');
            self.result = None;
        }
    }

    pub fn solve(&mut self) {
        let solved = solve(&self.equation, self.variable);
        match &solved {
            AlgebraResult::Solved { variable, .. }
            | AlgebraResult::SolvedRoots { variable, .. }
            | AlgebraResult::Expression { variable, .. } => self.variable = *variable,
            _ => {}
        }
        self.result = Some(solved.clone());
        if self.equation.trim().is_empty() {
            return;
        }
        let duplicate = self.history.first().is_some_and(|entry| {
            entry.equation == self.equation
                && entry.variable == self.variable
                && entry.result.as_ref() == Some(&solved)
        });
        if !duplicate {
            let id = self.next_history_id;
            self.next_history_id += 1;
            self.history.insert(
                0,
                AlgebraHistoryEntry {
                    id,
                    equation: self.equation.clone(),
                    result: Some(solved),
                    variable: self.variable,
                    note: String::new(),
                },
            );
            if self.history.len() > 60 {
                self.history.pop();
            }
        }
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    pub fn restore_history(&mut self, entries: Vec<(u64, char, String, String)>) {
        self.history = entries
            .into_iter()
            .map(|(id, variable, equation, note)| AlgebraHistoryEntry {
                id,
                result: Some(solve(&equation, variable)),
                equation,
                variable,
                note,
            })
            .collect();
        self.next_history_id = self
            .history
            .iter()
            .map(|entry| entry.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
    }

    pub fn load_history(&mut self, entry: &AlgebraHistoryEntry) {
        self.equation = entry.equation.clone();
        self.variable = entry.variable;
        self.result = entry.result.clone();
    }

    pub fn history_note(&self, id: u64) -> Option<&str> {
        self.history
            .iter()
            .find(|entry| entry.id == id)
            .map(|entry| entry.note.as_str())
    }

    pub fn set_history_note(&mut self, id: u64, note: impl Into<String>) {
        if let Some(entry) = self.history.iter_mut().find(|entry| entry.id == id) {
            entry.note = note.into();
        }
    }

    pub fn clear(&mut self) {
        self.equation.clear();
        self.result = None;
    }

    fn backspace(&mut self) {
        self.equation.pop();
        self.result = None;
    }

    fn input_digit(&mut self, ch: char) {
        if !ch.is_ascii_digit() || self.equation.len() >= 120 {
            return;
        }
        let current = self.current_number();
        if current == "0" && ch != '0' {
            self.equation.pop();
            self.equation.push(ch);
        } else if current == "0" && ch == '0' {
            // Keep one leading zero.
        } else {
            self.equation.push(ch);
        }
    }

    fn input_decimal(&mut self) {
        if self.equation.len() >= 120 {
            return;
        }
        if let Some(last) = self.equation.chars().last() {
            if !last.is_ascii_digit()
                && last != '.'
                && !matches!(
                    last,
                    '+' | '\u{2212}'
                        | '-'
                        | '\u{00D7}'
                        | '*'
                        | '\u{00F7}'
                        | '/'
                        | '('
                        | '='
                        | '['
                        | '{'
                )
            {
                return;
            }
        }
        let current = self.current_number();
        if current.contains('.') {
            return;
        }
        if current.is_empty() {
            self.equation.push_str("0.");
        } else {
            self.equation.push('.');
        }
    }

    fn input_operator(&mut self, operator: &str) {
        let operator = match operator {
            "*" | "\u{00D7}" => "\u{00D7}",
            "/" | "\u{00F7}" => "\u{00F7}",
            "-" | "\u{2212}" => "\u{2212}",
            "+" => "+",
            _ => return,
        };
        if self.equation.is_empty() {
            if operator == "\u{2212}" {
                self.equation.push('\u{2212}');
            }
            return;
        }
        let last = self.equation.chars().last().unwrap();
        if last == '=' {
            if operator == "\u{2212}" {
                self.equation.push('\u{2212}');
            }
            return;
        }
        if matches!(last, '(' | '[' | '{' | '^') {
            if operator == "\u{2212}" && last != '^' {
                self.equation.push('\u{2212}');
            }
            return;
        }
        if self.equation.len() >= 120 {
            return;
        }
        match last {
            '+' | '\u{00D7}' | '\u{00F7}' => {
                self.equation.pop();
                self.equation.push_str(operator);
            }
            '\u{2212}' | '-' => {
                let previous = self.equation.chars().rev().nth(1);
                if self.equation.chars().count() == 1
                    || previous.is_some_and(|ch| {
                        matches!(
                            ch,
                            '+' | '\u{2212}'
                                | '-'
                                | '\u{00D7}'
                                | '\u{00F7}'
                                | '('
                                | '='
                                | '['
                                | '{'
                        )
                    })
                {
                    return;
                }
                self.equation.pop();
                self.equation.push_str(operator);
            }
            '.' => {}
            _ => self.equation.push_str(operator),
        }
    }

    fn input_equals(&mut self) {
        if self.equation.len() >= 120 || self.equation.contains('=') {
            return;
        }
        let Some(last) = self.equation.chars().last() else {
            return;
        };
        if matches!(
            last,
            '+' | '\u{2212}'
                | '-'
                | '\u{00D7}'
                | '*'
                | '\u{00F7}'
                | '/'
                | '('
                | '['
                | '{'
                | '.'
                | '^'
        ) {
            return;
        }
        self.equation.push('=');
    }

    fn input_paren(&mut self, paren: &str) {
        if self.equation.len() >= 120 {
            return;
        }
        if matches!(paren, "(" | "[" | "{") {
            let opener = self.next_open_group();
            let last = self.equation.chars().last();
            if last.is_some_and(|ch| {
                ch.is_ascii_digit() || ch.is_alphabetic() || matches!(ch, ')' | ']' | '}')
            }) {
                self.equation.push('\u{00D7}');
                self.equation.push(opener);
            } else {
                self.equation.push(opener);
            }
            return;
        }

        let openers = self.unmatched_open_groups();
        let Some(open) = openers.last() else {
            return;
        };
        let last = self.equation.chars().last();
        if last.is_some_and(|ch| {
            ch.is_ascii_digit() || ch.is_alphabetic() || matches!(ch, ')' | ']' | '}')
        }) {
            self.equation.push(matching_close(*open));
        }
    }

    fn unmatched_open_groups(&self) -> Vec<char> {
        let mut openers = Vec::new();
        for ch in self.equation.chars() {
            match ch {
                '(' | '[' | '{' => openers.push(ch),
                ')' | ']' | '}' => {
                    openers.pop();
                }
                _ => {}
            }
        }
        openers
    }

    fn current_number(&self) -> String {
        let start = self
            .equation
            .char_indices()
            .rev()
            .find(|(_, ch)| {
                matches!(
                    ch,
                    '=' | '+'
                        | '\u{2212}'
                        | '-'
                        | '\u{00D7}'
                        | '*'
                        | '\u{00F7}'
                        | '/'
                        | '('
                        | '['
                        | '{'
                )
            })
            .map(|(index, ch)| index + ch.len_utf8())
            .unwrap_or(0);
        self.equation[start..].to_string()
    }
}

trait IfEmpty {
    fn if_empty(&self, fallback: &str) -> String;
}

impl IfEmpty for String {
    fn if_empty(&self, fallback: &str) -> String {
        if self.is_empty() {
            fallback.to_string()
        } else {
            self.clone()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solved(equation: &str, variable: char) -> AlgebraResult {
        solve(equation, variable)
    }

    #[test]
    fn linear_equations() {
        assert_eq!(
            solved("3x + 5 = 20", 'x'),
            AlgebraResult::Solved {
                variable: 'x',
                value: 5.0
            }
        );
        assert_eq!(
            solved("2(x-3)=4x", 'x'),
            AlgebraResult::Solved {
                variable: 'x',
                value: -3.0
            }
        );
        assert_eq!(
            solved("5/x=2/3", 'x'),
            AlgebraResult::Solved {
                variable: 'x',
                value: 7.5
            }
        );
    }

    #[test]
    fn other_letters_and_symbolic_results() {
        assert_eq!(
            solved("3t+5=20", 'x'),
            AlgebraResult::Solved {
                variable: 't',
                value: 5.0
            }
        );
        assert_eq!(
            solved("2x+3y=12", 'x'),
            AlgebraResult::Expression {
                variable: 'x',
                expression: "6 \u{2212} 1.5y".to_string()
            }
        );
        assert_eq!(
            solved("2x+3y=12", 'y'),
            AlgebraResult::Expression {
                variable: 'y',
                expression: "4 \u{2212} 0.666667x".to_string()
            }
        );
    }

    #[test]
    fn special_and_malformed_equations() {
        assert_eq!(solved("x=x", 'x'), AlgebraResult::InfiniteSolutions);
        assert_eq!(solved("5=5", 'x'), AlgebraResult::InfiniteSolutions);
        assert_eq!(solved("x+1=x+2", 'x'), AlgebraResult::NoSolution);
        assert_eq!(
            solved("2x+3y=12", 'q'),
            AlgebraResult::Error("Choose a letter in the equation".to_string())
        );
        assert_eq!(
            solved("3x+5", 'x'),
            AlgebraResult::Error("Add an = sign".to_string())
        );
        assert_eq!(
            solved("3x+5=20=30", 'x'),
            AlgebraResult::Error("Use one = sign".to_string())
        );
    }

    #[test]
    fn brackets_and_textbook_cases() {
        assert_eq!(
            solved("4[y\u{2212}3(2y\u{2212}1)]=2[y\u{2212}2(3y+2)]", 'y'),
            AlgebraResult::Solved {
                variable: 'y',
                value: 2.0
            }
        );
        assert_eq!(
            solved("3[(z+2)\u{2212}4]\u{2212}z+1=2z\u{2212}5", 'x'),
            AlgebraResult::InfiniteSolutions
        );
        assert_eq!(
            solved("2[3x+1=14", 'x'),
            AlgebraResult::Error("Missing ]".to_string())
        );
        assert_eq!(
            solved("(2x\u{2212}1)/5=3+(1\u{2212}3x)/4", 'x'),
            AlgebraResult::Solved {
                variable: 'x',
                value: 3.0
            }
        );
    }

    #[test]
    fn quadratics_and_higher_roots() {
        assert_eq!(
            solved("x^2=4", 'x'),
            AlgebraResult::SolvedRoots {
                variable: 'x',
                values: vec![-2.0, 2.0]
            }
        );
        assert_eq!(solved("x^2+1=0", 'x'), AlgebraResult::NoRealSolution);
        assert_eq!(
            solved("x^3\u{2212}6x^2+11x\u{2212}6=0", 'x'),
            AlgebraResult::SolvedRoots {
                variable: 'x',
                values: vec![1.0, 2.0, 3.0]
            }
        );
        assert_eq!(
            solved("x^12=1", 'x'),
            AlgebraResult::SolvedRoots {
                variable: 'x',
                values: vec![-1.0, 1.0]
            }
        );
    }

    #[test]
    fn fractional_constraints_are_respected() {
        assert_eq!(solved("(x-1)/(x-1)=2", 'x'), AlgebraResult::NoSolution);
        assert_eq!(solved("5/x=0", 'x'), AlgebraResult::NoSolution);
        assert_eq!(
            solved("1/(2/x)=1", 'x'),
            AlgebraResult::Solved {
                variable: 'x',
                value: 2.0
            }
        );
    }

    #[test]
    fn powers_are_validated() {
        assert_eq!(
            solved("x^y=2", 'x'),
            AlgebraResult::Error("Use whole-number powers".to_string())
        );
        assert_eq!(
            solved("x^0.5=2", 'x'),
            AlgebraResult::Error("Use whole-number powers".to_string())
        );
        assert_eq!(
            solved("x^-1=2", 'x'),
            AlgebraResult::Error("Use positive powers".to_string())
        );
        assert_eq!(
            solved("x^13=1", 'x'),
            AlgebraResult::Error("Power too high".to_string())
        );
    }

    #[test]
    fn decimal_commas() {
        assert_eq!(
            solved("x+7,25=9,75", 'x'),
            AlgebraResult::Solved {
                variable: 'x',
                value: 2.5
            }
        );
        assert_eq!(
            solved("1,2.3x=4", 'x'),
            AlgebraResult::Error("Check the expression".to_string())
        );
    }

    #[test]
    fn state_nests_groups_and_solves() {
        let mut state = AlgebraState::new();
        for ch in "4[x\u{2212}3(2x\u{2212}1)]=2[x\u{2212}2(3x+2)]".chars() {
            state.press(&ch.to_string());
        }
        state.solve();
        assert_eq!(state.display_text(), "x = 2");
    }

    #[test]
    fn history_and_notes_survive_reload() {
        let mut state = AlgebraState::new();
        state.set_equation("x^2=4");
        state.solve();
        let entry = state.history[0].clone();
        assert_eq!(entry.equation, "x^2=4");
        assert_eq!(
            entry.result,
            Some(AlgebraResult::SolvedRoots {
                variable: 'x',
                values: vec![-2.0, 2.0],
            })
        );
        state.set_history_note(entry.id, "check both roots");
        assert_eq!(state.history_note(entry.id), Some("check both roots"));
        state.clear();
        state.load_history(&entry);
        assert_eq!(state.equation, "x^2=4");
        assert_eq!(state.history_note(entry.id), Some("check both roots"));
    }

    #[test]
    fn formatting() {
        assert_eq!(format::solution_number(1.0 / 3.0), "0.333333");
        assert_eq!(
            format_expression(6.0, &BTreeMap::from([('y', -1.5)])),
            "6 \u{2212} 1.5y"
        );
        assert_eq!(
            format_expression(10.0, &BTreeMap::from([('y', -1.0)])),
            "10 \u{2212} y"
        );
    }
}
