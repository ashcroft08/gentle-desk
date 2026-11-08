//! "5m", "3h", "2d": how long ago something happened, for the sidebar.

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

#[cfg(test)]
mod tests;
