//! The throttle on its own: no thread, no clock, just the times it is asked with.

use super::*;

#[test]
fn the_first_item_passes_at_once() {
    let mut throttle = Throttle::new(Duration::from_millis(500));
    assert!(throttle.allow(Duration::ZERO));
}

#[test]
fn a_second_item_waits_for_the_interval() {
    let mut throttle = Throttle::new(Duration::from_millis(500));
    assert!(throttle.allow(Duration::ZERO));
    assert!(!throttle.allow(Duration::from_millis(499)));
    assert!(throttle.allow(Duration::from_millis(500)));
}

/// The interval is measured from the item actually sent, so a refused ask costs
/// nothing: a pass that asks on every file cannot shorten its own interval.
#[test]
fn the_interval_counts_from_the_item_actually_sent() {
    let mut throttle = Throttle::new(Duration::from_millis(500));
    assert!(throttle.allow(Duration::from_millis(100)));
    assert!(!throttle.allow(Duration::from_millis(599)));
    assert!(throttle.allow(Duration::from_millis(600)));
    assert!(!throttle.allow(Duration::from_millis(1099)));
    assert!(throttle.allow(Duration::from_millis(1100)));
}

#[test]
fn no_more_than_one_item_per_interval() {
    let mut throttle = Throttle::new(Duration::from_millis(500));
    let mut sent = 0;
    let mut now = Duration::ZERO;
    // A 20 s pass asking every 100 ms: one item per interval, never two.
    for _ in 0..=200 {
        if throttle.allow(now) {
            sent += 1;
        }
        now += Duration::from_millis(100);
    }
    assert_eq!(sent, 41);
}

#[test]
fn the_default_interval_is_the_one_a_pass_uses() {
    let mut throttle = Throttle::default();
    assert!(throttle.allow(Duration::ZERO));
    assert!(!throttle.allow(INTERVAL - Duration::from_millis(1)));
    assert!(throttle.allow(INTERVAL));
}
