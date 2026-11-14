//! QalSheet's dependency-free calculation core.
//!
//! Everything here is plain Rust with no allocation-heavy framework behind it:
//! the expression parser builds a small AST once, graph sampling reuses it, and
//! the algebra solver works on sparse polynomials. The Wear, phone and desktop
//! frontends all share this crate.

pub mod algebra;
pub mod calculator;
pub mod calculus;
pub mod constants;
pub mod expr;
pub mod format;
pub mod graph;
pub mod programmer;
pub mod stats;
pub mod units;

pub use expr::{AngleMode, EvalError, Expression};
