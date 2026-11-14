//! Small, dependency-free persistence for calculation history and notes.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use qalsheet_core::AngleMode;
use qalsheet_core::algebra::AlgebraState;
use qalsheet_core::calculator::{CalculatorState, HistoryEntry};

const HEADER: &str = "qalsheet-state-v1";

static STATE_PATH: OnceLock<PathBuf> = OnceLock::new();

pub fn set_state_path(path: Option<PathBuf>) {
    if let Some(path) = path {
        let _ = STATE_PATH.set(path);
    }
}

pub fn state_path() -> Option<&'static Path> {
    STATE_PATH.get().map(PathBuf::as_path)
}

pub fn desktop_state_path() -> Option<PathBuf> {
    if let Some(base) = std::env::var_os("XDG_STATE_HOME") {
        return Some(PathBuf::from(base).join("qalsheet").join("state.txt"));
    }
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home)
            .join(".local")
            .join("state")
            .join("qalsheet")
            .join("state.txt")
    })
}

/// Loads persisted history entries into both tabs. Missing or malformed files
/// leave the current state untouched.
pub fn load(calculator: &mut CalculatorState, algebra: &mut AlgebraState) {
    let Some(path) = state_path() else {
        log::warn!("QalSheet state path is not set");
        return;
    };
    let Ok(text) = fs::read_to_string(path) else {
        log::info!("No QalSheet state at {path:?}");
        return;
    };

    let mut calculator_entries = Vec::new();
    let mut algebra_entries = Vec::new();
    for line in text.lines() {
        if line == HEADER || line.trim().is_empty() {
            continue;
        }
        let mut parts = line.splitn(7, '\t');
        match parts.next() {
            Some("C") => {
                let Some(id) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
                    continue;
                };
                let angle = match parts.next().unwrap_or("-") {
                    "D" => Some(AngleMode::Degrees),
                    "R" => Some(AngleMode::Radians),
                    _ => None,
                };
                let value = parts.next().and_then(|value| value.parse::<f64>().ok());
                let Some(expression) = parts.next().map(unescape) else {
                    continue;
                };
                let Some(result) = parts.next().map(unescape) else {
                    continue;
                };
                let note = parts.next().map(unescape).unwrap_or_default();
                calculator_entries.push(HistoryEntry {
                    id,
                    expression,
                    result,
                    angle_mode: angle,
                    value,
                    note,
                });
            }
            Some("A") => {
                let Some(id) = parts.next().and_then(|value| value.parse::<u64>().ok()) else {
                    continue;
                };
                let Some(variable) = parts.next().and_then(|value| value.chars().next()) else {
                    continue;
                };
                let Some(equation) = parts.next().map(unescape) else {
                    continue;
                };
                let note = parts.next().map(unescape).unwrap_or_default();
                algebra_entries.push((id, variable, equation, note));
            }
            _ => {}
        }
    }

    if !calculator_entries.is_empty() {
        calculator.restore_history(calculator_entries);
    }
    if !algebra_entries.is_empty() {
        algebra.restore_history(algebra_entries);
    }
}

/// Saves both history lists and their notes. The write is atomic: a temporary
/// file is renamed over the old one.
pub fn save(calculator: &CalculatorState, algebra: &AlgebraState) {
    let Some(path) = state_path() else {
        return;
    };
    let mut text = String::with_capacity(4096 + calculator.history.len() * 96);
    text.push_str(HEADER);
    text.push('\n');
    for entry in &calculator.history {
        let angle = match entry.angle_mode {
            Some(AngleMode::Degrees) => "D",
            Some(AngleMode::Radians) => "R",
            None => "-",
        };
        let value = entry
            .value
            .map(|value| value.to_string())
            .unwrap_or_default();
        text.push_str("C\t");
        text.push_str(&entry.id.to_string());
        text.push('\t');
        text.push_str(angle);
        text.push('\t');
        text.push_str(&value);
        text.push('\t');
        text.push_str(&escape(&entry.expression));
        text.push('\t');
        text.push_str(&escape(&entry.result));
        text.push('\t');
        text.push_str(&escape(&entry.note));
        text.push('\n');
    }
    for entry in &algebra.history {
        text.push_str("A\t");
        text.push_str(&entry.id.to_string());
        text.push('\t');
        text.push(entry.variable);
        text.push('\t');
        text.push_str(&escape(&entry.equation));
        text.push('\t');
        text.push_str(&escape(&entry.note));
        text.push('\n');
    }

    if let Some(parent) = path.parent() {
        if let Err(error) = fs::create_dir_all(parent) {
            log::warn!("QalSheet state directory {parent:?}: {error}");
            return;
        }
    }
    let temporary = path.with_extension("tmp");
    if let Err(error) = fs::write(&temporary, text) {
        log::warn!("QalSheet state write {temporary:?}: {error}");
        return;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        log::warn!("QalSheet state rename {temporary:?} -> {path:?}: {error}");
        return;
    }
    log::info!("QalSheet state saved to {path:?}");
}

fn escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('\t', "\\t")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn unescape(value: &str) -> String {
    let mut output = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            output.push(ch);
            continue;
        }
        match chars.next() {
            Some('n') => output.push('\n'),
            Some('r') => output.push('\r'),
            Some('t') => output.push('\t'),
            Some('\\') => output.push('\\'),
            Some(other) => output.push(other),
            None => output.push('\\'),
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escaping_round_trips() {
        let value = "line 1\nline 2\twith \\ slash";
        assert_eq!(unescape(&escape(value)), value);
    }
}
