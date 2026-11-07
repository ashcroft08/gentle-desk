//! Candidate B: cost-bounded polling, as one pure scheduling function.
//!
//! The shipped code refreshes every 5 s whatever the last pass cost, so the
//! work per second grows with the number of chats. This decides *when* the next
//! pass may start, from what the last one cost:
//!
//! `delay = max(base, K x last_pass)`
//!
//! With `K = 20` the polling can use at most a twentieth of a core (5 %).
//! Nothing here reads the clock or the file system: this is the whole strategy,
//! and the example reports what it would do with the measured pass times.

use std::time::Duration;

/// The next refresh may start in this long.
///
/// Never sooner than `base`, and never sooner than `k` times the last pass, so
/// a slow home refreshes less often instead of spending more of the CPU. `k`
/// below 1 is treated as 1: a pass never starts back to back with the previous
/// one, and durations too large to multiply saturate instead of overflowing.
pub fn next_delay(last_pass: Duration, base: Duration, k: u32) -> Duration {
    base.max(last_pass.saturating_mul(k.max(1)))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: Duration = Duration::from_secs(5);
    const K: u32 = 20;

    fn ms(value: u64) -> Duration {
        Duration::from_millis(value)
    }

    #[test]
    fn fast_passes_keep_the_base_interval() {
        // A warm pass over a small home, and one that is still below base / K.
        assert_eq!(next_delay(ms(1), BASE, K), BASE);
        assert_eq!(next_delay(ms(100), BASE, K), BASE);
        // The boundary is exactly base / K = 250 ms: still no stretch.
        assert_eq!(next_delay(ms(249), BASE, K), BASE);
        assert_eq!(next_delay(ms(250), BASE, K), BASE);
    }

    #[test]
    fn slow_passes_stretch_the_delay_proportionally() {
        assert_eq!(next_delay(ms(251), BASE, K), ms(5020));
        // The 50 000-chat pass measured in virtual-sidebar-m13.
        assert_eq!(next_delay(ms(1200), BASE, K), ms(24_000));
        assert_eq!(next_delay(Duration::from_secs(60), BASE, K), ms(1_200_000));
    }

    #[test]
    fn a_zero_pass_is_just_the_base_interval() {
        assert_eq!(next_delay(Duration::ZERO, BASE, K), BASE);
    }

    #[test]
    fn k_below_one_cannot_make_back_to_back_passes() {
        // A 10 s pass stays 10 s apart even with K = 0 (treated as 1).
        assert_eq!(next_delay(ms(10_000), BASE, 0), ms(10_000));
        assert_eq!(next_delay(ms(10_000), BASE, 1), ms(10_000));
        // A fast pass with K = 1 still waits the base interval.
        assert_eq!(next_delay(ms(100), BASE, 1), BASE);
    }

    #[test]
    fn huge_durations_saturate_instead_of_overflowing() {
        assert_eq!(next_delay(Duration::MAX, BASE, K), Duration::MAX);
        assert_eq!(
            next_delay(Duration::from_secs(u64::MAX / 2), BASE, K),
            Duration::MAX
        );
        assert_eq!(
            next_delay(Duration::from_secs(u64::MAX), BASE, u32::MAX),
            Duration::MAX
        );
    }

    #[test]
    fn the_cpu_fraction_is_bounded_by_one_over_k_once_stretched() {
        for micros in [1_000u64, 50_000, 250_000, 251_000, 1_200_000, 30_000_000] {
            let last = Duration::from_micros(micros);
            let delay = next_delay(last, BASE, K);
            assert!(
                delay >= last,
                "the next pass would start before the last one finished"
            );
            let fraction = last.as_secs_f64() / delay.as_secs_f64();
            assert!(
                fraction <= 1.0 / f64::from(K) + f64::EPSILON,
                "{fraction} of a core is more than 1/{K} for a pass of {last:?}"
            );
        }
    }
}
