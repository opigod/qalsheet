//! Graph analysis: root finding, intersections, table values and grid steps.

use crate::expr::{AngleMode, Expression};

pub fn nice_step(range: f64, target_lines: f64) -> f64 {
    if !range.is_finite() || range <= 0.0 || target_lines <= 0.0 {
        return 1.0;
    }
    let raw = range / target_lines;
    let magnitude = 10_f64.powf(raw.log10().floor());
    let normalized = raw / magnitude;
    let factor = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    factor * magnitude
}

/// Finds x-intercepts in the visible window. Roots are sampled at pixel-like
/// density, refined by bisection and checked against the function value so a
/// vertical asymptote is not reported as a root.
pub fn find_roots(
    expression: &Expression,
    angle: AngleMode,
    x_min: f64,
    x_max: f64,
    samples: usize,
) -> Vec<f64> {
    find_roots_fn(
        &|x| expression.eval(angle, &[("x", x)]).ok(),
        x_min,
        x_max,
        samples,
    )
}

pub fn find_intersections(
    first: &Expression,
    second: &Expression,
    angle: AngleMode,
    x_min: f64,
    x_max: f64,
    samples: usize,
) -> Vec<(f64, f64)> {
    let roots = find_roots_fn(
        &|x| {
            let a = first.eval(angle, &[("x", x)]).ok()?;
            let b = second.eval(angle, &[("x", x)]).ok()?;
            Some(a - b)
        },
        x_min,
        x_max,
        samples,
    );
    roots
        .into_iter()
        .filter_map(|x| {
            let y = first.eval(angle, &[("x", x)]).ok()?;
            y.is_finite().then_some((x, y))
        })
        .collect()
}

fn find_roots_fn(
    function: &dyn Fn(f64) -> Option<f64>,
    x_min: f64,
    x_max: f64,
    samples: usize,
) -> Vec<f64> {
    let samples = samples.clamp(32, 20_000);
    let step = (x_max - x_min) / samples as f64;
    if !step.is_finite() || step <= 0.0 {
        return Vec::new();
    }
    let mut roots: Vec<f64> = Vec::new();
    let mut previous_x = x_min;
    let mut previous_y = function(previous_x);
    if previous_y.is_some_and(|value| value.abs() < 1e-10) {
        roots.push(previous_x);
    }
    for index in 1..=samples {
        let x = x_min + step * index as f64;
        let y = function(x);
        if let (Some(left), Some(right)) = (previous_y, y) {
            if right.abs() < 1e-10 {
                push_root(&mut roots, x);
            } else if left.abs() < 1e-10 {
                push_root(&mut roots, previous_x);
            } else if left.signum() != right.signum() {
                if let Some(root) = bisect(function, previous_x, x) {
                    push_root(&mut roots, root);
                }
            }
        }
        previous_x = x;
        previous_y = y;
    }
    roots.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    roots
}

fn bisect(function: &dyn Fn(f64) -> Option<f64>, mut left: f64, mut right: f64) -> Option<f64> {
    let mut left_value = function(left)?;
    let mut right_value = function(right)?;
    if left_value == 0.0 {
        return Some(left);
    }
    if right_value == 0.0 {
        return Some(right);
    }
    for _ in 0..80 {
        let middle = (left + right) / 2.0;
        let middle_value = function(middle)?;
        if middle_value.abs() < 1e-14 {
            return Some(middle);
        }
        if (middle - left).abs() < 1e-12 {
            break;
        }
        if left_value.signum() != middle_value.signum() {
            right = middle;
            right_value = middle_value;
        } else {
            left = middle;
            left_value = middle_value;
        }
    }
    let root = (left + right) / 2.0;
    let value = function(root)?;
    let tolerance = 1e-6 * (1.0 + root.abs());
    (value.abs() <= tolerance && right_value.is_finite()).then_some(root)
}

fn push_root(roots: &mut Vec<f64>, root: f64) {
    if roots
        .last()
        .is_none_or(|previous| (root - previous).abs() > 1e-6 * (1.0 + root.abs()))
    {
        roots.push(root);
    }
}

pub fn table_values(
    expression: &Expression,
    angle: AngleMode,
    start: f64,
    step: f64,
    count: usize,
) -> Vec<(f64, Option<f64>)> {
    (0..count)
        .map(|index| {
            let x = start + step * index as f64;
            let y = expression.eval(angle, &[("x", x)]).ok();
            (x, y)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_simple_roots() {
        let expression = crate::expr::compile("x^2-4").unwrap();
        let roots = find_roots(&expression, AngleMode::Radians, -5.0, 5.0, 1000);
        assert_eq!(roots.len(), 2);
        assert!((roots[0] + 2.0).abs() < 1e-6);
        assert!((roots[1] - 2.0).abs() < 1e-6);
    }

    #[test]
    fn finds_trig_roots_and_rejects_asymptotes() {
        let expression = crate::expr::compile("sin(x)").unwrap();
        let roots = find_roots(&expression, AngleMode::Radians, -10.0, 10.0, 2000);
        assert!(roots.iter().any(|root| root.abs() < 1e-5));
        assert!(roots.iter().filter(|root| **root > 0.0).count() >= 3);

        let tangent = crate::expr::compile("tan(x)").unwrap();
        let roots = find_roots(&tangent, AngleMode::Radians, -1.4, 1.4, 1000);
        assert!(roots.iter().all(|root| root.abs() < 0.01));
    }

    #[test]
    fn intersections() {
        let first = crate::expr::compile("x^2").unwrap();
        let second = crate::expr::compile("x+2").unwrap();
        let points = find_intersections(&first, &second, AngleMode::Radians, -3.0, 3.0, 1000);
        assert_eq!(points.len(), 2);
        assert!((points[0].0 + 1.0).abs() < 1e-6);
        assert!((points[1].0 - 2.0).abs() < 1e-6);
    }

    #[test]
    fn nice_steps() {
        assert!((nice_step(20.0, 10.0) - 2.0).abs() < 1e-12);
        assert!((nice_step(2.0, 10.0) - 0.2).abs() < 1e-12);
    }
}
