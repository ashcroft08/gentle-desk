//! The Account pane's slice (0.1.12, T3): the provider names the shell's auth
//! store holds, and what the shell's own check says about each of them.
//!
//! Names only. The store is read for its top-level keys
//! ([`gentle_runtime::auth::provider_names`] drops every value), and each check
//! reports a state and a reason, so no key, token or account identity can reach
//! the page.

use std::path::Path;

use gentle_runtime::auth::{self as rule, AuthStatus};

use super::{FileRead, ProviderProbe};
use crate::contract::{AuthProviderView, AuthStatusView, AuthView, ProviderStatusView};

/// Reads `path` for the provider names and answers what the shell said about
/// each one, in name order.
///
/// A store that is not there is a ready pane with no providers - nobody has
/// connected one on this machine - and a store that cannot be read says so.
/// A check that could not run at all is that provider's `Error`, never a silent
/// absence: a provider the page lists is always a provider it says something
/// about.
pub fn view(read: &FileRead, path: Option<&Path>, probe: &ProviderProbe) -> AuthView {
    let Some(path) = path else {
        return AuthView {
            status: AuthStatusView::Failed(
                "the shell's agent folder could not be found on this machine".to_owned(),
            ),
            providers: Vec::new(),
        };
    };
    let text = match read(path) {
        Ok(text) => text,
        Err(reason) => {
            return AuthView {
                status: AuthStatusView::Failed(reason),
                providers: Vec::new(),
            };
        }
    };
    let names = rule::provider_names(text.as_deref());
    AuthView {
        status: AuthStatusView::Ready,
        providers: names
            .iter()
            .map(|name| provider_view(name, probe))
            .collect(),
    }
}

/// One provider and its check.
fn provider_view(name: &str, probe: &ProviderProbe) -> AuthProviderView {
    match probe(name) {
        Ok(answer) => {
            let check = rule::check(answer.exit_code, &answer.stdout);
            AuthProviderView {
                name: name.to_owned(),
                status: status_view(check.status),
                reason: check.reason,
            }
        }
        Err(reason) => AuthProviderView {
            name: name.to_owned(),
            status: ProviderStatusView::Error,
            reason: Some(reason),
        },
    }
}

/// The rule's four states as the contract names them: the two enums carry the
/// same meanings, so the value never travels as text.
fn status_view(status: AuthStatus) -> ProviderStatusView {
    match status {
        AuthStatus::Ready => ProviderStatusView::Ready,
        AuthStatus::NotReady => ProviderStatusView::NotReady,
        AuthStatus::Invalid => ProviderStatusView::Invalid,
        AuthStatus::Error => ProviderStatusView::Error,
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::*;
    use crate::settings::ProviderAnswer;

    const STORE: &str = r#"{
        "anthropic": {"type": "oauth", "access": "sk-SECRET"},
        "commandcode": {"type": "api_key", "key": "cc-SECRET"}
    }"#;

    fn reader(text: Option<&str>) -> FileRead {
        let text = text.map(str::to_owned);
        Arc::new(move |_path: &Path| Ok(text.clone()))
    }

    /// A probe that answers the exit code and line of a chosen provider and
    /// records which names were asked, in order.
    fn probe(
        answers: &'static [(&'static str, i32, &'static str)],
        asked: Arc<std::sync::Mutex<Vec<String>>>,
    ) -> ProviderProbe {
        Arc::new(move |provider: &str| {
            asked
                .lock()
                .expect("the log is not poisoned")
                .push(provider.to_owned());
            let (_, code, line) = answers
                .iter()
                .find(|(name, _, _)| *name == provider)
                .expect("the test answers every provider it lists");
            Ok(ProviderAnswer {
                exit_code: *code,
                stdout: (*line).to_owned(),
            })
        })
    }

    #[test]
    fn every_named_provider_gets_its_own_answer() {
        let asked = Arc::new(std::sync::Mutex::new(Vec::new()));
        let probe = probe(
            &[
                (
                    "anthropic",
                    1,
                    r#"{"status":"not_ready","provider":"anthropic","reason":"OAuth token expired"}"#,
                ),
                (
                    "commandcode",
                    0,
                    r#"{"status":"ready","provider":"commandcode","authType":"api_key"}"#,
                ),
            ],
            Arc::clone(&asked),
        );

        let view = view(
            &reader(Some(STORE)),
            Some(&PathBuf::from("C:/pi/agent/auth.json")),
            &probe,
        );

        assert_eq!(view.status, AuthStatusView::Ready);
        assert_eq!(
            *asked.lock().expect("the log is not poisoned"),
            vec!["anthropic".to_owned(), "commandcode".to_owned()],
            "each name is checked once, in the store's order"
        );
        assert_eq!(view.providers.len(), 2);
        assert_eq!(view.providers[0].name, "anthropic");
        assert_eq!(view.providers[0].status, ProviderStatusView::NotReady);
        assert_eq!(
            view.providers[0].reason.as_deref(),
            Some("OAuth token expired")
        );
        assert_eq!(view.providers[1].status, ProviderStatusView::Ready);
        assert_eq!(view.providers[1].reason, None);
    }

    #[test]
    fn the_store_values_never_reach_the_view() {
        let asked = Arc::new(std::sync::Mutex::new(Vec::new()));
        let view = view(
            &reader(Some(STORE)),
            Some(&PathBuf::from("auth.json")),
            &probe(&[("anthropic", 0, "{}"), ("commandcode", 0, "{}")], asked),
        );
        let carried = format!("{view:?}");
        assert!(!carried.contains("SECRET"), "{carried}");
        assert!(!carried.contains("api_key"), "{carried}");
        assert!(!carried.contains("oauth"), "{carried}");
    }

    #[test]
    fn a_check_that_cannot_run_is_that_providers_error() {
        let failing: ProviderProbe =
            Arc::new(|provider: &str| Err(format!("gentle-shell could not run for {provider}")));

        let view = view(
            &reader(Some(STORE)),
            Some(&PathBuf::from("auth.json")),
            &failing,
        );

        assert_eq!(view.providers.len(), 2);
        assert!(view
            .providers
            .iter()
            .all(|provider| provider.status == ProviderStatusView::Error));
        assert!(view.providers[0]
            .reason
            .as_deref()
            .is_some_and(|reason| reason.contains("could not run")));
    }

    #[test]
    fn a_store_nobody_wrote_is_a_ready_pane_with_no_providers() {
        let asked = Arc::new(std::sync::Mutex::new(Vec::new()));
        let view = view(
            &reader(None),
            Some(&PathBuf::from("auth.json")),
            &probe(&[], asked),
        );
        assert_eq!(view.status, AuthStatusView::Ready);
        assert!(view.providers.is_empty());
    }

    #[test]
    fn a_store_that_cannot_be_read_says_why() {
        let failing: FileRead = Arc::new(|_path: &Path| Err("permission denied".to_owned()));
        let asked = Arc::new(std::sync::Mutex::new(Vec::new()));
        let view = view(
            &failing,
            Some(&PathBuf::from("auth.json")),
            &probe(&[], asked),
        );
        assert_eq!(
            view.status,
            AuthStatusView::Failed("permission denied".to_owned())
        );
    }

    #[test]
    fn no_agent_folder_is_said_plainly() {
        let asked = Arc::new(std::sync::Mutex::new(Vec::new()));
        let view = view(&reader(Some(STORE)), None, &probe(&[], asked));
        match view.status {
            AuthStatusView::Failed(reason) => assert!(reason.contains("agent folder"), "{reason}"),
            other => panic!("no folder must say so, got {other:?}"),
        }
    }
}
