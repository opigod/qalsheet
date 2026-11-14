//! Calculator state and key semantics.
//!
//! This is the same model the Wear app used, extended with memory, `ans`, a
//! larger history and more scientific functions. Keeping the state pure makes
//! it trivial to test and lets every frontend drive it with its own keypad.

use crate::expr::{self, AngleMode};
use crate::format;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FunctionKey {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Sinh,
    Cosh,
    Tanh,
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
    Square,
    Power,
    Reciprocal,
    Factorial,
    Percent,
    Sign,
    Pi,
    E,
    Tau,
    Phi,
    Ans,
    OpenParen,
    CloseParen,
}

impl FunctionKey {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Sinh => "sinh",
            Self::Cosh => "cosh",
            Self::Tanh => "tanh",
            Self::Ln => "ln",
            Self::Log => "log",
            Self::Log2 => "log2",
            Self::Exp => "exp",
            Self::Sqrt => "√",
            Self::Cbrt => "cbrt",
            Self::Abs => "|x|",
            Self::Floor => "floor",
            Self::Ceil => "ceil",
            Self::Round => "round",
            Self::Square => "x²",
            Self::Power => "x^y",
            Self::Reciprocal => "1/x",
            Self::Factorial => "!",
            Self::Percent => "%",
            Self::Sign => "±",
            Self::Pi => "π",
            Self::E => "e",
            Self::Tau => "τ",
            Self::Phi => "φ",
            Self::Ans => "Ans",
            Self::OpenParen => "(",
            Self::CloseParen => ")",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Sin => "Sine",
            Self::Cos => "Cosine",
            Self::Tan => "Tangent",
            Self::Asin => "Arcsine",
            Self::Acos => "Arccosine",
            Self::Atan => "Arctangent",
            Self::Sinh => "Hyperbolic sine",
            Self::Cosh => "Hyperbolic cosine",
            Self::Tanh => "Hyperbolic tangent",
            Self::Ln => "Natural logarithm",
            Self::Log => "Logarithm base 10",
            Self::Log2 => "Logarithm base 2",
            Self::Exp => "e raised to the power",
            Self::Sqrt => "Square root",
            Self::Cbrt => "Cube root",
            Self::Abs => "Absolute value",
            Self::Floor => "Round down",
            Self::Ceil => "Round up",
            Self::Round => "Round",
            Self::Square => "Square",
            Self::Power => "Power",
            Self::Reciprocal => "Reciprocal",
            Self::Factorial => "Factorial",
            Self::Percent => "Percent",
            Self::Sign => "Toggle sign",
            Self::Pi => "Pi",
            Self::E => "Euler's number",
            Self::Tau => "Tau",
            Self::Phi => "Golden ratio",
            Self::Ans => "Previous answer",
            Self::OpenParen => "Open parenthesis",
            Self::CloseParen => "Close parenthesis",
        }
    }

    pub fn function_name(self) -> Option<&'static str> {
        Some(match self {
            Self::Sin => "sin",
            Self::Cos => "cos",
            Self::Tan => "tan",
            Self::Asin => "asin",
            Self::Acos => "acos",
            Self::Atan => "atan",
            Self::Sinh => "sinh",
            Self::Cosh => "cosh",
            Self::Tanh => "tanh",
            Self::Ln => "ln",
            Self::Log => "log",
            Self::Log2 => "log2",
            Self::Exp => "exp",
            Self::Sqrt => "\u{221A}",
            Self::Cbrt => "cbrt",
            Self::Abs => "abs",
            Self::Floor => "floor",
            Self::Ceil => "ceil",
            Self::Round => "round",
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct HistoryEntry {
    pub id: u64,
    pub expression: String,
    pub result: String,
    pub angle_mode: Option<AngleMode>,
    pub value: Option<f64>,
    pub note: String,
}

#[derive(Clone, Debug)]
pub struct CalculatorState {
    expression: String,
    result: Option<String>,
    just_evaluated: bool,
    pub angle_mode: AngleMode,
    pub show_scientific: bool,
    pub history: Vec<HistoryEntry>,
    next_history_id: u64,
    last_value: Option<f64>,
    ans: Option<f64>,
    memory: Option<f64>,
}

impl Default for CalculatorState {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorState {
    pub fn new() -> Self {
        Self {
            expression: String::new(),
            result: None,
            just_evaluated: false,
            angle_mode: AngleMode::Degrees,
            show_scientific: false,
            history: Vec::new(),
            next_history_id: 1,
            last_value: None,
            ans: None,
            memory: None,
        }
    }

    pub fn expression(&self) -> &str {
        &self.expression
    }

    pub fn result(&self) -> Option<&str> {
        self.result.as_deref()
    }

    pub fn is_error(&self) -> bool {
        self.result.as_deref() == Some("Error")
    }

    pub fn display_text(&self) -> String {
        self.result
            .clone()
            .unwrap_or_else(|| self.expression.clone().if_empty("0"))
    }

    pub fn top_text(&self) -> String {
        if self.just_evaluated {
            self.expression.clone()
        } else if self.expression.is_empty() {
            "qalsheet".to_string()
        } else if let Some(preview) = self.preview() {
            format!("= {preview}")
        } else {
            String::new()
        }
    }

    pub fn top_is_preview(&self) -> bool {
        !self.just_evaluated && !self.expression.is_empty() && self.preview().is_some()
    }

    pub fn preview(&self) -> Option<String> {
        if self.just_evaluated || self.expression.is_empty() {
            return None;
        }
        if !self.expression.chars().any(|ch| {
            matches!(
                ch,
                '+' | '\u{2212}'
                    | '-'
                    | '\u{00D7}'
                    | '*'
                    | '\u{00F7}'
                    | '/'
                    | '('
                    | ')'
                    | '^'
                    | '!'
                    | '%'
            )
        }) {
            return None;
        }
        let evaluable = expr::evaluable(&self.expression)?;
        let value = self.evaluate(&evaluable)?;
        Some(format::plain_number(value))
    }

    pub fn ans(&self) -> Option<f64> {
        self.ans
    }

    pub fn memory(&self) -> Option<f64> {
        self.memory
    }

    pub fn memory_active(&self) -> bool {
        self.memory.is_some()
    }

    pub fn toggle_angle_mode(&mut self) {
        self.angle_mode = self.angle_mode.toggled();
    }

    pub fn set_angle_mode(&mut self, mode: AngleMode) {
        self.angle_mode = mode;
    }

    pub fn show_scientific_page(&mut self, show: bool) {
        self.show_scientific = show;
    }

    pub fn clear(&mut self) {
        self.expression.clear();
        self.result = None;
        self.just_evaluated = false;
        self.last_value = None;
    }

    pub fn clear_history(&mut self) {
        self.history.clear();
    }

    /// Handles the basic keypad plus direct keyboard characters.
    pub fn press(&mut self, key: &str) {
        match key {
            "C" | "clear" => self.clear(),
            "\u{232B}" | "backspace" | "\u{8}" => self.backspace(),
            "=" | "enter" => self.calculate(),
            "+" | "-" | "\u{2212}" | "\u{00D7}" | "\u{00F7}" | "*" | "/" => {
                self.input_operator(key)
            }
            "^" => self.input_operator("^"),
            "(" => self.input_open_paren(),
            ")" => self.input_close_paren(),
            "." | "," => self.input_decimal(),
            "%" => self.apply_function(FunctionKey::Percent),
            "!" => self.apply_function(FunctionKey::Factorial),
            other if other.chars().count() == 1 && other.chars().all(|ch| ch.is_ascii_digit()) => {
                self.input_digit(other)
            }
            _ => {}
        }
    }

    /// Inserts a run of digits/operators typed on a physical keyboard.
    pub fn type_text(&mut self, text: &str) {
        for ch in text.chars() {
            match ch {
                '0'..='9' | '.' => self.press(&ch.to_string()),
                '+' | '-' | '*' | '/' | '^' | '(' | ')' | '%' | '!' => self.press(&ch.to_string()),
                '\u{00D7}' | '\u{00F7}' | '\u{2212}' => self.press(&ch.to_string()),
                _ => {}
            }
        }
    }

    pub fn apply_function(&mut self, key: FunctionKey) {
        if self.just_evaluated {
            if self.is_error() {
                self.clear();
            } else if !self.continue_from_result() {
                self.clear();
            }
        }
        self.close_incomplete_parens();

        if let Some(name) = key.function_name() {
            self.insert_function(name);
            self.show_scientific = false;
            return;
        }

        match key {
            FunctionKey::Square => {
                if self.has_operand() {
                    self.append_raw("^2");
                }
            }
            FunctionKey::Power => {
                if self.has_operand() {
                    self.append_raw("^");
                }
            }
            FunctionKey::Reciprocal => {
                if self.has_operand() {
                    self.wrap_last_operand("1/(", ")");
                } else {
                    self.append_raw("1/(");
                }
            }
            FunctionKey::Factorial => {
                if self.has_operand() {
                    self.append_raw("!");
                }
            }
            FunctionKey::Percent => {
                if self.has_operand() {
                    self.append_raw("%");
                }
            }
            FunctionKey::Sign => self.toggle_sign(),
            FunctionKey::Pi => self.append_raw("\u{03C0}"),
            FunctionKey::E => self.append_raw("e"),
            FunctionKey::Tau => self.append_raw("\u{03C4}"),
            FunctionKey::Phi => self.append_raw("\u{03C6}"),
            FunctionKey::Ans => self.append_raw("ans"),
            FunctionKey::OpenParen => self.input_open_paren(),
            FunctionKey::CloseParen => self.input_close_paren(),
            _ => {}
        }
        self.show_scientific = false;
    }

    pub fn memory_store(&mut self) {
        if let Some(value) = self.current_value() {
            self.memory = Some(value);
        }
    }

    pub fn memory_add(&mut self) {
        if let Some(value) = self.current_value() {
            self.memory = Some(self.memory.unwrap_or(0.0) + value);
        }
    }

    pub fn memory_subtract(&mut self) {
        if let Some(value) = self.current_value() {
            self.memory = Some(self.memory.unwrap_or(0.0) - value);
        }
    }

    pub fn memory_recall(&mut self) {
        if let Some(value) = self.memory {
            self.insert_raw(&format::as_expression(value));
        }
    }

    pub fn memory_clear(&mut self) {
        self.memory = None;
    }

    pub fn copy_text(&self) -> String {
        self.display_text()
    }

    /// Inserts a numeric value (used by the constants browser).
    pub fn insert_number(&mut self, value: f64) {
        self.insert_raw(&format::as_expression(value));
    }

    pub fn restore_history(&mut self, entries: Vec<HistoryEntry>) {
        self.history = entries;
        self.next_history_id = self
            .history
            .iter()
            .map(|entry| entry.id)
            .max()
            .unwrap_or(0)
            .saturating_add(1);
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

    pub fn load_history(&mut self, entry: &HistoryEntry) {
        if entry.result == "Error" {
            return;
        }
        if let Some(mode) = entry.angle_mode {
            self.angle_mode = mode;
        }
        self.expression = entry
            .value
            .map(format::as_expression)
            .unwrap_or_else(|| entry.result.clone());
        self.result = None;
        self.just_evaluated = false;
        self.last_value = None;
        self.show_scientific = false;
    }

    fn evaluate(&self, source: &str) -> Option<f64> {
        let mut vars = Vec::with_capacity(1);
        if let Some(ans) = self.ans {
            vars.push(("ans", ans));
        }
        expr::evaluate(source, self.angle_mode, &vars).ok()
    }

    fn current_value(&self) -> Option<f64> {
        if self.just_evaluated {
            return self.last_value;
        }
        let evaluable = expr::evaluable(&self.expression)?;
        self.evaluate(&evaluable)
    }

    fn continue_from_result(&mut self) -> bool {
        let Some(value) = self.last_value else {
            return false;
        };
        self.expression = format::as_expression(value);
        self.result = None;
        self.just_evaluated = false;
        self.last_value = None;
        true
    }

    fn insert_function(&mut self, name: &str) {
        if self.has_operand() {
            self.wrap_last_operand(&format!("{name}("), ")");
        } else {
            self.append_raw(&format!("{name}("));
        }
    }

    fn close_incomplete_parens(&mut self) {
        let Some(last) = self.expression.chars().last() else {
            return;
        };
        let complete = last.is_ascii_digit()
            || matches!(
                last,
                ')' | '\u{03C0}' | 'e' | '\u{03C4}' | '\u{03C6}' | '!' | '%'
            );
        if !complete {
            return;
        }
        let balance = paren_balance(&self.expression);
        if balance <= 0 {
            return;
        }
        let closing = ")".repeat(balance as usize);
        if self.expression.len() + closing.len() <= MAX_EXPRESSION {
            self.expression.push_str(&closing);
        }
    }

    fn wrap_last_operand(&mut self, prefix: &str, suffix: &str) {
        let Some(start) = self.last_operand_start() else {
            return;
        };
        let before = &self.expression[..start];
        let operand = &self.expression[start..];
        let candidate = format!("{before}{prefix}{operand}{suffix}");
        if candidate.len() <= MAX_EXPRESSION {
            self.expression = candidate;
        }
    }

    fn append_raw(&mut self, text: &str) {
        if self.expression.len() + text.len() <= MAX_EXPRESSION {
            self.expression.push_str(text);
        }
    }

    fn insert_raw(&mut self, text: &str) {
        if self.just_evaluated {
            self.clear();
        }
        self.append_raw(text);
    }

    fn has_operand(&self) -> bool {
        self.last_operand_start().is_some()
    }

    fn last_operand_start(&self) -> Option<usize> {
        if self.expression.is_empty() {
            return None;
        }
        let bytes = self.expression.as_bytes();
        let mut end = bytes.len();
        while end > 0 && matches!(bytes[end - 1], b'!' | b'%') {
            end -= 1;
        }
        if end == 0 {
            return None;
        }
        let last = bytes[end - 1];
        if last == b')' {
            let mut depth = 0i32;
            let mut index = end;
            while index > 0 {
                index -= 1;
                match bytes[index] {
                    b')' => depth += 1,
                    b'(' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
            }
            if depth != 0 {
                return None;
            }
            let mut start = index;
            let mut name_index = index;
            while name_index > 0 {
                let ch = self.expression[..name_index].chars().next_back()?;
                if ch.is_alphabetic() || ch == '\u{221A}' {
                    name_index -= ch.len_utf8();
                } else {
                    break;
                }
            }
            if name_index < index {
                start = name_index;
            }
            return Some(start);
        }
        if last.is_ascii_digit() || last == b'.' {
            let mut index = end;
            while index > 0 && (bytes[index - 1].is_ascii_digit() || bytes[index - 1] == b'.') {
                index -= 1;
            }
            return Some(index);
        }
        if last == b'e'
            || self.expression[..end].ends_with('\u{03C0}')
            || self.expression[..end].ends_with('\u{03C4}')
            || self.expression[..end].ends_with('\u{03C6}')
        {
            return Some(
                self.expression[..end]
                    .char_indices()
                    .next_back()
                    .map(|(i, _)| i)?,
            );
        }
        None
    }

    fn input_digit(&mut self, digit: &str) {
        let Some(ch) = digit.chars().next().filter(|ch| ch.is_ascii_digit()) else {
            return;
        };
        if self.just_evaluated {
            self.clear();
        }
        if self.expression.len() >= MAX_EXPRESSION {
            return;
        }
        let current = self.current_number();
        if current == "0" && ch != '0' {
            self.expression.pop();
            self.expression.push(ch);
        } else if current == "0" && ch == '0' {
            // Keep a single leading zero.
        } else {
            self.expression.push(ch);
        }
    }

    fn input_decimal(&mut self) {
        if self.just_evaluated {
            self.expression = "0.".to_string();
            self.result = None;
            self.just_evaluated = false;
            self.last_value = None;
            return;
        }
        if self.expression.len() >= MAX_EXPRESSION {
            return;
        }
        if let Some(last) = self.expression.chars().last() {
            if !last.is_ascii_digit()
                && last != '.'
                && !matches!(
                    last,
                    '+' | '\u{2212}' | '-' | '\u{00D7}' | '*' | '\u{00F7}' | '/' | '(' | '^'
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
            self.expression.push_str("0.");
        } else {
            self.expression.push('.');
        }
    }

    fn input_operator(&mut self, operator: &str) {
        let operator = match operator {
            "*" | "\u{00D7}" => "\u{00D7}",
            "/" | "\u{00F7}" => "\u{00F7}",
            "-" | "\u{2212}" => "\u{2212}",
            "+" => "+",
            "^" => "^",
            _ => return,
        };
        if self.just_evaluated {
            if self.is_error() {
                self.clear();
                if operator == "\u{2212}" {
                    self.expression.push('\u{2212}');
                }
                return;
            }
            if let Some(value) = self.last_value {
                self.expression = format::as_expression(value);
            } else if let Some(result) = &self.result {
                self.expression = result.clone();
            }
            self.result = None;
            self.just_evaluated = false;
            self.last_value = None;
            self.expression.push_str(operator);
            return;
        }
        if self.expression.is_empty() {
            if operator == "\u{2212}" {
                self.expression.push('\u{2212}');
            }
            return;
        }
        let last = self.expression.chars().last().unwrap();
        let len = self.expression.chars().count();
        match last {
            '+' | '\u{00D7}' | '\u{00F7}' => {
                self.expression.pop();
                self.expression.push_str(operator);
            }
            '^' => {
                self.expression.pop();
                self.expression.push_str(operator);
            }
            '\u{2212}' | '-' => {
                let previous = self.expression.chars().rev().nth(1);
                if len == 1 {
                    return;
                }
                if previous
                    .is_some_and(|ch| matches!(ch, '+' | '\u{00D7}' | '\u{00F7}' | '(' | '^'))
                {
                    return;
                }
                self.expression.pop();
                self.expression.push_str(operator);
            }
            '(' | '/' => {
                if operator == "\u{2212}" {
                    self.expression.push('\u{2212}');
                }
            }
            _ if self.expression.len() >= MAX_EXPRESSION => {}
            _ => self.expression.push_str(operator),
        }
    }

    fn input_open_paren(&mut self) {
        if self.just_evaluated {
            self.clear();
        }
        if self.expression.len() >= MAX_EXPRESSION {
            return;
        }
        let last = self.expression.chars().last();
        let needs_multiply = last.is_some_and(|ch| {
            ch.is_ascii_digit()
                || matches!(
                    ch,
                    ')' | '.' | '\u{03C0}' | 'e' | '\u{03C4}' | '\u{03C6}' | '!' | '%'
                )
        });
        if needs_multiply {
            self.expression.push_str("\u{00D7}(");
        } else {
            self.expression.push('(');
        }
    }

    fn input_close_paren(&mut self) {
        if self.just_evaluated {
            return;
        }
        if paren_balance(&self.expression) <= 0 {
            return;
        }
        let Some(last) = self.expression.chars().last() else {
            return;
        };
        if matches!(
            last,
            '+' | '\u{2212}' | '-' | '\u{00D7}' | '*' | '\u{00F7}' | '/' | '(' | '^' | '.'
        ) {
            return;
        }
        if self.expression.len() < MAX_EXPRESSION {
            self.expression.push(')');
        }
    }

    fn toggle_sign(&mut self) {
        if self.expression.starts_with('\u{2212}') || self.expression.starts_with('-') {
            self.expression.remove(0);
        } else if self.expression.is_empty() {
            self.expression.push('\u{2212}');
        } else {
            self.expression.insert(0, '\u{2212}');
        }
    }

    fn backspace(&mut self) {
        if self.just_evaluated || self.is_error() {
            self.clear();
            return;
        }
        self.expression.pop();
    }

    fn calculate(&mut self) {
        let Some(evaluable) = expr::evaluable(&self.expression) else {
            self.result = Some("Error".to_string());
            self.just_evaluated = true;
            self.last_value = None;
            return;
        };
        let Some(value) = self.evaluate(&evaluable) else {
            self.result = Some("Error".to_string());
            self.just_evaluated = true;
            self.last_value = None;
            return;
        };
        let formatted = format::plain_number(value);
        let meaningful = self.expression.chars().any(|ch| {
            matches!(
                ch,
                '+' | '\u{2212}'
                    | '-'
                    | '\u{00D7}'
                    | '*'
                    | '\u{00F7}'
                    | '/'
                    | '('
                    | ')'
                    | '^'
                    | '!'
                    | '%'
            )
        }) || formatted != evaluable;
        self.expression = evaluable;
        self.result = Some(formatted.clone());
        self.just_evaluated = true;
        self.last_value = Some(value);
        self.ans = Some(value);

        if meaningful {
            let angle_mode = expr::uses_trigonometry(&self.expression).then_some(self.angle_mode);
            let duplicate = self.history.first().is_some_and(|last| {
                last.expression == self.expression
                    && last.result == formatted
                    && last.angle_mode == angle_mode
            });
            if !duplicate {
                let id = self.next_history_id;
                self.next_history_id += 1;
                self.history.insert(
                    0,
                    HistoryEntry {
                        id,
                        expression: self.expression.clone(),
                        result: formatted,
                        angle_mode,
                        value: Some(value),
                        note: String::new(),
                    },
                );
                if self.history.len() > 100 {
                    self.history.pop();
                }
            }
        }
    }

    fn current_number(&self) -> &str {
        let start = self
            .expression
            .char_indices()
            .rev()
            .find(|(_, ch)| {
                matches!(
                    ch,
                    '+' | '\u{2212}' | '-' | '\u{00D7}' | '*' | '\u{00F7}' | '/' | '(' | '^'
                )
            })
            .map(|(index, ch)| index + ch.len_utf8())
            .unwrap_or(0);
        &self.expression[start..]
    }
}

const MAX_EXPRESSION: usize = 160;

fn paren_balance(expression: &str) -> i32 {
    expression.chars().fold(0, |balance, ch| match ch {
        '(' => balance + 1,
        ')' => balance - 1,
        _ => balance,
    })
}

trait IfEmpty {
    fn if_empty(self, fallback: &str) -> String;
}

impl IfEmpty for String {
    fn if_empty(self, fallback: &str) -> String {
        if self.is_empty() {
            fallback.to_string()
        } else {
            self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press_all(state: &mut CalculatorState, keys: &str) {
        for ch in keys.chars() {
            state.press(&ch.to_string());
        }
    }

    #[test]
    fn history_notes_are_attached_to_entries() {
        let mut state = CalculatorState::new();
        press_all(&mut state, "12+3=");
        let id = state.history[0].id;
        state.set_history_note(id, "rent split");
        assert_eq!(state.history_note(id), Some("rent split"));
        assert_eq!(state.history[0].note, "rent split");
    }

    #[test]
    fn builds_history_and_continues() {
        let mut state = CalculatorState::new();
        press_all(&mut state, "12+3=");
        assert_eq!(state.display_text(), "15");
        assert_eq!(state.history.len(), 1);
        assert_eq!(state.history[0].expression, "12+3");
        assert_eq!(state.history[0].result, "15");

        state.press("\u{00D7}");
        state.press("2");
        state.press("=");
        assert_eq!(state.display_text(), "30");
        assert_eq!(state.history.len(), 2);
    }

    #[test]
    fn decimal_and_clear() {
        let mut state = CalculatorState::new();
        for key in ["1", ".", "2", ".", "3"] {
            state.press(key);
        }
        assert_eq!(state.display_text(), "1.23");
        state.press("C");
        for key in ["9", "9", "\u{232B}"] {
            state.press(key);
        }
        assert_eq!(state.display_text(), "9");
        state.press("C");
        assert_eq!(state.display_text(), "0");
    }

    #[test]
    fn wraps_operand_with_function() {
        let mut state = CalculatorState::new();
        press_all(&mut state, "12+30");
        state.apply_function(FunctionKey::Sin);
        assert_eq!(state.expression(), "12+sin(30)");
        assert!(!state.show_scientific);
        state.press("=");
        assert_eq!(state.display_text(), "12.5");
        assert_eq!(state.history[0].angle_mode, Some(AngleMode::Degrees));
    }

    #[test]
    fn postfix_and_reciprocal() {
        let mut state = CalculatorState::new();
        state.press("5");
        state.apply_function(FunctionKey::Square);
        assert_eq!(state.expression(), "5^2");
        state.press("=");
        assert_eq!(state.display_text(), "25");

        state.clear();
        press_all(&mut state, "50");
        state.apply_function(FunctionKey::Percent);
        state.press("=");
        assert_eq!(state.display_text(), "0.5");

        state.clear();
        state.press("4");
        state.apply_function(FunctionKey::Reciprocal);
        assert_eq!(state.expression(), "1/(4)");
        state.press("=");
        assert_eq!(state.display_text(), "0.25");
    }

    #[test]
    fn closes_incomplete_group_before_postfix() {
        let mut state = CalculatorState::new();
        state.apply_function(FunctionKey::Sin);
        press_all(&mut state, "30");
        assert_eq!(state.expression(), "sin(30");
        state.apply_function(FunctionKey::Square);
        assert_eq!(state.expression(), "sin(30)^2");
        state.press("=");
        assert_eq!(state.display_text(), "0.25");
    }

    #[test]
    fn memory_and_ans() {
        let mut state = CalculatorState::new();
        press_all(&mut state, "2+3=");
        assert_eq!(state.ans(), Some(5.0));
        state.memory_store();
        assert_eq!(state.memory(), Some(5.0));
        state.clear();
        state.memory_recall();
        assert_eq!(state.expression(), "5");
        state.press("=");
        assert_eq!(state.display_text(), "5");
        state.memory_add();
        assert_eq!(state.memory(), Some(10.0));
    }

    #[test]
    fn errors_can_be_cleared_by_a_function() {
        let mut state = CalculatorState::new();
        press_all(&mut state, "1\u{00F7}0=");
        assert!(state.is_error());
        state.apply_function(FunctionKey::Sin);
        assert_eq!(state.expression(), "sin(");
        assert!(!state.is_error());
    }

    #[test]
    fn scientific_history_reloads_extreme_values() {
        let mut state = CalculatorState::new();
        press_all(&mut state, "10");
        state.apply_function(FunctionKey::Power);
        press_all(&mut state, "20=");
        assert_eq!(state.display_text(), "1e20");
        let entry = state.history[0].clone();
        state.clear();
        state.load_history(&entry);
        assert_eq!(state.expression(), "1\u{00D7}10^20");
        state.press("=");
        assert_eq!(state.display_text(), "1e20");
    }
}
