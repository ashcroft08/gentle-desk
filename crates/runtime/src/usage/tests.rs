//! The usage rules' tests: the locator (entry and checkout), the parser (the
//! real document, the refusals, no-data and a null reset), and the reset label.

use std::path::{Path, PathBuf};

use super::*;

const SEC: u64 = 1000;
const MIN: u64 = 60 * SEC;
const HOUR: u64 = 60 * MIN;
const DAY: u64 = 24 * HOUR;
const NOW: u64 = 1_790_000_000_000;

const HOME: &str = "C:/Users/u/.pi/agent";

/// The shell's settings as this machine writes them, reduced to the one entry
/// that matters (the real file lists nine packages).
const SETTINGS: &str = r#"{
    "defaultProvider": "commandcode",
    "packages": ["npm:gentle-pi", "git:github.com/ashcroft08/pi-usage-sources"]
}"#;

/// The document the package's headless mode printed on this machine, verbatim:
/// one no-data provider, two ok providers with plans, and a provider without one.
const DOCUMENT: &str = r#"{"schema":"pi-usage-sources/usage/v1","generatedAt":1794637790033,"sources":[{"provider":"opencode-go","status":"no-data","fetchedAt":1794637790033,"limits":[]},{"provider":"antigravity","status":"ok","fetchedAt":1794637790033,"limits":[{"name":"Gemini Models","limitReached":false,"windows":[{"label":"week","usedPercent":0,"resetAt":1795242599000,"windowSeconds":604800}]},{"name":"Claude and GPT models","limitReached":false,"windows":[{"label":"week","usedPercent":0,"resetAt":1795242599000,"windowSeconds":604800}]}]},{"provider":"commandcode","status":"ok","plan":"individual goat","fetchedAt":1794637790033,"limits":[{"name":"command code","limitReached":false,"windows":[{"label":"5h","usedPercent":4.6771898400000005,"resetAt":1794644915319,"windowSeconds":18000},{"label":"week","usedPercent":88.272975324,"resetAt":1794727272498,"windowSeconds":604800}]},{"name":"credits","limitReached":false,"windows":[{"label":"billing","usedPercent":44.13765581735515,"resetAt":1796714153000,"windowSeconds":2592000}]}]},{"provider":"claude-bridge","status":"ok","plan":"pro","fetchedAt":1794633348955,"limits":[{"name":"claude","limitReached":false,"windows":[{"label":"5h","usedPercent":21,"resetAt":1794644400000,"windowSeconds":18000},{"label":"week","usedPercent":96,"resetAt":1794794400000,"windowSeconds":604800}]}]}]}"#;

fn home() -> PathBuf {
    PathBuf::from(HOME)
}

#[test]
fn the_entry_is_the_one_that_names_the_usage_sources() {
    assert_eq!(
        package_entry(Some(SETTINGS)).as_deref(),
        Some("git:github.com/ashcroft08/pi-usage-sources")
    );
    // A local checkout is named the same way, the last segment being the package.
    assert_eq!(
        package_entry(Some(
            r#"{"packages":["npm:gentle-pi","C:/dev/pi-usage-sources"]}"#
        ))
        .as_deref(),
        Some("C:/dev/pi-usage-sources")
    );
    // An object entry names its source in a field.
    assert_eq!(
        package_entry(Some(
            r#"{"packages":[{"source":"git:github.com/ashcroft08/pi-usage-sources"}]}"#
        ))
        .as_deref(),
        Some("git:github.com/ashcroft08/pi-usage-sources")
    );
}

#[test]
fn no_entry_no_script() {
    assert_eq!(package_entry(None), None);
    assert_eq!(package_entry(Some("")), None);
    assert_eq!(package_entry(Some("{not json")), None);
    assert_eq!(package_entry(Some(r#"{"defaultProvider":"x"}"#)), None);
    assert_eq!(
        package_entry(Some(
            r#"{"packages":["npm:gentle-pi","npm:gentle-engram"]}"#
        )),
        None
    );
    // A package whose name merely ENDS with the same letters is another package.
    assert_eq!(
        package_entry(Some(
            r#"{"packages":["git:host/owner/not-pi-usage-sources"]}"#
        )),
        None
    );
    assert_eq!(
        package_entry(Some(r#"{"packages":["npm:pi-usage-sources"]}"#)),
        None,
        "an npm install has no checkout under the agent home"
    );
    assert_eq!(script(None, &home()), None);
    assert_eq!(script(Some(r#"{"packages":[]}"#), &home()), None);
}

#[test]
fn a_git_entry_hangs_off_the_agent_homes_git_folder() {
    assert_eq!(
        checkout("git:github.com/ashcroft08/pi-usage-sources", &home()),
        Some(home().join("git/github.com/ashcroft08/pi-usage-sources"))
    );
    // A trailing `.git` names the same repository.
    assert_eq!(
        checkout("git:github.com/ashcroft08/pi-usage-sources.git", &home()),
        Some(home().join("git/github.com/ashcroft08/pi-usage-sources"))
    );
}

#[test]
fn a_local_entry_is_the_path_it_spells() {
    assert_eq!(
        checkout("C:/dev/pi-usage-sources", &home()),
        Some(PathBuf::from("C:/dev/pi-usage-sources"))
    );
    assert_eq!(
        checkout("dev/pi-usage-sources", &home()),
        Some(home().join("dev/pi-usage-sources")),
        "a relative path is relative to the agent home"
    );
    assert_eq!(checkout("npm:pi-usage-sources", &home()), None);
}

#[test]
fn an_address_that_would_leave_the_git_folder_names_nothing() {
    assert_eq!(checkout("git:github.com/../../etc", &home()), None);
    assert_eq!(checkout("git:", &home()), None);
    assert_eq!(checkout("git:github.com//pi-usage-sources", &home()), None);
}

#[test]
fn the_script_sits_in_the_checkout() {
    assert_eq!(
        script(Some(SETTINGS), &home()),
        Some(
            Path::new(HOME)
                .join("git/github.com/ashcroft08/pi-usage-sources")
                .join("usage-json.ts")
        )
    );
}

#[test]
fn the_real_document_reads_as_its_four_providers() {
    let document = parse(DOCUMENT).expect("the real document is this reader's");

    let names: Vec<&str> = document
        .providers
        .iter()
        .map(|provider| provider.provider.as_str())
        .collect();
    assert_eq!(
        names,
        vec!["opencode-go", "antigravity", "commandcode", "claude-bridge"]
    );

    let commandcode = &document.providers[2];
    assert_eq!(commandcode.status, ProviderStatus::Ok);
    assert_eq!(commandcode.plan.as_deref(), Some("individual goat"));
    assert_eq!(commandcode.limits.len(), 2);
    assert_eq!(commandcode.limits[0].name, "command code");
    assert!(!commandcode.limits[0].limit_reached);
    assert_eq!(commandcode.limits[0].windows.len(), 2);
    assert_eq!(commandcode.limits[0].windows[0].label, "5h");
    assert!((commandcode.limits[0].windows[0].used_percent - 4.6771898400000005).abs() < 1e-12);
    assert_eq!(
        commandcode.limits[0].windows[1].reset_at,
        Some(1_794_727_272_498)
    );

    // A provider that reports no plan keeps none, never an empty word.
    assert_eq!(document.providers[1].plan, None);
    assert_eq!(document.providers[3].plan.as_deref(), Some("pro"));
}

#[test]
fn a_no_data_provider_is_no_data_and_never_zero() {
    let document = parse(DOCUMENT).expect("the real document is this reader's");
    let opencode = &document.providers[0];

    assert_eq!(opencode.status, ProviderStatus::NoData);
    assert!(
        opencode.limits.is_empty(),
        "no-data carries no window to read as 0%"
    );
}

#[test]
fn a_reset_time_that_is_null_is_no_countdown() {
    let document = parse(
        r#"{"schema":"pi-usage-sources/usage/v1","generatedAt":1,"sources":[
            {"provider":"p","status":"ok","fetchedAt":1,"limits":[
                {"name":"l","limitReached":false,"windows":[
                    {"label":"week","usedPercent":12.5,"resetAt":null}]}]}]}"#,
    )
    .expect("a null reset is a real answer");

    let window = &document.providers[0].limits[0].windows[0];
    assert_eq!(window.reset_at, None);
    assert_eq!(window.used_percent, 12.5);
    assert_eq!(reset_label(NOW, window.reset_at), None);
}

#[test]
fn a_failed_provider_carries_the_reason_the_package_redacted() {
    let document = parse(
        r#"{"schema":"pi-usage-sources/usage/v1","generatedAt":1,"sources":[
            {"provider":"claude-bridge","status":"failed","error":"timed out after 10000ms",
             "fetchedAt":1,"limits":[]}]}"#,
    )
    .expect("a failed provider is an answer, not a refusal");

    assert_eq!(document.providers[0].status, ProviderStatus::Failed);
    assert_eq!(
        document.providers[0].error.as_deref(),
        Some("timed out after 10000ms")
    );
    assert!(document.providers[0].limits.is_empty());
}

#[test]
fn a_document_of_another_schema_is_refused_and_named() {
    let error = parse(r#"{"schema":"someone/else","generatedAt":1,"sources":[]}"#)
        .expect_err("another schema is not this pane's document");
    assert!(error.contains("someone/else"), "{error}");
}

#[test]
fn a_document_of_another_shape_is_refused() {
    for body in [
        "not json",
        "[]",
        "null",
        r#"{"generatedAt":1,"sources":[]}"#,
        r#"{"schema":"pi-usage-sources/usage/v1"}"#,
        r#"{"schema":"pi-usage-sources/usage/v1","sources":{}}"#,
    ] {
        assert!(
            parse(body).is_err(),
            "this shape is refused, never guessed: {body}"
        );
    }
}

#[test]
fn a_provider_of_another_shape_is_refused() {
    let head = r#"{"schema":"pi-usage-sources/usage/v1","generatedAt":1,"sources":["#;
    let tail = "]}";
    for source in [
        // No name, or a name that is only spaces.
        r#"{"status":"no-data","limits":[]}"#,
        r#"{"provider":"  ","status":"no-data","limits":[]}"#,
        // A status this reader does not know, or none at all.
        r#"{"provider":"p","status":"mystery","limits":[]}"#,
        r#"{"provider":"p","limits":[]}"#,
        // No limit list at all.
        r#"{"provider":"p","status":"ok"}"#,
        // A limit without a name or a reach mark.
        r#"{"provider":"p","status":"ok","limits":[{"limitReached":false,"windows":[]}]}"#,
        r#"{"provider":"p","status":"ok","limits":[{"name":"l","windows":[]}]}"#,
        // A window without a label, without a percentage, or with a reset time
        // that is not a number.
        r#"{"provider":"p","status":"ok","limits":[{"name":"l","limitReached":false,"windows":[{"usedPercent":1,"resetAt":null}]}]}"#,
        r#"{"provider":"p","status":"ok","limits":[{"name":"l","limitReached":false,"windows":[{"label":"5h","resetAt":null}]}]}"#,
        r#"{"provider":"p","status":"ok","limits":[{"name":"l","limitReached":false,"windows":[{"label":"5h","usedPercent":1,"resetAt":"soon"}]}]}"#,
    ] {
        let body = format!("{head}{source}{tail}");
        assert!(
            parse(&body).is_err(),
            "this provider is refused, never guessed: {source}"
        );
    }
}

#[test]
fn the_reset_label_counts_up_from_now() {
    assert_eq!(
        reset_label(NOW, Some(NOW + 4 * HOUR)).as_deref(),
        Some("resets in 4h")
    );
    assert_eq!(
        reset_label(NOW, Some(NOW + 6 * DAY + 3 * HOUR)).as_deref(),
        Some("resets in 7d")
    );
    // Under a minute still counts as a minute: "resets in 0m" would be a lie.
    assert_eq!(
        reset_label(NOW, Some(NOW + 30 * SEC)).as_deref(),
        Some("resets in 1m")
    );
    // A reset that already happened is not a countdown into the past.
    assert_eq!(reset_label(NOW, Some(NOW)).as_deref(), Some("resets now"));
    assert_eq!(
        reset_label(NOW, Some(NOW - HOUR)).as_deref(),
        Some("resets now")
    );
    assert_eq!(reset_label(NOW, None), None);
}
