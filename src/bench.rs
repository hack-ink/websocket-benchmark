use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub struct BenchConfig {
	pub msg_count: u32,
	pub payload_len: usize,
	pub warmup_rounds: u32,
	pub rounds: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
	Rtt,
	Throughput,
}

#[derive(Clone, Copy, Debug)]
pub struct Stats {
	pub median: f64,
	pub p90: f64,
	pub p99: f64,
	pub mean: f64,
	pub stdev: f64,
}

pub fn duration_to_micros(value: Duration) -> f64 {
	value.as_secs_f64() * 1_000_000.0
}

pub fn compute_stats(values: &[f64]) -> Stats {
	let mut sorted = values.to_vec();
	sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

	let mean = values.iter().sum::<f64>() / values.len() as f64;
	let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / values.len() as f64;

	Stats {
		median: percentile(&sorted, 50.0),
		p90: percentile(&sorted, 90.0),
		p99: percentile(&sorted, 99.0),
		mean,
		stdev: variance.sqrt(),
	}
}

pub fn report_latency(samples_micros: &[f64]) {
	let stats = compute_stats(samples_micros);
	println!(
		"RTT result (us): median={:.2}, p90={:.2}, p99={:.2}, mean={:.2}, stdev={:.2}.",
		stats.median, stats.p90, stats.p99, stats.mean, stats.stdev
	);
}

pub fn report_throughput(samples_mib: &[f64]) {
	let stats = compute_stats(samples_mib);
	println!(
		"Throughput result (MiB/s, tx+rx): median={:.2}, p90={:.2}, p99={:.2}, mean={:.2}, stdev={:.2}.",
		stats.median, stats.p90, stats.p99, stats.mean, stats.stdev
	);
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
	if sorted.is_empty() {
		return f64::NAN;
	}

	let rank = p / 100.0 * (sorted.len() - 1) as f64;
	let lower = rank.floor() as usize;
	let upper = rank.ceil() as usize;

	if lower == upper {
		sorted[lower]
	} else {
		let weight = rank - lower as f64;
		sorted[lower] + (sorted[upper] - sorted[lower]) * weight
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn approx_eq(actual: f64, expected: f64, eps: f64) {
		assert!((actual - expected).abs() <= eps, "Expected {expected}, got {actual}");
	}

	#[test]
	fn stats_basic_distribution() {
		let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
		let stats = compute_stats(&values);

		approx_eq(stats.median, 3.0, 1e-9);
		approx_eq(stats.p90, 4.6, 1e-9);
		approx_eq(stats.p99, 4.96, 1e-9);
		approx_eq(stats.mean, 3.0, 1e-9);
		approx_eq(stats.stdev, 2.0_f64.sqrt(), 1e-9);
	}

	#[test]
	fn stats_single_value() {
		let values = vec![42.0];
		let stats = compute_stats(&values);

		approx_eq(stats.median, 42.0, 1e-9);
		approx_eq(stats.p90, 42.0, 1e-9);
		approx_eq(stats.p99, 42.0, 1e-9);
		approx_eq(stats.mean, 42.0, 1e-9);
		approx_eq(stats.stdev, 0.0, 1e-9);
	}

	#[test]
	fn duration_to_micros_converts_seconds() {
		let duration = Duration::from_secs_f64(1.5);
		approx_eq(duration_to_micros(duration), 1_500_000.0, 1e-6);
	}
}
