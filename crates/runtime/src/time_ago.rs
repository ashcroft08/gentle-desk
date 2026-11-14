//! "5m", "3h", "2d": how long ago something happened, for the sidebar, and how
//! long until something happens, for the subscription usage's reset countdown.

/// A short age of `then_ms` as seen at `now_ms` (both epoch milliseconds).
/// Future or identical times read as "now".
pub fn time_ago(now_ms: u64, then_ms: u64) -> String {
    let minutes = now_ms.saturating_sub(then_ms) / 60_000;
    let hours = minutes / 60;
    let days = hours / 24;
    match () {
        _ if minutes < 1 => "now".to_owned(),
        _ if hours < 1 => format!("{minutes}m"),
        _ if days < 1 => format!("{hours}h"),
        _ if days < 30 => format!("{days}d"),
        _ if days < 365 => format!("{}mo", days / 30),
        _ => format!("{}y", days / 365),
    }
}

/// A short wait until `at_ms` as seen at `now_ms` (both epoch milliseconds), in
/// the same labels [`time_ago`] draws: the subscription usage's "resets in 4h".
///
/// Rounded UP, the other way from an age: a countdown that says "3h" when 3h40m
/// remain understates the wait, and a reset under a minute still needs a minute
/// on the clock rather than "0m". A time that has passed, or is now, reads
/// "now".
///
/// The same rounding at every step, so a wait is never drawn shorter than it is.
///
/// # Panics
///
/// Never: a `now_ms` at or after `at_ms` is "now". A caller with a past time
/// generally wants [`time_ago`]'s rule instead; the usage rule asks this one
/// only after checking the order, and gets "now" either way.
pub fn time_until(now_ms: u64, at_ms: u64) -> String {
    if at_ms <= now_ms {
        return "now".to_owned();
    }
    const MINUTE: u64 = 60_000;
    const HOUR: u64 = 60 * MINUTE;
    const DAY: u64 = 24 * HOUR;
    let millis = at_ms - now_ms;
    let minutes = millis.div_ceil(MINUTE);
    let hours = millis.div_ceil(HOUR);
    let days = millis.div_ceil(DAY);
    let months = millis.div_ceil(30 * DAY);
    let years = millis.div_ceil(365 * DAY);
    match () {
        _ if millis < HOUR => format!("{minutes}m"),
        _ if millis < DAY => format!("{hours}h"),
        _ if millis < 30 * DAY => format!("{days}d"),
        _ if millis < 365 * DAY => format!("{months}mo"),
        _ => format!("{years}y"),
    }
}

#[cfg(test)]
mod tests;
