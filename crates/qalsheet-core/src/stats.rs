//! One-variable statistics, quartiles and simple linear regression.

use crate::format;

#[derive(Clone, Debug, Default)]
pub struct Stats {
    pub count: usize,
    pub sum: f64,
    pub mean: f64,
    pub median: f64,
    pub min: f64,
    pub max: f64,
    pub range: f64,
    pub q1: f64,
    pub q3: f64,
    pub iqr: f64,
    pub population_variance: f64,
    pub sample_variance: f64,
    pub population_stddev: f64,
    pub sample_stddev: f64,
    pub mode: Option<f64>,
}

impl Stats {
    pub fn compute(values: &[f64]) -> Option<Self> {
        if values.is_empty() {
            return None;
        }
        let mut sorted = values.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let count = sorted.len();
        let sum: f64 = sorted.iter().sum();
        let mean = sum / count as f64;
        let median_value = median(&sorted);
        let min = sorted[0];
        let max = sorted[count - 1];
        let population_variance = sorted
            .iter()
            .map(|value| (value - mean).powi(2))
            .sum::<f64>()
            / count as f64;
        let sample_variance = if count > 1 {
            sorted
                .iter()
                .map(|value| (value - mean).powi(2))
                .sum::<f64>()
                / (count - 1) as f64
        } else {
            0.0
        };
        let middle = count / 2;
        let q1 = median(&sorted[..middle]);
        let q3 = if count % 2 == 0 {
            median(&sorted[middle..])
        } else {
            median(&sorted[middle + 1..])
        };

        let mut best_value = sorted[0];
        let mut best_count = 0;
        let mut index = 0;
        while index < sorted.len() {
            let value = sorted[index];
            let mut end = index + 1;
            while end < sorted.len() && sorted[end] == value {
                end += 1;
            }
            let frequency = end - index;
            if frequency > best_count {
                best_count = frequency;
                best_value = value;
            }
            index = end;
        }
        let mode = (best_count > 1).then_some(best_value);

        Some(Self {
            count,
            sum,
            mean,
            median: median_value,
            min,
            max,
            range: max - min,
            q1,
            q3,
            iqr: q3 - q1,
            population_variance,
            sample_variance,
            population_stddev: population_variance.sqrt(),
            sample_stddev: sample_variance.sqrt(),
            mode,
        })
    }
}

fn median(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }
    let middle = values.len() / 2;
    if values.len() % 2 == 0 {
        (values[middle - 1] + values[middle]) / 2.0
    } else {
        values[middle]
    }
}

#[derive(Clone, Debug)]
pub struct Regression {
    pub slope: f64,
    pub intercept: f64,
    pub r: f64,
    pub r_squared: f64,
    pub count: usize,
}

pub fn linear_regression(xs: &[f64], ys: &[f64]) -> Option<Regression> {
    if xs.len() != ys.len() || xs.len() < 2 {
        return None;
    }
    let count = xs.len();
    let mean_x = xs.iter().sum::<f64>() / count as f64;
    let mean_y = ys.iter().sum::<f64>() / count as f64;
    let mut covariance = 0.0;
    let mut variance_x = 0.0;
    let mut variance_y = 0.0;
    for (x, y) in xs.iter().zip(ys) {
        covariance += (x - mean_x) * (y - mean_y);
        variance_x += (x - mean_x).powi(2);
        variance_y += (y - mean_y).powi(2);
    }
    if variance_x == 0.0 {
        return None;
    }
    let slope = covariance / variance_x;
    let intercept = mean_y - slope * mean_x;
    let r = if variance_y == 0.0 {
        0.0
    } else {
        covariance / (variance_x.sqrt() * variance_y.sqrt())
    };
    Some(Regression {
        slope,
        intercept,
        r,
        r_squared: r * r,
        count,
    })
}

/// Parses a free-form list: commas, semicolons, spaces and newlines all work.
pub fn parse_numbers(input: &str) -> Vec<f64> {
    input
        .replace('\u{2212}', " -")
        .replace([';', ','], " ")
        .split_whitespace()
        .filter_map(|token| token.parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .collect()
}

/// Parses two columns (x and y) for regression. Each pair is separated by a
/// newline, semicolon or comma; spaces separate the two numbers.
pub fn parse_pairs(input: &str) -> Option<(Vec<f64>, Vec<f64>)> {
    let mut xs = Vec::new();
    let mut ys = Vec::new();
    for line in input.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line
            .split(|ch: char| ch == ',' || ch == ';' || ch.is_whitespace())
            .filter(|part| !part.is_empty())
            .collect();
        if parts.len() < 2 {
            continue;
        }
        let x = parts[0].replace('\u{2212}', "-").parse::<f64>().ok()?;
        let y = parts[1].replace('\u{2212}', "-").parse::<f64>().ok()?;
        xs.push(x);
        ys.push(y);
    }
    (xs.len() >= 2).then_some((xs, ys))
}

pub fn summary(stats: &Stats) -> String {
    format!(
        "n={}, mean={}, median={}, sample sd={}",
        stats.count,
        format::plain_number(stats.mean),
        format::plain_number(stats.median),
        format::plain_number(stats.sample_stddev)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_statistics() {
        let stats = Stats::compute(&[2.0, 4.0, 4.0, 4.0, 5.0, 5.0, 7.0, 9.0]).unwrap();
        assert_eq!(stats.count, 8);
        assert!((stats.mean - 5.0).abs() < 1e-12);
        assert!((stats.median - 4.5).abs() < 1e-12);
        assert!((stats.population_variance - 4.0).abs() < 1e-12);
        assert!((stats.sample_variance - 32.0 / 7.0).abs() < 1e-12);
        assert!((stats.q1 - 4.0).abs() < 1e-12);
        assert!((stats.q3 - 6.0).abs() < 1e-12);
        assert_eq!(stats.mode, Some(4.0));
    }

    #[test]
    fn parsing_and_regression() {
        let values = parse_numbers("1, 2; 3\n4\u{2212}5");
        assert_eq!(values, vec![1.0, 2.0, 3.0, 4.0, -5.0]);
        let (xs, ys) = parse_pairs("1 2\n2 4\n3 6\n").unwrap();
        let regression = linear_regression(&xs, &ys).unwrap();
        assert!((regression.slope - 2.0).abs() < 1e-12);
        assert!((regression.intercept).abs() < 1e-12);
        assert!((regression.r_squared - 1.0).abs() < 1e-12);
    }

    #[test]
    fn empty_input() {
        assert!(Stats::compute(&[]).is_none());
        assert!(linear_regression(&[1.0], &[2.0]).is_none());
    }
}
