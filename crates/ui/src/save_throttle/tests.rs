use super::*;
use std::time::Duration;

fn secs(value: u64) -> Duration {
    Duration::from_secs(value)
}

#[test]
fn a_clean_cache_is_never_written() {
    let throttle = SaveThrottle::new(secs(30));
    assert!(!throttle.is_dirty());
    assert!(!throttle.should_save(secs(0)));
    assert!(!throttle.should_save(secs(10_000)));
}

#[test]
fn the_first_change_is_written_at_once() {
    let mut throttle = SaveThrottle::new(secs(30));
    throttle.mark_dirty();
    assert!(throttle.is_dirty());
    assert!(throttle.should_save(secs(0)));
}

#[test]
fn a_second_change_within_the_interval_waits() {
    let mut throttle = SaveThrottle::new(secs(30));
    throttle.mark_dirty();
    assert!(throttle.should_save(secs(1)));
    throttle.saved(secs(1));
    assert!(!throttle.is_dirty());

    throttle.mark_dirty();
    assert!(!throttle.should_save(secs(2)));
    assert!(!throttle.should_save(secs(30)));
    // Exactly one interval later the write is due.
    assert!(throttle.should_save(secs(31)));
}

#[test]
fn after_the_interval_the_write_is_due_and_the_interval_restarts() {
    let mut throttle = SaveThrottle::new(secs(30));
    throttle.mark_dirty();
    throttle.saved(secs(0));

    throttle.mark_dirty();
    assert!(throttle.should_save(secs(30)));
    throttle.saved(secs(30));
    throttle.mark_dirty();
    assert!(!throttle.should_save(secs(59)));
    assert!(throttle.should_save(secs(60)));
}

#[test]
fn a_change_before_a_save_keeps_the_cache_dirty() {
    let mut throttle = SaveThrottle::new(secs(30));
    throttle.mark_dirty();
    throttle.saved(secs(0));
    throttle.mark_dirty();
    throttle.mark_dirty();
    assert!(throttle.is_dirty());
    assert!(!throttle.should_save(secs(29)));
    assert!(throttle.should_save(secs(30)));
}

#[test]
fn a_backwards_clock_does_not_panic_and_does_not_save_early() {
    let mut throttle = SaveThrottle::new(secs(30));
    throttle.mark_dirty();
    throttle.saved(secs(100));
    throttle.mark_dirty();
    // Now reads earlier than the last save: the elapsed time saturates at zero.
    assert!(!throttle.should_save(secs(90)));
    assert!(!throttle.should_save(secs(0)));
    assert!(throttle.should_save(secs(130)));
}
