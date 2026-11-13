//! Where the shell's authentication stands (0.1.12, S1, T3): the provider names
//! an `auth.json` holds, and what `gentle-shell auth check --json` says about one
//! of them.
//!
//! Names only, never a secret. The file is read for its top-level KEYS and every
//! value is dropped by the parser, so no token, refresh token or account id can
//! travel to the view even by accident; a test pins that. The per-provider
//! status comes from the shell's own check, run with `--no-refresh` so the desk
//! never makes pi refresh an OAuth token behind the terminal's back.

use std::collections::BTreeSet;

use serde_json::Value;

/// What one provider's check came to, in the four states the shell reports
/// (`{status:"ready"}` exit 0, `{status:"not_ready"}` exit 1, and the two error
/// states of exit 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthStatus {
    Ready,
    NotReady,
    Invalid,
    Error,
}

/// One provider's check: its state and, when the shell said why, its reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderCheck {
    pub status: AuthStatus,
    /// The shell's own `reason`, verbatim. Never a credential: the check reports
    /// a problem, not the material behind it.
    pub reason: Option<String>,
}

/// The provider names an `auth.json` holds: its top-level keys, sorted. Every
/// value is dropped here, so nothing that identifies an account or holds a
/// token can reach the view.
///
/// A file that is not a JSON object, or no file at all (`None`), holds no names:
/// an unreadable auth store is an empty list, never a guess.
pub fn provider_names(text: Option<&str>) -> Vec<String> {
    let Some(text) = text else {
        return Vec::new();
    };
    let Ok(Value::Object(object)) = serde_json::from_str::<Value>(text) else {
        return Vec::new();
    };
    let names: BTreeSet<String> = object
        .keys()
        .filter(|name| !name.trim().is_empty())
        .cloned()
        .collect();
    names.into_iter().collect()
}

/// One `gentle-shell auth check --provider <id> --json --no-refresh` answer,
/// from the exit code and the single JSON line it printed.
///
/// The exit code is what the shell's own contract keys on (0 ready, 1 not ready,
/// 2 invalid or error); the line names the state and the reason. When the line
/// carries a status this reader knows, that status is what counts - so a shell
/// that answers `error` on exit 2 is reported as an error and not as an invalid
/// credential - and a line that cannot be read leaves the exit code's state with
/// no reason (`unknown`), never an invented one.
pub fn check(exit_code: i32, line: &str) -> ProviderCheck {
    let parsed = serde_json::from_str::<Value>(line)
        .ok()
        .filter(Value::is_object);
    let status = parsed
        .as_ref()
        .and_then(|value| value.get("status"))
        .and_then(Value::as_str)
        .and_then(status_of);
    let reason = parsed
        .as_ref()
        .and_then(|value| value.get("reason"))
        .and_then(Value::as_str)
        .filter(|reason| !reason.trim().is_empty())
        .map(str::to_owned);
    match (status, exit_code) {
        (Some(status), _) => ProviderCheck { status, reason },
        (None, 0) => ProviderCheck {
            status: AuthStatus::Ready,
            reason: None,
        },
        (None, 1) => ProviderCheck {
            status: AuthStatus::NotReady,
            reason: None,
        },
        (None, _) => ProviderCheck {
            status: AuthStatus::Error,
            // Nothing readable came back: the code is the only fact left, and a
            // code is not a sentence.
            reason: None,
        },
    }
}

/// The four states the shell names, and nothing else.
fn status_of(name: &str) -> Option<AuthStatus> {
    match name {
        "ready" => Some(AuthStatus::Ready),
        "not_ready" => Some(AuthStatus::NotReady),
        "invalid" => Some(AuthStatus::Invalid),
        "error" => Some(AuthStatus::Error),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_are_the_keys_and_nothing_of_the_values() {
        // What a real auth.json looks like, with the material reduced but the
        // shapes intact: only the provider names may come back.
        let auth = r#"{
            "anthropic": {"type": "oauth", "access": "sk-ant-oat-SECRET", "refresh": "rt-SECRET"},
            "openai-codex": {"type": "api_key", "key": "sk-SECRET", "accountId": "user@example.test"},
            "commandcode": {"type": "oauth", "access": "cc-SECRET"}
        }"#;

        let names = provider_names(Some(auth));

        assert_eq!(names, vec!["anthropic", "commandcode", "openai-codex"]);
        let carried = format!("{names:?}");
        for secret in ["SECRET", "user@example.test", "sk-ant", "rt-", "accountId"] {
            assert!(
                !carried.contains(secret),
                "the parser kept {secret}: {carried}"
            );
        }
    }

    #[test]
    fn an_empty_or_unreadable_store_holds_no_names() {
        assert_eq!(provider_names(None), Vec::<String>::new());
        assert_eq!(provider_names(Some("")), Vec::<String>::new());
        assert_eq!(provider_names(Some("{not json")), Vec::<String>::new());
        assert_eq!(provider_names(Some("[1, 2]")), Vec::<String>::new());
        assert_eq!(provider_names(Some("null")), Vec::<String>::new());
    }

    #[test]
    fn a_ready_check_is_the_exit_zero_line() {
        assert_eq!(
            check(
                0,
                r#"{"status":"ready","provider":"anthropic","authType":"oauth"}"#
            ),
            ProviderCheck {
                status: AuthStatus::Ready,
                reason: None
            }
        );
    }

    #[test]
    fn a_not_ready_check_carries_the_shells_reason() {
        assert_eq!(
            check(
                1,
                r#"{"status":"not_ready","provider":"anthropic","reason":"OAuth token expired"}"#
            ),
            ProviderCheck {
                status: AuthStatus::NotReady,
                reason: Some("OAuth token expired".to_owned())
            }
        );
    }

    #[test]
    fn the_two_error_states_of_exit_two_stay_apart() {
        assert_eq!(
            check(
                2,
                r#"{"status":"invalid","provider":"x","reason":"bad key"}"#
            )
            .status,
            AuthStatus::Invalid
        );
        assert_eq!(
            check(
                2,
                r#"{"status":"error","provider":"x","reason":"no network"}"#
            )
            .status,
            AuthStatus::Error
        );
    }

    #[test]
    fn a_line_that_cannot_be_read_falls_back_to_the_exit_code() {
        assert_eq!(check(0, "").status, AuthStatus::Ready);
        assert_eq!(check(1, "not json").status, AuthStatus::NotReady);
        assert_eq!(check(2, "not json").status, AuthStatus::Error);
        assert_eq!(check(3, "not json").status, AuthStatus::Error);
        // A status this reader does not know is no status: the code decides.
        assert_eq!(
            check(1, r#"{"status":"mystery","reason":"why"}"#).status,
            AuthStatus::NotReady
        );
    }

    #[test]
    fn nothing_but_the_status_and_the_reason_ever_comes_back() {
        // A line that (somehow) carried material still yields only the two
        // fields this view draws.
        let report = check(
            1,
            r#"{"status":"not_ready","provider":"p","reason":"needs login","access":"sk-SECRET"}"#,
        );
        let carried = format!("{report:?}");
        assert!(carried.contains("needs login"));
        assert!(!carried.contains("SECRET"), "{carried}");
    }
}
