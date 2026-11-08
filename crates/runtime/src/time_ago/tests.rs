use super::*;

const SEC: u64 = 1000;
const MIN: u64 = 60 * SEC;
const HOUR: u64 = 60 * MIN;
const DAY: u64 = 24 * HOUR;
const NOW: u64 = 1_790_000_000_000;

fn ago(delta: u64) -> String {
    time_ago(NOW, NOW - delta)
}

#[test]
fn under_a_minute_is_now() {
    assert_eq!(ago(0), "now");
    assert_eq!(ago(59 * SEC), "now");
}

#[test]
fn a_time_in_the_future_is_now() {
    assert_eq!(time_ago(NOW, NOW + 5 * MIN), "now");
}

#[test]
fn minutes_hours_and_days_round_down() {
    assert_eq!(ago(MIN), "1m");
    assert_eq!(ago(59 * MIN + 59 * SEC), "59m");
    assert_eq!(ago(HOUR), "1h");
    assert_eq!(ago(23 * HOUR + 59 * MIN), "23h");
    assert_eq!(ago(DAY), "1d");
    assert_eq!(ago(29 * DAY + 23 * HOUR), "29d");
}

#[test]
fn older_than_thirty_days_is_months_then_years() {
    assert_eq!(ago(30 * DAY), "1mo");
    assert_eq!(ago(364 * DAY), "12mo");
    assert_eq!(ago(365 * DAY), "1y");
    assert_eq!(ago(800 * DAY), "2y");
}

#[test]
fn an_unknown_zero_time_does_not_panic() {
    assert_eq!(time_ago(NOW, 0), "56y");
}
