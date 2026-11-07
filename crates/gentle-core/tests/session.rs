//! Session summaries, checked against the semantics of pi's own listing
//! (`buildSessionInfo` in pi's session-manager).

use gentle_core::session::{parse_iso_ms, SessionScanner, SessionSummary};

const HEADER: &str = r#"{"type":"session","version":3,"id":"abc","timestamp":"2026-11-07T02:36:21.532Z","cwd":"C:\\dev\\proj"}"#;

fn scan(lines: &[&str]) -> Option<SessionSummary> {
    let mut scanner = SessionScanner::new();
    for line in lines {
        scanner.push_line(line);
    }
    scanner.summary()
}

fn message(role: &str, content: &str, timestamp: &str) -> String {
    format!(
        r#"{{"type":"message","id":"m","timestamp":"{timestamp}","message":{{"role":"{role}","content":{content}}}}}"#
    )
}

#[test]
fn the_header_gives_id_cwd_and_creation_time() {
    let summary = scan(&[HEADER]).unwrap();
    assert_eq!(summary.id, "abc");
    assert_eq!(summary.cwd, "C:\\dev\\proj");
    assert_eq!(summary.created_ms, Some(1_794_018_981_532));
    assert_eq!(summary.message_count, 0);
    assert_eq!(summary.first_message, None);
    assert_eq!(summary.name, None);
    assert_eq!(summary.last_activity_ms, None);
}

#[test]
fn a_file_that_does_not_start_with_a_session_header_is_not_a_session() {
    let first = message("user", r#""hi""#, "2026-11-07T02:37:00.000Z");
    assert_eq!(scan(&[&first, HEADER]), None);
    assert_eq!(scan(&[]), None);
}

#[test]
fn blank_and_malformed_lines_are_skipped_wherever_they_are() {
    let user = message("user", r#""hola""#, "2026-11-07T02:37:00.000Z");
    let summary = scan(&["", "{broken", HEADER, "   ", "not json", &user]).unwrap();
    assert_eq!(summary.message_count, 1);
}

#[test]
fn messages_are_counted_including_ones_without_displayable_text() {
    let user = message("user", r#""hola""#, "2026-11-07T02:37:00.000Z");
    let system = message("system", r#""""#, "2026-11-07T02:36:30.000Z");
    let tool = message(
        "toolResult",
        r#"[{"type":"text","text":"out"}]"#,
        "2026-11-07T02:37:05.000Z",
    );
    let summary = scan(&[HEADER, &system, &user, &tool]).unwrap();
    assert_eq!(summary.message_count, 3);
}

#[test]
fn the_first_message_is_the_first_user_text_joined_from_text_blocks() {
    let system = message("system", r#""You are...""#, "2026-11-07T02:36:30.000Z");
    let assistant = message(
        "assistant",
        r#"[{"type":"text","text":"hi"}]"#,
        "2026-11-07T02:36:40.000Z",
    );
    let user = message(
        "user",
        r#"[{"type":"text","text":"uno"},{"type":"image"},{"type":"text","text":"dos"}]"#,
        "2026-11-07T02:37:00.000Z",
    );
    let later = message("user", r#""otro""#, "2026-11-07T02:38:00.000Z");
    let summary = scan(&[HEADER, &system, &assistant, &user, &later]).unwrap();
    assert_eq!(summary.first_message.as_deref(), Some("uno dos"));
}

#[test]
fn a_user_message_with_no_text_is_not_the_first_message() {
    let empty = message("user", r#"[{"type":"image"}]"#, "2026-11-07T02:36:40.000Z");
    let real = message("user", r#""de verdad""#, "2026-11-07T02:37:00.000Z");
    let summary = scan(&[HEADER, &empty, &real]).unwrap();
    assert_eq!(summary.first_message.as_deref(), Some("de verdad"));
}

#[test]
fn the_latest_session_info_name_wins_and_a_blank_one_clears_it() {
    let named = r#"{"type":"session_info","id":"1","timestamp":"2026-11-07T02:40:00.000Z","name":"  Primer nombre  "}"#;
    let renamed = r#"{"type":"session_info","id":"2","timestamp":"2026-11-07T02:41:00.000Z","name":"Segundo"}"#;
    let cleared =
        r#"{"type":"session_info","id":"3","timestamp":"2026-11-07T02:42:00.000Z","name":"  "}"#;
    assert_eq!(
        scan(&[HEADER, named]).unwrap().name.as_deref(),
        Some("Primer nombre")
    );
    assert_eq!(
        scan(&[HEADER, named, renamed]).unwrap().name.as_deref(),
        Some("Segundo")
    );
    assert_eq!(scan(&[HEADER, named, renamed, cleared]).unwrap().name, None);
}

#[test]
fn activity_is_the_latest_user_or_assistant_time_preferring_the_message_timestamp() {
    let user = message("user", r#""a""#, "2026-11-07T02:37:00.000Z");
    // The message's own epoch-ms timestamp wins over the entry's ISO one.
    let assistant = r#"{"type":"message","id":"m","timestamp":"2026-11-07T02:38:00.000Z","message":{"role":"assistant","content":"b","timestamp":1794020000000}}"#;
    // Tool results and system messages never count as activity.
    let tool = message("toolResult", r#""x""#, "2026-11-07T09:00:00.000Z");
    let summary = scan(&[HEADER, &user, assistant, &tool]).unwrap();
    assert_eq!(summary.last_activity_ms, Some(1_794_020_000_000));
}

#[test]
fn activity_never_goes_backwards_when_entries_are_out_of_order() {
    let late = message("user", r#""a""#, "2026-11-07T05:00:00.000Z");
    let early = message("assistant", r#""b""#, "2026-11-07T03:00:00.000Z");
    let summary = scan(&[HEADER, &late, &early]).unwrap();
    assert_eq!(
        summary.last_activity_ms,
        parse_iso_ms("2026-11-07T05:00:00.000Z")
    );
}

#[test]
fn summaries_can_be_built_incrementally_in_chunks() {
    let user = message("user", r#""hola""#, "2026-11-07T02:37:00.000Z");
    let mut scanner = SessionScanner::new();
    scanner.push_line(HEADER);
    let partial = scanner.summary().unwrap();
    assert_eq!(partial.message_count, 0);
    scanner.push_line(&user);
    assert_eq!(scanner.summary().unwrap().message_count, 1);
    // The earlier snapshot is a value: later lines do not change it.
    assert_eq!(partial.message_count, 0);
}

#[test]
fn iso_timestamps_parse_to_epoch_milliseconds() {
    assert_eq!(parse_iso_ms("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(parse_iso_ms("1970-01-01T00:00:01.5Z"), Some(1500));
    assert_eq!(
        parse_iso_ms("2000-02-29T12:00:00.000Z"),
        Some(951_825_600_000)
    );
    assert_eq!(
        parse_iso_ms("2026-11-07T02:36:21.532Z"),
        Some(1_794_018_981_532)
    );
    assert_eq!(
        parse_iso_ms("2024-03-01T00:00:00Z"),
        Some(1_709_251_200_000)
    );
}

/// A persisted index stores the scanner itself, so its state must survive
/// JSON and resume reading where it stopped.
#[test]
fn a_scanner_resumes_after_a_json_round_trip() {
    let user = message("user", r#""hola""#, "2026-11-07T02:37:00.000Z");
    let mut cut = SessionScanner::new();
    cut.push_line(HEADER);
    let json = serde_json::to_string(&cut).unwrap();
    let mut resumed: SessionScanner = serde_json::from_str(&json).unwrap();
    resumed.push_line(&user);
    assert_eq!(resumed.summary(), scan(&[HEADER, &user]));
    assert_eq!(
        resumed.summary().unwrap().first_message.as_deref(),
        Some("hola")
    );
}

#[test]
fn a_rejected_scanner_is_still_rejected_after_a_json_round_trip() {
    let user = message("user", r#""hola""#, "2026-11-07T02:37:00.000Z");
    let mut scanner = SessionScanner::new();
    scanner.push_line(&user);
    let json = serde_json::to_string(&scanner).unwrap();
    let mut resumed: SessionScanner = serde_json::from_str(&json).unwrap();
    resumed.push_line(HEADER);
    assert_eq!(resumed.summary(), None);
}

#[test]
fn a_summary_survives_a_json_round_trip() {
    let user = message("user", r#""hola""#, "2026-11-07T02:37:00.000Z");
    let summary = scan(&[HEADER, &user]).unwrap();
    let json = serde_json::to_string(&summary).unwrap();
    assert_eq!(
        serde_json::from_str::<SessionSummary>(&json).unwrap(),
        summary
    );
}

#[test]
fn malformed_timestamps_are_none() {
    for bad in [
        "",
        "yesterday",
        "2026-13-01T00:00:00Z",
        "2026-11-32T00:00:00Z",
        "2026-11-07 02:36:21Z",
        "2026-11-07T25:00:00Z",
    ] {
        assert_eq!(parse_iso_ms(bad), None, "{bad}");
    }
}
