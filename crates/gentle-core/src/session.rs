//! Summaries of pi session files.
//!
//! A session is an append-only JSONL file: a `session` header line followed by
//! entries. [`SessionScanner`] folds those lines one at a time into a
//! [`SessionSummary`], so a caller that already scanned a file can feed only
//! the bytes appended since (the incremental index in `gentle-backend` does).
//! Pure: no file system, no clock.

/// What a session list shows for one file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSummary {
    pub id: String,
    pub cwd: String,
    /// Header timestamp, epoch milliseconds.
    pub created_ms: Option<u64>,
    /// Latest `session_info` name, trimmed; `None` if never set or cleared.
    pub name: Option<String>,
    /// Text of the first user message that has any.
    pub first_message: Option<String>,
    /// Every `message` entry, whatever its role.
    pub message_count: u32,
    /// Latest user/assistant message time, epoch milliseconds.
    pub last_activity_ms: Option<u64>,
}

/// Incremental line-by-line builder of a [`SessionSummary`].
#[derive(Debug, Clone, Default)]
pub struct SessionScanner {
    summary: Option<SessionSummary>,
    /// The first parsable entry was not a `session` header: not a session file.
    rejected: bool,
}

impl SessionScanner {
    pub fn new() -> Self {
        Self::default()
    }

    /// Folds one line in. Blank and malformed lines are skipped, like pi does.
    pub fn push_line(&mut self, line: &str) {
        if self.rejected || line.trim().is_empty() {
            return;
        }
        let Ok(serde_json::Value::Object(entry)) = serde_json::from_str(line) else {
            return;
        };
        let kind = entry.get("type").and_then(|v| v.as_str());
        let Some(summary) = self.summary.as_mut() else {
            if kind == Some("session") {
                self.summary = Some(SessionSummary {
                    id: str_field(&entry, "id").unwrap_or_default().to_owned(),
                    cwd: str_field(&entry, "cwd").unwrap_or_default().to_owned(),
                    created_ms: str_field(&entry, "timestamp").and_then(parse_iso_ms),
                    name: None,
                    first_message: None,
                    message_count: 0,
                    last_activity_ms: None,
                });
            } else {
                self.rejected = true;
            }
            return;
        };
        match kind {
            Some("session_info") => {
                summary.name = str_field(&entry, "name")
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(str::to_owned);
            }
            Some("message") => fold_message(summary, &entry),
            _ => {}
        }
    }

    /// The summary so far; `None` until a valid header was seen.
    pub fn summary(&self) -> Option<SessionSummary> {
        self.summary.clone()
    }
}

fn fold_message(summary: &mut SessionSummary, entry: &serde_json::Map<String, serde_json::Value>) {
    summary.message_count += 1;
    let Some(message) = entry.get("message").and_then(|m| m.as_object()) else {
        return;
    };
    let role = message.get("role").and_then(|r| r.as_str());
    if !message.contains_key("content") || !matches!(role, Some("user" | "assistant")) {
        return;
    }
    // The message's own epoch-ms timestamp beats the entry's ISO one.
    let activity = message
        .get("timestamp")
        .and_then(|t| t.as_u64())
        .or_else(|| str_field(entry, "timestamp").and_then(parse_iso_ms));
    if let Some(activity) = activity {
        summary.last_activity_ms = Some(
            summary
                .last_activity_ms
                .map_or(activity, |a| a.max(activity)),
        );
    }
    if role == Some("user") && summary.first_message.is_none() {
        let text = message_text(&message["content"]);
        if !text.is_empty() {
            summary.first_message = Some(text);
        }
    }
}

/// A string content as is; a block list as its `text` blocks joined by a space.
fn message_text(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Array(blocks) => blocks
            .iter()
            .filter(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))
            .filter_map(|b| b.get("text").and_then(|t| t.as_str()))
            .collect::<Vec<_>>()
            .join(" "),
        _ => String::new(),
    }
}

fn str_field<'a>(
    entry: &'a serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Option<&'a str> {
    entry.get(key).and_then(|v| v.as_str())
}

/// Parses `YYYY-MM-DDTHH:MM:SS[.fff]Z` (UTC) into epoch milliseconds.
pub fn parse_iso_ms(text: &str) -> Option<u64> {
    let bytes = text.as_bytes();
    if bytes.len() < 20
        || !text.ends_with('Z')
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return None;
    }
    let number = |range: std::ops::Range<usize>| text.get(range)?.parse::<u32>().ok();
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second) = (number(11..13)?, number(14..16)?, number(17..19)?);
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    let fraction = &text[19..text.len() - 1];
    let millis = match fraction.strip_prefix('.') {
        None if fraction.is_empty() => 0,
        Some(digits) if !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()) => {
            // Milliseconds: pad or truncate the fraction to three digits.
            format!("{digits:0<3}")[..3].parse::<u64>().ok()?
        }
        _ => return None,
    };
    let days = days_from_civil(i64::from(year), month, day);
    let seconds = days * 86_400 + i64::from(hour * 3600 + minute * 60 + second);
    u64::try_from(seconds).ok().map(|s| s * 1000 + millis)
}

/// Days since 1970-01-01 of a proleptic Gregorian date (Howard Hinnant's algorithm).
fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = y.div_euclid(400);
    let year_of_era = y - era * 400;
    let month_index = i64::from((month + 9) % 12);
    let day_of_year = (153 * month_index + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}
