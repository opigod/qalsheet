//! A small, allocation-light expression compiler and evaluator.
//!
//! The parser accepts the notation people actually type:
//!
//! * `+ - * / ^`, Unicode `× ÷ −`, parentheses
//! * implicit multiplication: `2π`, `2(3+4)`, `2sin(30)`, `(2)(3)`
//! * constants `π`, `e`, `τ`, `φ` and the variable `ans`
//! * trigonometry (degrees or radians), logs, roots, rounding and a few helpers
//! * postfix factorial and percent
//!
//! `compile` builds an AST. Graph sampling and calculus reuse that AST instead
//! of reparsing the text for every pixel.

use std::f64::consts::{E, PI, TAU};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AngleMode {
    Degrees,
    Radians,
}

impl AngleMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Degrees => "DEG",
            Self::Radians => "RAD",
        }
    }

    pub fn to_radians(self, value: f64) -> f64 {
        match self {
            Self::Degrees => value * PI / 180.0,
            Self::Radians => value,
        }
    }

    pub fn from_radians(self, value: f64) -> f64 {
        match self {
            Self::Degrees => value * 180.0 / PI,
            Self::Radians => value,
        }
    }

    pub fn toggled(self) -> Self {
        match self {
            Self::Degrees => Self::Radians,
            Self::Radians => Self::Degrees,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalError {
    pub message: String,
}

impl EvalError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for EvalError {}

pub type EvalResult<T = f64> = Result<T, EvalError>;

#[derive(Clone, Debug)]
pub struct Expression {
    root: Node,
    source: String,
}

impl Expression {
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Evaluate with named variables. `x` is the graph variable; `ans` is the
    /// last calculator result.
    pub fn eval(&self, angle: AngleMode, vars: &[(&str, f64)]) -> EvalResult {
        self.root.eval(angle, &|name| {
            vars.iter()
                .find(|(key, _)| *key == name)
                .map(|(_, value)| *value)
        })
    }

    /// Evaluate with a lookup closure, avoiding a temporary map in hot loops.
    pub fn eval_with(&self, angle: AngleMode, lookup: impl Fn(&str) -> Option<f64>) -> EvalResult {
        self.root.eval(angle, &lookup)
    }
}

/// Compiles an expression, repairing nothing. Use [`evaluable`] first for the
/// calculator's unfinished input.
pub fn compile(source: &str) -> EvalResult<Expression> {
    let normalized = normalize(source);
    if normalized.trim().is_empty() {
        return Err(EvalError::new("Enter an expression"));
    }
    let mut parser = Parser::new(&normalized);
    let root = parser.parse()?;
    Ok(Expression {
        root,
        source: normalized,
    })
}

/// Convenience for one-off evaluation.
pub fn evaluate(source: &str, angle: AngleMode, vars: &[(&str, f64)]) -> EvalResult {
    compile(source)?.eval(angle, vars)
}

/// Repairs the calculator's in-progress text: trims a dangling operator,
/// closes unmatched parentheses and rejects text that cannot become valid.
pub fn evaluable(raw: &str) -> Option<String> {
    let mut expression = raw.trim().to_string();
    if expression.is_empty() {
        return None;
    }

    while expression.chars().last().is_some_and(|ch| {
        matches!(
            ch,
            '+' | '\u{2212}' | '-' | '\u{00D7}' | '*' | '\u{00F7}' | '/' | '(' | '^'
        )
    }) {
        expression.pop();
        while expression.ends_with(' ') {
            expression.pop();
        }
    }
    if expression.is_empty() || expression == "." {
        return None;
    }

    let all_letters = expression.chars().all(|ch| ch.is_alphabetic());
    if all_letters {
        let lower = expression.to_lowercase();
        if !matches!(lower.as_str(), "e" | "pi" | "ans" | "tau" | "phi") {
            return None;
        }
    }

    let mut balance = 0i32;
    for ch in expression.chars() {
        match ch {
            '(' => balance += 1,
            ')' => {
                balance -= 1;
                if balance < 0 {
                    return None;
                }
            }
            _ => {}
        }
    }
    if balance > 0 {
        expression.push_str(&")".repeat(balance as usize));
    }
    Some(expression)
}

pub fn uses_trigonometry(expression: &str) -> bool {
    [
        "sin(", "cos(", "tan(", "asin(", "acos(", "atan(", "sinh(", "cosh(", "tanh(",
    ]
    .iter()
    .any(|needle| expression.contains(needle))
}

fn normalize(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\u{00D7}' | '\u{00B7}' | '\u{2217}' => out.push('*'),
            '\u{00F7}' => out.push('/'),
            '\u{2212}' | '\u{2013}' | '\u{2014}' => out.push('-'),
            '*' if chars.peek() == Some(&'*') => {
                chars.next();
                out.push('^');
            }
            _ => out.push(ch),
        }
    }
    out
}

#[derive(Clone, Debug)]
enum Node {
    Number(f64),
    Var(String),
    Neg(Box<Node>),
    Binary(BinOp, Box<Node>, Box<Node>),
    Call(Func, Vec<Node>),
    Factorial(Box<Node>),
    Percent(Box<Node>),
}

#[derive(Clone, Copy, Debug)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Func {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
    Asinh,
    Acosh,
    Atanh,
    Ln,
    Log,
    Log2,
    Exp,
    Sqrt,
    Cbrt,
    Abs,
    Floor,
    Ceil,
    Round,
    Sign,
    Min,
    Max,
    Mod,
}

impl Node {
    fn eval(&self, angle: AngleMode, lookup: &dyn Fn(&str) -> Option<f64>) -> EvalResult {
        let value = match self {
            Node::Number(value) => *value,
            Node::Var(name) => match name.as_str() {
                "pi" => PI,
                "e" => E,
                "tau" => TAU,
                "phi" => (1.0 + 5.0_f64.sqrt()) / 2.0,
                _ => lookup(name).ok_or_else(|| EvalError::new(format!("Unknown name {name}")))?,
            },
            Node::Neg(inner) => -inner.eval(angle, lookup)?,
            Node::Binary(op, left, right) => {
                let left = left.eval(angle, lookup)?;
                let right = right.eval(angle, lookup)?;
                match op {
                    BinOp::Add => left + right,
                    BinOp::Sub => left - right,
                    BinOp::Mul => left * right,
                    BinOp::Div => {
                        if right == 0.0 {
                            return Err(EvalError::new("Division by zero"));
                        }
                        left / right
                    }
                    BinOp::Pow => {
                        let result = left.powf(right);
                        if !result.is_finite() {
                            return Err(EvalError::new("Overflow"));
                        }
                        result
                    }
                }
            }
            Node::Call(func, arguments) => {
                let mut values = Vec::with_capacity(arguments.len());
                for argument in arguments {
                    values.push(argument.eval(angle, lookup)?);
                }
                call(*func, &values, angle)?
            }
            Node::Factorial(inner) => {
                let value = inner.eval(angle, lookup)?;
                factorial(value)?
            }
            Node::Percent(inner) => inner.eval(angle, lookup)? / 100.0,
        };
        if !value.is_finite() {
            return Err(EvalError::new("Overflow"));
        }
        Ok(value)
    }
}

fn call(func: Func, args: &[f64], angle: AngleMode) -> EvalResult {
    let one = |name: &str| {
        if args.len() == 1 {
            Ok(args[0])
        } else {
            Err(EvalError::new(format!("{name} needs one argument")))
        }
    };
    let clean_trig = |value: f64| if value.abs() < 1e-12 { 0.0 } else { value };
    match func {
        Func::Sin => Ok(clean_trig(angle.to_radians(one("sin")?).sin())),
        Func::Cos => Ok(clean_trig(angle.to_radians(one("cos")?).cos())),
        Func::Tan => {
            let radians = angle.to_radians(one("tan")?);
            if radians.cos().abs() < 1e-12 {
                return Err(EvalError::new("Tangent undefined"));
            }
            Ok(clean_trig(radians.tan()))
        }
        Func::Asin => {
            let value = one("asin")?;
            if !(-1.0..=1.0).contains(&value) {
                return Err(EvalError::new("asin domain"));
            }
            Ok(angle.from_radians(value.asin()))
        }
        Func::Acos => {
            let value = one("acos")?;
            if !(-1.0..=1.0).contains(&value) {
                return Err(EvalError::new("acos domain"));
            }
            Ok(angle.from_radians(value.acos()))
        }
        Func::Atan => Ok(angle.from_radians(one("atan")?.atan())),
        Func::Sinh => Ok(one("sinh")?.sinh()),
        Func::Cosh => Ok(one("cosh")?.cosh()),
        Func::Tanh => Ok(one("tanh")?.tanh()),
        Func::Asinh => Ok(one("asinh")?.asinh()),
        Func::Acosh => {
            let value = one("acosh")?;
            if value < 1.0 {
                return Err(EvalError::new("acosh domain"));
            }
            Ok(value.acosh())
        }
        Func::Atanh => {
            let value = one("atanh")?;
            if value.abs() >= 1.0 {
                return Err(EvalError::new("atanh domain"));
            }
            Ok(value.atanh())
        }
        Func::Ln => {
            let value = one("ln")?;
            if value <= 0.0 {
                return Err(EvalError::new("ln domain"));
            }
            Ok(value.ln())
        }
        Func::Log => {
            let value = one("log")?;
            if value <= 0.0 {
                return Err(EvalError::new("log domain"));
            }
            Ok(value.log10())
        }
        Func::Log2 => {
            let value = one("log2")?;
            if value <= 0.0 {
                return Err(EvalError::new("log2 domain"));
            }
            Ok(value.log2())
        }
        Func::Exp => Ok(one("exp")?.exp()),
        Func::Sqrt => {
            let value = one("sqrt")?;
            if value < 0.0 {
                return Err(EvalError::new("sqrt domain"));
            }
            Ok(value.sqrt())
        }
        Func::Cbrt => Ok(one("cbrt")?.cbrt()),
        Func::Abs => Ok(one("abs")?.abs()),
        Func::Floor => Ok(one("floor")?.floor()),
        Func::Ceil => Ok(one("ceil")?.ceil()),
        Func::Round => Ok(one("round")?.round()),
        Func::Sign => Ok(one("sign")?.signum()),
        Func::Min => {
            if args.is_empty() {
                return Err(EvalError::new("min needs an argument"));
            }
            Ok(args.iter().copied().fold(f64::INFINITY, f64::min))
        }
        Func::Max => {
            if args.is_empty() {
                return Err(EvalError::new("max needs an argument"));
            }
            Ok(args.iter().copied().fold(f64::NEG_INFINITY, f64::max))
        }
        Func::Mod => {
            if args.len() != 2 {
                return Err(EvalError::new("mod needs two arguments"));
            }
            if args[1] == 0.0 {
                return Err(EvalError::new("Division by zero"));
            }
            Ok(args[0].rem_euclid(args[1].abs()))
        }
    }
}

fn factorial(value: f64) -> EvalResult {
    if value < 0.0 || value.fract() != 0.0 || value > 170.0 {
        return Err(EvalError::new("Factorial domain"));
    }
    let limit = value as u32;
    let mut result = 1.0_f64;
    for index in 2..=limit {
        result *= index as f64;
    }
    if !result.is_finite() {
        return Err(EvalError::new("Factorial overflow"));
    }
    Ok(result)
}

struct Parser {
    source: Vec<char>,
    position: usize,
}

impl Parser {
    fn new(source: &str) -> Self {
        Self {
            source: source.chars().collect(),
            position: 0,
        }
    }

    fn parse(&mut self) -> EvalResult<Node> {
        let node = self.expression()?;
        self.skip_spaces();
        if self.position != self.source.len() {
            return Err(EvalError::new("Check the expression"));
        }
        Ok(node)
    }

    fn expression(&mut self) -> EvalResult<Node> {
        let mut node = self.term()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some('+') => {
                    self.position += 1;
                    node = Node::Binary(BinOp::Add, Box::new(node), Box::new(self.term()?));
                }
                Some('-') => {
                    self.position += 1;
                    node = Node::Binary(BinOp::Sub, Box::new(node), Box::new(self.term()?));
                }
                _ => return Ok(node),
            }
        }
    }

    fn term(&mut self) -> EvalResult<Node> {
        let mut node = self.factor()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some('*') => {
                    self.position += 1;
                    node = Node::Binary(BinOp::Mul, Box::new(node), Box::new(self.factor()?));
                }
                Some('/') => {
                    self.position += 1;
                    node = Node::Binary(BinOp::Div, Box::new(node), Box::new(self.factor()?));
                }
                Some(ch) if starts_implicit(ch) => {
                    node = Node::Binary(BinOp::Mul, Box::new(node), Box::new(self.factor()?));
                }
                _ => return Ok(node),
            }
        }
    }

    fn factor(&mut self) -> EvalResult<Node> {
        self.skip_spaces();
        match self.peek() {
            Some('+') => {
                self.position += 1;
                self.factor()
            }
            Some('-') => {
                self.position += 1;
                Ok(Node::Neg(Box::new(self.factor()?)))
            }
            _ => {
                let mut node = self.postfix()?;
                self.skip_spaces();
                if self.peek() == Some('^') {
                    self.position += 1;
                    node = Node::Binary(BinOp::Pow, Box::new(node), Box::new(self.factor()?));
                }
                Ok(node)
            }
        }
    }

    fn postfix(&mut self) -> EvalResult<Node> {
        let mut node = self.primary()?;
        loop {
            self.skip_spaces();
            match self.peek() {
                Some('!') => {
                    self.position += 1;
                    node = Node::Factorial(Box::new(node));
                }
                Some('%') => {
                    self.position += 1;
                    node = Node::Percent(Box::new(node));
                }
                _ => return Ok(node),
            }
        }
    }

    fn primary(&mut self) -> EvalResult<Node> {
        self.skip_spaces();
        match self.peek() {
            Some('(') => {
                self.position += 1;
                let node = self.expression()?;
                self.skip_spaces();
                if self.peek() != Some(')') {
                    return Err(EvalError::new("Missing )"));
                }
                self.position += 1;
                Ok(node)
            }
            Some('\u{03C0}') => {
                self.position += 1;
                Ok(Node::Var("pi".to_string()))
            }
            Some('\u{221A}') => {
                self.position += 1;
                Ok(Node::Call(Func::Sqrt, vec![self.primary()?]))
            }
            Some(ch) if ch.is_alphabetic() => self.named_value(),
            Some(ch) if ch.is_ascii_digit() || ch == '.' => self.number(),
            _ => Err(EvalError::new("Number expected")),
        }
    }

    fn number(&mut self) -> EvalResult<Node> {
        let start = self.position;
        let mut dots = 0;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.position += 1;
            } else if ch == '.' {
                dots += 1;
                self.position += 1;
            } else {
                break;
            }
        }
        if dots > 1 {
            return Err(EvalError::new("Check the number"));
        }
        let token: String = self.source[start..self.position].iter().collect();
        if token == "." {
            return Err(EvalError::new("Number expected"));
        }
        token
            .parse::<f64>()
            .map(Node::Number)
            .map_err(|_| EvalError::new("Check the number"))
    }

    fn named_value(&mut self) -> EvalResult<Node> {
        let start = self.position;
        while self.peek().is_some_and(|ch| ch.is_alphabetic()) {
            self.position += 1;
        }
        let mut name: String = self.source[start..self.position]
            .iter()
            .collect::<String>()
            .to_lowercase();

        // log10 and log2 are single names.
        if name == "log" {
            if self.peek() == Some('1') && self.peek_at(1) == Some('0') {
                self.position += 2;
                name = "log10".to_string();
            } else if self.peek() == Some('2') {
                self.position += 1;
                name = "log2".to_string();
            }
        }

        if matches!(name.as_str(), "pi" | "e" | "tau" | "phi") {
            return Ok(Node::Var(name));
        }

        self.skip_spaces();
        if self.peek() != Some('(') {
            if name == "ans" || name.chars().count() == 1 {
                return Ok(Node::Var(name));
            }
            return Err(EvalError::new("Function needs parentheses"));
        }

        self.position += 1;
        let mut arguments = Vec::new();
        self.skip_spaces();
        if self.peek() == Some(')') {
            self.position += 1;
        } else {
            loop {
                arguments.push(self.expression()?);
                self.skip_spaces();
                match self.peek() {
                    Some(',') => {
                        self.position += 1;
                    }
                    Some(')') => {
                        self.position += 1;
                        break;
                    }
                    _ => return Err(EvalError::new("Missing )")),
                }
            }
        }

        let function = match name.as_str() {
            "sin" => Func::Sin,
            "cos" => Func::Cos,
            "tan" => Func::Tan,
            "asin" => Func::Asin,
            "acos" => Func::Acos,
            "atan" => Func::Atan,
            "sinh" => Func::Sinh,
            "cosh" => Func::Cosh,
            "tanh" => Func::Tanh,
            "asinh" => Func::Asinh,
            "acosh" => Func::Acosh,
            "atanh" => Func::Atanh,
            "ln" => Func::Ln,
            "log" | "log10" => Func::Log,
            "log2" => Func::Log2,
            "exp" => Func::Exp,
            "sqrt" => Func::Sqrt,
            "cbrt" => Func::Cbrt,
            "abs" => Func::Abs,
            "floor" => Func::Floor,
            "ceil" => Func::Ceil,
            "round" => Func::Round,
            "sign" => Func::Sign,
            "min" => Func::Min,
            "max" => Func::Max,
            "mod" => Func::Mod,
            _ => return Err(EvalError::new(format!("Unknown function {name}"))),
        };
        Ok(Node::Call(function, arguments))
    }

    fn starts_implicit(ch: char) -> bool {
        ch.is_ascii_digit()
            || ch == '.'
            || ch == '('
            || ch == '\u{03C0}'
            || ch == '\u{221A}'
            || ch.is_alphabetic()
    }

    fn peek(&self) -> Option<char> {
        self.source.get(self.position).copied()
    }

    fn peek_at(&self, offset: usize) -> Option<char> {
        self.source.get(self.position + offset).copied()
    }

    fn skip_spaces(&mut self) {
        while self.peek().is_some_and(|ch| ch.is_whitespace()) {
            self.position += 1;
        }
    }
}

// `starts_implicit` is used through `Self::starts_implicit`; keep a free helper
// for readable pattern matching in `term`.
fn starts_implicit(ch: char) -> bool {
    Parser::starts_implicit(ch)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn value(expression: &str) -> f64 {
        let repaired = evaluable(expression).expect("not repair-able");
        evaluate(&repaired, AngleMode::Degrees, &[]).expect("not evaluable")
    }

    #[test]
    fn precedence_and_powers() {
        assert!((value("2+3×4") - 14.0).abs() < 1e-12);
        assert!((value("(2+3)×4") - 20.0).abs() < 1e-12);
        assert!((value("2^3^2") - 512.0).abs() < 1e-12);
        assert!((value("(2^3)^2") - 64.0).abs() < 1e-12);
        assert!((value("2^−2") - 0.25).abs() < 1e-12);
        assert!((value("−2^2") + 4.0).abs() < 1e-12);
        assert!((value("2^3!") - 64.0).abs() < 1e-12);
    }

    #[test]
    fn functions_domains_and_constants() {
        assert!((value("sin(30)") - 0.5).abs() < 1e-12);
        assert!((evaluate("sin(π÷2)", AngleMode::Radians, &[]).unwrap() - 1.0).abs() < 1e-9);
        assert!((value("ln(e)") - 1.0).abs() < 1e-12);
        assert!((value("log(100)") - 2.0).abs() < 1e-12);
        assert!((value("log2(8)") - 3.0).abs() < 1e-12);
        assert!((value("√16") - 4.0).abs() < 1e-12);
        assert!((value("sin(30)cos(60)") - 0.25).abs() < 1e-12);
        assert!((value("2tau") - 2.0 * TAU).abs() < 1e-12);
        assert!((value("2e") - 2.0 * E).abs() < 1e-12);
        assert!((value("5!") - 120.0).abs() < 1e-12);
        assert!((value("200+10%") - 200.1).abs() < 1e-9);
    }

    #[test]
    fn domain_errors() {
        for expression in [
            "1÷0", "asin(2)", "acos(-2)", "ln(0)", "log(-1)", "√(-4)", "tan(90)", "10^400", "3.5!",
            "(-2)!", "171!",
        ] {
            let repaired = evaluable(expression).expect(expression);
            assert!(
                evaluate(&repaired, AngleMode::Degrees, &[]).is_err(),
                "{expression} should fail"
            );
        }
        assert!(evaluable("sin(").is_none());
    }

    #[test]
    fn repairs_unfinished_expressions() {
        assert_eq!(evaluable("2+3×").as_deref(), Some("2+3"));
        assert_eq!(evaluable("(2+3").as_deref(), Some("(2+3)"));
        assert_eq!(evaluable("5×(").as_deref(), Some("5"));
        assert_eq!(evaluable("sin(30+").as_deref(), Some("sin(30)"));
        assert!(evaluable("2+)").is_none());
    }

    #[test]
    fn variables() {
        let expression = compile("x^2+2x+1").unwrap();
        let result = expression.eval(AngleMode::Radians, &[("x", 3.0)]).unwrap();
        assert!((result - 16.0).abs() < 1e-12);
        let result = expression
            .eval_with(AngleMode::Radians, |name| (name == "x").then_some(4.0))
            .unwrap();
        assert!((result - 25.0).abs() < 1e-12);
    }

    #[test]
    fn angle_modes() {
        assert!((value("asin(0.5)") - 30.0).abs() < 1e-9);
        let radians = evaluate("sin(π÷2)", AngleMode::Radians, &[]).unwrap();
        assert!((radians - 1.0).abs() < 1e-12);
    }

    #[test]
    fn helper_functions() {
        assert!((value("min(3,7)") - 3.0).abs() < 1e-12);
        assert!((value("max(3,7,5)") - 7.0).abs() < 1e-12);
        assert!((value("mod(7,3)") - 1.0).abs() < 1e-12);
        assert!((value("round(2.6)") - 3.0).abs() < 1e-12);
    }
}
