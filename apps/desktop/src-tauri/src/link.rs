//! The link policy of the shell (T5a): WHICH url the system browser may be
//! asked to open.
//!
//! A link arrives from a model's markdown, which is untrusted text: it can name
//! `file:` (a local program or a share), `javascript:` or `data:` (script in
//! whatever opens it), or an `http` url with embedded credentials. Only a plain
//! `http`/`https` url with a host and no credentials is opened; every other
//! string is refused before anything runs.
//!
//! Control characters are checked BEFORE parsing, because the URL standard
//! strips tabs and newlines: `https://example.com/a\nb` would otherwise parse
//! into a link nobody typed.

use url::Url;

/// Whether one string may be handed to the system browser, with the reason it
/// may not when it may not.
///
/// The reason is plain text for the caller (the `open_link` command returns it
/// to the web side), and it never quotes the url: a refused url can carry
/// credentials, and a reason that echoes it would put them in a log.
pub fn check(url: &str) -> Result<(), String> {
    if url.trim().is_empty() {
        return Err("there is no link to open".to_owned());
    }
    if url.chars().any(char::is_control) {
        return Err("a link carrying control characters is not opened".to_owned());
    }
    let parsed = Url::parse(url).map_err(|_| "that is not a whole link".to_owned())?;
    match parsed.scheme() {
        "http" | "https" => {}
        _ => {
            return Err("only http and https links are opened".to_owned());
        }
    }
    if parsed.host_str().is_none_or(str::is_empty) {
        return Err("a link without a host is not opened".to_owned());
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err("a link naming credentials is not opened".to_owned());
    }
    Ok(())
}
