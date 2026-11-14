//! Numerical calculus helpers for the tool drawer.

use crate::expr::{self, AngleMode, EvalError, Expression};

/// Five-point central difference. Accurate to roughly h^4 for smooth curves.
pub fn derivative(expression: &Expression, x: f64, angle: AngleMode) -> Result<f64, EvalError> {
    let h = 1e-5 * x.abs().max(1.0);
    let f = |value: f64| expression.eval(angle, &[("x", value)]);
    let value =
        (-f(x + 2.0 * h)? + 8.0 * f(x + h)? - 8.0 * f(x - h)? + f(x - 2.0 * h)?) / (12.0 * h);
    if value.is_finite() {
        Ok(value)
    } else {
        Err(EvalError {
            message: "Derivative is not finite here".to_string(),
        })
    }
}

/// Adaptive Simpson integration. Handles ordinary textbook functions well and
/// reports a domain error instead of inventing a value across a discontinuity.
pub fn integrate(
    expression: &Expression,
    lower: f64,
    upper: f64,
    angle: AngleMode,
) -> Result<f64, EvalError> {
    if !lower.is_finite() || !upper.is_finite() {
        return Err(EvalError {
            message: "Use finite limits".to_string(),
        });
    }
    if lower == upper {
        return Ok(0.0);
    }
    let (a, b) = if lower < upper {
        (lower, upper)
    } else {
        (upper, lower)
    };
    let sign = if lower < upper { 1.0 } else { -1.0 };
    let f = |value: f64| expression.eval(angle, &[("x", value)]);
    let value = adaptive_simpson(&f, a, b, f(a)?, f(b)?, f((a + b) / 2.0)?, 1e-10, 20)?;
    Ok(sign * value)
}

#[allow(clippy::too_many_arguments)]
fn adaptive_simpson(
    f: &dyn Fn(f64) -> Result<f64, EvalError>,
    a: f64,
    b: f64,
    fa: f64,
    fb: f64,
    fm: f64,
    tolerance: f64,
    depth: u32,
) -> Result<f64, EvalError> {
    let middle = (a + b) / 2.0;
    let left_middle = (a + middle) / 2.0;
    let right_middle = (middle + b) / 2.0;
    let flm = f(left_middle)?;
    let frm = f(right_middle)?;
    let whole = (b - a) / 6.0 * (fa + 4.0 * fm + fb);
    let left = (middle - a) / 6.0 * (fa + 4.0 * flm + fm);
    let right = (b - middle) / 6.0 * (fm + 4.0 * frm + fb);
    let delta = left + right - whole;
    if depth == 0 || delta.abs() <= 15.0 * tolerance {
        return Ok(left + right + delta / 15.0);
    }
    Ok(
        adaptive_simpson(f, a, middle, fa, fm, flm, tolerance / 2.0, depth - 1)?
            + adaptive_simpson(f, middle, b, fm, fb, frm, tolerance / 2.0, depth - 1)?,
    )
}

/// Compiles once and evaluates a single number, used by the tool UI.
pub fn evaluate_at(source: &str, x: f64, angle: AngleMode) -> Result<f64, EvalError> {
    expr::compile(source)?.eval(angle, &[("x", x)])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derivative_of_square() {
        let expression = expr::compile("x^2").unwrap();
        let value = derivative(&expression, 3.0, AngleMode::Radians).unwrap();
        assert!((value - 6.0).abs() < 1e-7);
    }

    #[test]
    fn integral_of_square() {
        let expression = expr::compile("x^2").unwrap();
        let value = integrate(&expression, 0.0, 3.0, AngleMode::Radians).unwrap();
        assert!((value - 9.0).abs() < 1e-8);
    }

    #[test]
    fn reversed_limits() {
        let expression = expr::compile("x").unwrap();
        let value = integrate(&expression, 2.0, 0.0, AngleMode::Radians).unwrap();
        assert!((value + 2.0).abs() < 1e-9);
    }
}
