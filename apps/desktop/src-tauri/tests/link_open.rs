//! Windowless tests of the shell's only link rule (T5a): a link the thread
//! renders is opened in the system browser, and NOTHING but http and https
//! ever reaches the opener.
//!
//! The opener is the seam ([`LinkOpener`]), so a fake records every URL and no
//! browser is ever started by these tests.

use std::sync::{Mutex, MutexGuard};

use gentle_desk_lib::link_opener::{open, LinkOpener};

/// The opener a test injects: it remembers every URL it was asked to open,
/// including the ones it failed to open.
#[derive(Default)]
struct FakeOpener {
    opened: Mutex<Vec<String>>,
    failing: bool,
}

impl FakeOpener {
    fn failing() -> Self {
        Self {
            opened: Mutex::new(Vec::new()),
            failing: true,
        }
    }

    fn opened(&self) -> Vec<String> {
        lock(&self.opened).clone()
    }
}

impl LinkOpener for FakeOpener {
    fn open_url(&self, url: &str) -> Result<(), String> {
        lock(&self.opened).push(url.to_owned());
        if self.failing {
            Err("no browser is registered for this link".to_owned())
        } else {
            Ok(())
        }
    }
}

#[test]
fn an_http_link_is_opened_once_with_the_exact_url() {
    let opener = FakeOpener::default();
    let url = "http://example.com/thread/7?page=2#reply-3";

    assert_eq!(open(&opener, url), Ok(()));

    assert_eq!(
        opener.opened(),
        vec![url.to_owned()],
        "the URL reaches the opener as the thread sent it"
    );
}

#[test]
fn an_https_link_is_opened_once_with_the_exact_url() {
    let opener = FakeOpener::default();
    let url = "https://example.com/a/b";

    assert_eq!(open(&opener, url), Ok(()));

    assert_eq!(opener.opened(), vec![url.to_owned()]);
}

#[test]
fn every_other_scheme_is_refused_before_the_opener_runs() {
    let opener = FakeOpener::default();

    for url in [
        "file:///C:/Windows/System32/cmd.exe",
        "javascript:alert(1)",
        "vbscript:msgbox(1)",
        "data:text/html,<b>hello</b>",
        "mailto:someone@example.com",
        "ftp://example.com/file",
        "relative/path",
        "../upwards",
        "/absolute/path",
        "example.com/no-scheme",
        "",
        "   ",
    ] {
        assert!(
            open(&opener, url).is_err(),
            "`{url}` must be refused, not opened"
        );
    }
    assert_eq!(
        opener.opened(),
        Vec::<String>::new(),
        "a refused URL never reaches the opener"
    );
}

#[test]
fn a_link_naming_credentials_or_carrying_control_characters_is_refused() {
    let opener = FakeOpener::default();

    for url in [
        "https://user:pass@example.com/",
        "https://user@example.com/",
        "http://user:pass@localhost:8080/x",
        "https://example.com/a\nb",
        "https://example.com/a\rb",
        "https://example.com/a\tb",
        "https://example.com/a\u{0}b",
    ] {
        assert!(
            open(&opener, url).is_err(),
            "`{url}` must be refused, not opened"
        );
    }
    assert_eq!(opener.opened(), Vec::<String>::new());
}

#[test]
fn an_https_link_with_no_host_is_refused() {
    let opener = FakeOpener::default();

    assert!(open(&opener, "https://").is_err());
    assert_eq!(opener.opened(), Vec::<String>::new());
}

#[test]
fn a_failing_opener_is_an_error_and_never_a_panic() {
    let opener = FakeOpener::failing();

    let result = open(&opener, "https://example.com/x");

    assert!(
        result.is_err(),
        "the failure reaches the web side as an error"
    );
    assert_eq!(
        opener.opened(),
        vec!["https://example.com/x".to_owned()],
        "the attempt is the one the thread asked for"
    );
}

/// A lock that survives a poisoned mutex: a test that failed while holding one
/// must not turn into a second failure.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}
