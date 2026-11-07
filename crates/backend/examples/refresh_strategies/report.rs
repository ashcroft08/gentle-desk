//! Turning samples into numbers and numbers into aligned tables.
//!
//! No measurement happens here: this only formats what the two measured
//! candidates report, so every table in the output uses the same units.

use std::time::Duration;

/// Milliseconds with one decimal: the unit the per-pass figures use.
pub fn ms(duration: Duration) -> String {
    format!("{:.1} ms", duration.as_secs_f64() * 1000.0)
}

/// Seconds with two decimals: easier to read for delays and staleness.
pub fn secs(duration: Duration) -> String {
    format!("{:.2} s", duration.as_secs_f64())
}

/// `work` as a share of one core over `wall`, in percent.
pub fn core_fraction(work: Duration, wall: Duration) -> String {
    let fraction = work.as_secs_f64() / wall.as_secs_f64().max(f64::MIN_POSITIVE);
    format!("{:.2} %", fraction * 100.0)
}

/// Bytes as MiB with one decimal.
pub fn mib(bytes: u64) -> String {
    format!("{:.1} MiB", bytes as f64 / (1024.0 * 1024.0))
}

/// min / median / max of a set of samples.
#[derive(Debug, Clone, Copy)]
pub struct Spread {
    pub min: Duration,
    pub median: Duration,
    pub max: Duration,
}

impl Spread {
    /// Sorts a copy of `samples`; an even count averages the two middle ones.
    pub fn of(mut samples: Vec<Duration>) -> Self {
        assert!(!samples.is_empty(), "a spread needs at least one sample");
        samples.sort();
        let middle = samples.len() / 2;
        let median = if samples.len() % 2 == 1 {
            samples[middle]
        } else {
            let (a, b) = (samples[middle - 1], samples[middle]);
            Duration::from_nanos(((a.as_nanos() + b.as_nanos()) / 2) as u64)
        };
        Self {
            min: samples[0],
            median,
            max: samples[samples.len() - 1],
        }
    }
}

/// Prints a header row and its rows, every column padded to its widest cell.
pub fn table(headers: &[&str], rows: &[Vec<String>]) {
    let mut widths: Vec<usize> = headers.iter().map(|h| h.chars().count()).collect();
    for row in rows {
        for (index, cell) in row.iter().enumerate() {
            widths[index] = widths[index].max(cell.chars().count());
        }
    }
    print_row(
        &headers.iter().map(|h| (*h).to_owned()).collect::<Vec<_>>(),
        &widths,
    );
    print_row(
        &widths
            .iter()
            .map(|width| "-".repeat(*width))
            .collect::<Vec<_>>(),
        &widths,
    );
    for row in rows {
        print_row(row, &widths);
    }
}

fn print_row(cells: &[String], widths: &[usize]) {
    let padded: Vec<String> = cells
        .iter()
        .enumerate()
        .map(|(index, cell)| format!("{:<width$}", cell, width = widths[index]))
        .collect();
    println!("| {} |", padded.join(" | "));
}
