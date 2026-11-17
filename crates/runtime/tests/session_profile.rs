//! Which profile one chat is using, read from that chat's own session file
//! (profiles-editor T2a, L9's ruling).
//!
//! The channel exposes no read for a session's profile binding (L8), so the app
//! reads `gentle-pi.session-profile/v1` custom entries out of the chat's session
//! JSONL instead: the LAST profile-family entry decides, exactly as the record's
//! own replay contract says (`docs/session-profile-format.md:9-60`), and its
//! payloads are pinned here verbatim from that document.
//!
//! This is an APPROXIMATION and says so in the module it tests: gentle-pi
//! resolves the binding by replaying the session's active branch with a disk
//! corroboration rule, and none of that is reimplemented. What this reader does
//! is what the ruling asked for - the last entry's own kind and name - labelled
//! in the UI as an approximation.
//!
//! Every case is pure text in, one value out: no file, no clock, no shell.

use gentle_runtime::session_profile::{
    last_session_profile, last_session_profile_entry, SessionProfileBind, SessionProfileEntry,
    SessionProfileOrigin, CUSTOM_TYPE,
};

const FIXTURE: &str = include_str!("fixtures/session-profile.jsonl");

const HEADER: &str = r#"{"type":"session","version":3,"id":"s1","timestamp":"2026-11-17T10:00:00.000Z","cwd":"C:\\dev\\p"}"#;

/// The document's first payload (`docs/session-profile-format.md:11`): an
/// explicit choice, written by the screen's "Use in this session".
const DOC_USER_BIND: &str = r#"{"kind":"bind","origin":"user","name":"work","modelProfiles":{"worker":{"model":"provider/model"},"orchestrator":{"thinking":"high"}}}"#;

/// Its second payload (`:12`): the same choice with a startup state.
const DOC_USER_BIND_WITH_STARTUP: &str = r#"{"kind":"bind","origin":"user","name":"work","modelProfiles":{"worker":{"model":"provider/model"}},"startup":"none"}"#;

/// Its third payload (`:13`): a startup record, the layer the session started
/// from.
const DOC_LOCAL_BIND: &str = r#"{"kind":"bind","origin":"local","name":"team","modelProfiles":{"worker":{"model":"provider/model"},"orchestrator":{"model":"provider/model"}}}"#;

/// Its fourth payload (`:14`): the legacy clear.
const DOC_CLEAR: &str = r#"{"kind":"clear"}"#;

/// Its fifth payload (`:15`): the record of a session that started with no
/// profile in any layer.
const DOC_CLEAR_STARTUP_NONE: &str = r#"{"kind":"clear","startup":"none"}"#;

/// One session entry carrying `payload`, in the shape a session file writes: the
/// document's own field name is `data` (`lib/session-profile-persistence.ts:218`).
fn entry(custom_type: &str, payload: &str) -> String {
    format!(
        r#"{{"type":"custom","customType":"{custom_type}","data":{payload},"id":"c","parentId":null,"timestamp":"2026-11-17T10:00:00.000Z"}}"#
    )
}

/// One session: a header, then the given entries.
fn session(entries: &[String]) -> String {
    let mut text = String::from(HEADER);
    for entry in entries {
        text.push('\n');
        text.push_str(entry);
    }
    text.push('\n');
    text
}

fn bind(name: &str, origin: SessionProfileOrigin) -> SessionProfileBind {
    SessionProfileBind {
        name: name.to_owned(),
        origin,
    }
}

#[test]
fn the_last_documented_bind_entry_is_the_profile() {
    // The fixture pins three of the document's payloads as real entries, in
    // order, with a foreign custom entry between them.
    assert_eq!(CUSTOM_TYPE, "gentle-pi.session-profile/v1");
    assert_eq!(
        last_session_profile(FIXTURE),
        Some(bind("team", SessionProfileOrigin::Local)),
        "the last profile-family entry wins, not the first"
    );
    assert_eq!(
        last_session_profile_entry(FIXTURE),
        SessionProfileEntry::Bound(bind("team", SessionProfileOrigin::Local))
    );
}

#[test]
fn an_explicit_choice_is_the_user_origin() {
    let text = session(&[
        entry(CUSTOM_TYPE, DOC_USER_BIND),
        entry(CUSTOM_TYPE, DOC_USER_BIND_WITH_STARTUP),
    ]);
    assert_eq!(
        last_session_profile(&text),
        Some(bind("work", SessionProfileOrigin::User))
    );
}

#[test]
fn every_documented_origin_reads() {
    let cases = [
        ("user", SessionProfileOrigin::User),
        ("local", SessionProfileOrigin::Local),
        ("repo", SessionProfileOrigin::Repo),
        ("global", SessionProfileOrigin::Global),
    ];
    for (origin, expected) in cases {
        let payload = format!(
            r#"{{"kind":"bind","origin":"{origin}","name":"team","modelProfiles":{{"worker":{{"model":"provider/model"}}}}}}"#
        );
        let text = session(&[entry(CUSTOM_TYPE, &payload)]);
        assert_eq!(
            last_session_profile(&text),
            Some(bind("team", expected)),
            "the closed origin set is {origin}"
        );
    }
    // L8/L9's own record: a startup record keeps its own name.
    let text = session(&[entry(CUSTOM_TYPE, DOC_LOCAL_BIND)]);
    assert_eq!(
        last_session_profile(&text),
        Some(bind("team", SessionProfileOrigin::Local))
    );
}

#[test]
fn a_clear_after_a_bind_clears_it() {
    for clear in [DOC_CLEAR, DOC_CLEAR_STARTUP_NONE] {
        let text = session(&[entry(CUSTOM_TYPE, DOC_USER_BIND), entry(CUSTOM_TYPE, clear)]);
        assert_eq!(
            last_session_profile(&text),
            None,
            "a terminal clear leaves no profile to show: {clear}"
        );
        assert_eq!(
            last_session_profile_entry(&text),
            SessionProfileEntry::Cleared,
            "the clear is reported as itself, not as an absent entry"
        );
    }
}

#[test]
fn a_clear_before_a_bind_does_not_win() {
    let text = session(&[
        entry(CUSTOM_TYPE, DOC_CLEAR),
        entry(CUSTOM_TYPE, DOC_USER_BIND),
    ]);
    assert_eq!(
        last_session_profile(&text),
        Some(bind("work", SessionProfileOrigin::User)),
        "the newest entry decides, and this one is a bind"
    );
}

#[test]
fn a_session_with_no_profile_entry_has_none() {
    for text in [
        String::new(),
        "\n\n   \n".to_owned(),
        session(&[]),
        session(&[entry("gentle-pi.something-else/v1", DOC_USER_BIND)]),
        "not json at all\n{\"type\":\"custom\"\n".to_owned(),
    ] {
        assert_eq!(last_session_profile(&text), None, "{text:?}");
        assert_eq!(
            last_session_profile_entry(&text),
            SessionProfileEntry::Absent
        );
    }
}

#[test]
fn malformed_lines_are_skipped_never_fatal() {
    let text = format!(
        "{HEADER}\nnot json\n{{\"type\":\"custom\",\"customType\":{CUSTOM_TYPE}}}\n{}\n[]\nnull\n{}\n",
        entry(CUSTOM_TYPE, DOC_USER_BIND),
        entry(CUSTOM_TYPE, DOC_LOCAL_BIND),
    );
    assert_eq!(
        last_session_profile(&text),
        Some(bind("team", SessionProfileOrigin::Local)),
        "one broken line must not lose the profile the session recorded"
    );
}

#[test]
fn a_windows_line_ending_reads_the_same() {
    // Pi writes `\n`; a file that travelled through Windows tooling may not.
    let text = session(&[entry(CUSTOM_TYPE, DOC_USER_BIND)]).replace('\n', "\r\n");
    assert_eq!(
        last_session_profile(&text),
        Some(bind("work", SessionProfileOrigin::User))
    );
}

#[test]
fn an_unknown_kind_is_terminal_and_never_resurrects_an_earlier_bind() {
    // "The newest profile-family entry is terminal ... never revive an older
    // binding" (`docs/session-profile-format.md:41-53`).
    for kind in [
        r#"{"kind":"unbind"}"#,
        r#"{"kind":"clear-ish","name":"team"}"#,
        r#"{"name":"team"}"#,
    ] {
        let text = session(&[entry(CUSTOM_TYPE, DOC_USER_BIND), entry(CUSTOM_TYPE, kind)]);
        assert_eq!(
            last_session_profile(&text),
            None,
            "an entry this reader cannot use wins, and no earlier bind comes back: {kind}"
        );
        assert_eq!(
            last_session_profile_entry(&text),
            SessionProfileEntry::Invalid,
            "{kind} is invalid, not absent: a reader that showed the default would be guessing"
        );
    }
}

#[test]
fn a_bind_the_document_calls_invalid_is_invalid() {
    // "known invalid fields reject the whole binding"
    // (`docs/session-profile-format.md:36-38`).
    for payload in [
        // no name
        r#"{"kind":"bind","origin":"user","modelProfiles":{}}"#,
        // an empty name is not a profile name
        r#"{"kind":"bind","origin":"user","name":"","modelProfiles":{}}"#,
        // an origin outside the closed set
        r#"{"kind":"bind","origin":"elsewhere","name":"team","modelProfiles":{}}"#,
        // no origin
        r#"{"kind":"bind","name":"team","modelProfiles":{}}"#,
        // no snapshot at all
        r#"{"kind":"bind","origin":"user","name":"team"}"#,
    ] {
        let text = session(&[
            entry(CUSTOM_TYPE, DOC_USER_BIND),
            entry(CUSTOM_TYPE, payload),
        ]);
        assert_eq!(
            last_session_profile_entry(&text),
            SessionProfileEntry::Invalid,
            "{payload}"
        );
        assert_eq!(last_session_profile(&text), None);
    }
}

#[test]
fn an_entry_of_the_family_without_a_payload_object_is_invalid() {
    let text = format!(
        "{HEADER}\n{}\n{}\n{}\n",
        entry(CUSTOM_TYPE, DOC_USER_BIND),
        r#"{"type":"custom","customType":"gentle-pi.session-profile/v1","id":"c"}"#,
        r#"{"type":"custom","customType":"gentle-pi.session-profile/v1","data":"clear","id":"d"}"#,
    );
    assert_eq!(
        last_session_profile_entry(&text),
        SessionProfileEntry::Invalid
    );
}

#[test]
fn a_bind_of_another_version_is_not_this_readers_family() {
    let text = session(&[
        entry(CUSTOM_TYPE, DOC_USER_BIND),
        entry("gentle-pi.session-profile/v2", DOC_LOCAL_BIND),
    ]);
    assert_eq!(
        last_session_profile(&text),
        Some(bind("work", SessionProfileOrigin::User)),
        "an entry of another version carries no meaning for this reader"
    );
    assert_eq!(last_session_profile("{\"customType\":5}"), None);
}

#[test]
fn unknown_extra_keys_carry_no_meaning() {
    let payload = r#"{"kind":"bind","origin":"user","name":"work","modelProfiles":{},"future":{"x":1},"label":"Work"}"#;
    let entry = format!(
        r#"{{"type":"custom","customType":"{CUSTOM_TYPE}","data":{payload},"id":"c","futureField":true}}"#
    );
    let text = session(&[entry]);
    assert_eq!(
        last_session_profile(&text),
        Some(bind("work", SessionProfileOrigin::User)),
        "an extra member is ignored, exactly as the v1 codec ignores it"
    );
}
