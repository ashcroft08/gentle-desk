//! The release manifest of this run: where `latest.json` lives and the one fetch
//! of it the what's-new step makes (what's-new S3).
//!
//! Networking lives here and nowhere else in the bridge. The endpoint is read out
//! of the SAME app config the updater plugin reads - the `tauri.conf.json`
//! embedded at build time - so the app asks one address for one manifest, never
//! two, and a machine with no endpoint configured asks nothing.
//!
//! Nothing here runs on the thread that draws: `commands::Bridge::start_whats_new`
//! hands the fetch to a thread of its own and pushes what it answers, so a slow
//! or absent release server never delays the first frame (S6).

use std::sync::Arc;
use std::time::Duration;

use gentle_runtime::whats_new::Latest;

use crate::commands::Fetcher;

/// How long one fetch may take, connection included: the notes are best effort
/// (S3), so a server that does not answer in time is a run that stores the
/// version and shows nothing.
const TIMEOUT: Duration = Duration::from_secs(8);

/// The endpoint the updater is configured with: the FIRST of the updater's
/// `endpoints` in the app config, which is the one the plugin asks first. `None`
/// when this build names none, which is a run with no release server to ask.
pub fn endpoint(app: &tauri::AppHandle<tauri::Wry>) -> Option<String> {
    endpoint_of(app.config().plugins.0.get("updater"))
}

/// The first endpoint of one `plugins.updater` entry, or `None` for an entry that
/// names none. Pure, so the shape of the config is pinned without a window.
pub(crate) fn endpoint_of(updater: Option<&serde_json::Value>) -> Option<String> {
    updater?
        .get("endpoints")?
        .as_array()?
        .first()?
        .as_str()
        .map(str::to_owned)
}

/// The real fetcher the what's-new step is given.
pub fn fetcher() -> Fetcher {
    Arc::new(fetch)
}

/// One GET of `url`, bounded by [`TIMEOUT`], answering the version and notes the
/// manifest announces. Every failure - no network, no answer, an answer that is
/// not a success, a body that is not this manifest - is the same value: the step
/// then stores the version and shows nothing (S3, S5).
fn fetch(url: &str) -> Result<Latest, String> {
    install_tls_provider();
    let client = reqwest::blocking::Client::builder()
        .timeout(TIMEOUT)
        .build()
        .map_err(|error| error.to_string())?;
    let response = client.get(url).send().map_err(|error| error.to_string())?;
    if !response.status().is_success() {
        return Err(format!("the release server answered {}", response.status()));
    }
    response.json::<Latest>().map_err(|error| error.to_string())
}

/// Installs the ring crypto provider a rustls client needs once per process: the
/// same provider the updater plugin installs for its own request, so the two
/// never disagree about which one is in use. A provider already installed is not
/// an error - it is the one this would set.
fn install_tls_provider() {
    let _ = rustls::crypto::ring::default_provider().install_default();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_endpoint_is_the_first_one_the_updater_names() {
        // The order is the plugin's own: it asks the endpoints in the order the
        // config lists them, so the manifest this step reads is the one the
        // updater would read first.
        let updater = serde_json::json!({
            "pubkey": "key",
            "endpoints": [
                "https://example.test/releases/latest.json",
                "https://second.test/releases/latest.json"
            ]
        });
        assert_eq!(
            endpoint_of(Some(&updater)).as_deref(),
            Some("https://example.test/releases/latest.json")
        );
    }

    #[test]
    fn a_config_that_names_no_endpoint_names_none() {
        assert_eq!(endpoint_of(None), None);
        assert_eq!(
            endpoint_of(Some(&serde_json::json!({"pubkey": "key"}))),
            None
        );
        assert_eq!(
            endpoint_of(Some(&serde_json::json!({"endpoints": []}))),
            None
        );
        assert_eq!(
            endpoint_of(Some(&serde_json::json!({"endpoints": [42]}))),
            None
        );
        assert_eq!(
            endpoint_of(Some(&serde_json::json!({"endpoints": "https://one.test"}))),
            None
        );
    }

    #[test]
    fn the_config_of_this_build_names_the_release_manifest() {
        // The real `tauri.conf.json`, embedded at build time: the address the
        // updater plugin asks, and the manifest file itself over https.
        let config: serde_json::Value = serde_json::from_str(include_str!("../tauri.conf.json"))
            .expect("the app config is JSON");
        let endpoint = endpoint_of(
            config
                .get("plugins")
                .and_then(|plugins| plugins.get("updater")),
        )
        .expect("the app config names an updater endpoint");
        assert!(
            endpoint.starts_with("https://"),
            "the manifest travels over https, got {endpoint}"
        );
        assert!(
            endpoint.ends_with("/latest.json"),
            "the endpoint is the release manifest, got {endpoint}"
        );
    }

    #[test]
    fn an_address_that_is_no_url_fails_before_any_request() {
        // Never the network: a URL reqwest cannot parse is refused while the
        // request is built, which is the error path the step turns into "store the
        // version and show nothing".
        assert!(fetch("not a url").is_err());
    }
}
