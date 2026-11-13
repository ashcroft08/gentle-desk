//! The one GET the update check makes: the release manifest and the npm
//! registry entry, both read as text and parsed by their own rule.
//!
//! The client is the same one the what's-new step uses (`release`): the updater
//! plugin's own TLS stack, so no second backend enters the build, and a bounded
//! timeout because a check that cannot answer is a row left unknown, never a
//! window that waits.

use std::time::Duration;

/// One GET of `url`, answering the body. Every failure - no network, no answer,
/// an answer that is not a success - is the same `Err`, which the pane turns
/// into an unknown version.
pub fn get(url: &str, timeout: Duration) -> Result<String, String> {
    crate::release::install_tls_provider();
    let client = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .map_err(|error| error.to_string())?;
    let response = client.get(url).send().map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("{url} answered {}", response.status()));
    }
    response.text().map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_address_that_is_no_url_fails_before_any_request() {
        // Never the network: a URL reqwest cannot parse is refused while the
        // request is built.
        assert!(get("not a url", Duration::from_millis(1)).is_err());
    }
}
