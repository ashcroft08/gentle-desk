//! The scheduling function moved from the refresh-strategies measurement
//! example, with its assertions unchanged, plus the full-pass cadence.

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

#[test]
fn a_watching_home_reconciles_at_most_once_a_minute() {
    assert_eq!(RECONCILE_BASE, Duration::from_secs(60));
    // A fast pass while watching waits the full floor, not the 5 s poll.
    assert_eq!(full_pass_delay(true, Duration::ZERO), RECONCILE_BASE);
    assert_eq!(full_pass_delay(true, ms(1)), RECONCILE_BASE);
    // The floor is 60 s / K = 3 s of pass time: still no stretch.
    assert_eq!(full_pass_delay(true, ms(2_999)), RECONCILE_BASE);
    assert_eq!(full_pass_delay(true, ms(3_000)), RECONCILE_BASE);
}

#[test]
fn a_watching_home_stretches_a_slow_pass_proportionally() {
    assert_eq!(full_pass_delay(true, ms(3_001)), ms(60_020));
    // The 1.5 s pass of a 50 000-chat home stays well below the floor.
    assert_eq!(full_pass_delay(true, ms(1_500)), RECONCILE_BASE);
    // A pass that takes longer than a minute can only stretch further.
    assert_eq!(
        full_pass_delay(true, Duration::from_secs(60)),
        ms(1_200_000)
    );
}

#[test]
fn a_home_without_a_watcher_polls_every_five_seconds() {
    assert_eq!(POLL_BASE, Duration::from_secs(5));
    assert_eq!(full_pass_delay(false, Duration::ZERO), POLL_BASE);
    assert_eq!(full_pass_delay(false, ms(1)), POLL_BASE);
    // The stretched poll is the same one the measurement example reported.
    assert_eq!(full_pass_delay(false, ms(1_200)), ms(24_000));
}

#[test]
fn the_full_pass_cadence_is_safe_at_the_extremes() {
    for watching in [true, false] {
        assert_eq!(full_pass_delay(watching, Duration::MAX), Duration::MAX);
        assert_eq!(
            full_pass_delay(watching, Duration::from_secs(u64::MAX / 2)),
            Duration::MAX
        );
        // A zero pass is the static floor, never a back-to-back pass.
        let floor = if watching { RECONCILE_BASE } else { POLL_BASE };
        assert_eq!(full_pass_delay(watching, Duration::ZERO), floor);
    }
}
