//! The subscription usage the maintainer's own `pi-usage-sources` package
//! reports (settings-0.1.12 S6's deferred half, delivered here): where its
//! checkout sits on this machine, and what the one document it prints means.
//!
//! Two rules live here, both pure:
//!
//! - the LOCATOR reads the shell's `settings.json` for the entry that names the
//!   package and turns it into the folder pi checked it out into
//!   (`<agent home>/git/<host>/<owner>/<repo>` for a `git:` entry, the path
//!   itself for a local one);
//! - the PARSER reads the versioned document the script prints, strictly: a
//!   document of another schema is refused rather than guessed, a provider the
//!   package answered "no-data" for is NO DATA and never a zero percent, and a
//!   window's reset time may be an explicit null (no countdown).
//!
//! Nothing here runs a process or touches a file: the shell's side of the pane
//! supplies the text, the document and the clock.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::time_ago::time_until;

/// The schema string the package's headless mode prints. A document of any other
/// schema is not this pane's to read.
pub const SCHEMA: &str = "pi-usage-sources/usage/v1";

/// The package's own name: the last segment of the entry `settings.json` holds.
pub const PACKAGE: &str = "pi-usage-sources";

/// The script the pane runs inside the checkout: the package's headless JSON
/// entry point.
pub const SCRIPT: &str = "usage-json.ts";

/// The entry of the shell's `settings.json` that names the package, verbatim:
/// the string of a `git:` or local-path package, or an object entry's `source`.
///
/// `None` for a document that cannot be read, for one that lists no such
/// package, and for the `npm:` spelling: an npm-installed copy has no checkout
/// under the agent home, so the pane could not run it even if it named it.
pub fn package_entry(settings: Option<&str>) -> Option<String> {
    let document: Value = serde_json::from_str(settings?).ok()?;
    let packages = document.get("packages")?.as_array()?;
    packages.iter().find_map(|package| {
        let source = match package {
            Value::String(source) => source.as_str(),
            Value::Object(fields) => fields.get("source")?.as_str()?,
            _ => return None,
        };
        // Every package spelling ends with the package's name as its last path
        // segment: `git:host/owner/pi-usage-sources`, `<folder>/pi-usage-sources`.
        let last = source.rsplit('/').next()?.trim_end_matches(".git");
        (last == PACKAGE && !source.starts_with("npm:")).then(|| source.to_owned())
    })
}

/// The folder `entry` names: pi checks a `git:` package out at
/// `<agent home>/git/<host>/<owner>/<repo>`, and a local entry is the path it
/// spells (absolute, or relative to the agent home).
///
/// `None` for an entry this rule cannot turn into a folder: an `npm:` package
/// has no checkout there, and a `git:` address with an empty, `.` or `..`
/// segment would climb out of the git folder rather than name a repository.
pub fn checkout(entry: &str, home: &Path) -> Option<PathBuf> {
    if entry.starts_with("npm:") {
        return None;
    }
    let Some(address) = entry.strip_prefix("git:") else {
        let path = Path::new(entry);
        return if path.is_absolute() {
            Some(path.to_path_buf())
        } else {
            Some(home.join(path))
        };
    };
    // A repository is named, never walked: `..` in an address is refused.
    let address = address.trim_end_matches('/').trim_end_matches(".git");
    let mut path = home.join("git");
    for segment in address.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." {
            return None;
        }
        path.push(segment);
    }
    (path != home.join("git")).then_some(path)
}

/// The script the pane runs: the package's [`SCRIPT`] inside its checkout,
/// `None` when no entry names the package or the entry names no folder.
pub fn script(settings: Option<&str>, home: &Path) -> Option<PathBuf> {
    let entry = package_entry(settings)?;
    checkout(&entry, home).map(|folder| folder.join(SCRIPT))
}

/// How one provider of the document stands: the three statuses the package
/// prints, and nothing else.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderStatus {
    /// The provider answered: its limits and windows are real readings.
    Ok,
    /// The provider had nothing to report (no credentials, nothing fetched yet):
    /// the pane draws "no data", never a zero percent.
    NoData,
    /// The provider's fetch failed; `error` says why, already redacted by the
    /// package.
    Failed,
}

/// One usage window as a provider reports it: its label ("5h", "week"), the
/// share of it used, and when it resets (an explicit `None` is "no countdown").
#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    pub label: String,
    pub used_percent: f64,
    pub reset_at: Option<u64>,
}

/// One named limit of a provider, with its windows.
#[derive(Debug, Clone, PartialEq)]
pub struct Limit {
    pub name: String,
    pub limit_reached: bool,
    pub windows: Vec<Window>,
}

/// One provider of the document: its name, how it stands, its plan when it
/// reports one, and its limits.
#[derive(Debug, Clone, PartialEq)]
pub struct Provider {
    pub provider: String,
    pub status: ProviderStatus,
    pub plan: Option<String>,
    pub error: Option<String>,
    pub limits: Vec<Limit>,
}

/// The whole document, reduced to what the pane draws.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub providers: Vec<Provider>,
}

/// Reads the package's document, strictly.
///
/// Refused with a plain reason, never guessed: something that is not a JSON
/// object, the wrong schema string, a `sources` list that is not a list, a
/// provider whose fields this reader cannot use, and a status this reader does
/// not know. A provider the package reported as `failed` or `no-data` is a
/// STATUS, not a refusal: those are answers.
pub fn parse(text: &str) -> Result<Document, String> {
    let document: Value =
        serde_json::from_str(text).map_err(|_| unreadable(&format!("schema \"{SCHEMA}\"")))?;
    if !document.is_object() {
        return Err(unreadable(&format!("schema \"{SCHEMA}\"")));
    }
    match document.get("schema").and_then(Value::as_str) {
        Some(SCHEMA) => {}
        Some(other) => return Err(unreadable(&format!("schema \"{other}\""))),
        None => return Err(unreadable(&format!("schema \"{SCHEMA}\""))),
    }
    let sources = document
        .get("sources")
        .and_then(Value::as_array)
        .ok_or_else(|| unreadable("a document with no provider list"))?;
    let providers = sources
        .iter()
        .enumerate()
        .map(|(index, source)| provider(source, index + 1))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Document { providers })
}

/// The one refusal sentence: what the pane could not read, and that it read
/// nothing of it.
fn unreadable(what: &str) -> String {
    format!("the usage sources answered {what} this pane does not read")
}

/// One provider of `sources`, `position` being its 1-based place in the list so
/// a refusal can name which one it was.
fn provider(source: &Value, position: usize) -> Result<Provider, String> {
    let provider = match source.get("provider").and_then(Value::as_str) {
        Some(name) if !name.trim().is_empty() => name.to_owned(),
        _ => return Err(unreadable(&format!("provider {position} with no name"))),
    };
    let status = match source.get("status").and_then(Value::as_str) {
        Some("ok") => ProviderStatus::Ok,
        Some("no-data") => ProviderStatus::NoData,
        Some("failed") => ProviderStatus::Failed,
        Some(other) => return Err(unreadable(&format!("status \"{other}\""))),
        None => return Err(unreadable("status \"unknown\"")),
    };
    let limits = source
        .get("limits")
        .and_then(Value::as_array)
        .ok_or_else(|| unreadable(&format!("provider {provider} with no limit list")))?
        .iter()
        .enumerate()
        .map(|(index, entry)| limit(entry, &provider, index + 1))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Provider {
        provider,
        status,
        // A plan and a failure reason are the package's own words; an empty one
        // is the same as none.
        plan: text_of(source, "plan"),
        error: text_of(source, "error"),
        limits,
    })
}

/// One limit of `provider`, `position` being its 1-based place in the list.
fn limit(limit: &Value, provider: &str, position: usize) -> Result<Limit, String> {
    let name = match limit.get("name").and_then(Value::as_str) {
        Some(name) if !name.trim().is_empty() => name.to_owned(),
        _ => return Err(unreadable(&format!("a limit of {provider} with no name"))),
    };
    let limit_reached = limit
        .get("limitReached")
        .and_then(Value::as_bool)
        .ok_or_else(|| unreadable(&format!("a limit of {provider} with no reach mark")))?;
    let windows = limit
        .get("windows")
        .and_then(Value::as_array)
        .ok_or_else(|| unreadable(&format!("limit {position} of {provider} with no windows")))?
        .iter()
        .map(|entry| window(entry, provider, &name))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Limit {
        name,
        limit_reached,
        windows,
    })
}

/// One window of `limit` of `provider`.
fn window(window: &Value, provider: &str, limit: &str) -> Result<Window, String> {
    let label = match window.get("label").and_then(Value::as_str) {
        Some(label) if !label.trim().is_empty() => label.to_owned(),
        _ => {
            return Err(unreadable(&format!(
                "a window of {provider} ({limit}) with no label"
            )))
        }
    };
    let used_percent = window
        .get("usedPercent")
        .and_then(Value::as_f64)
        .filter(|percent| percent.is_finite())
        .ok_or_else(|| {
            unreadable(&format!(
                "a window of {provider} ({limit}) with no percentage"
            ))
        })?;
    // The package keeps an explicit null for a provider that reports no reset,
    // so `null` is a real answer and a missing field is not.
    let reset_at = match window.get("resetAt") {
        Some(Value::Null) | None => None,
        Some(value) => Some(
            value
                .as_u64()
                .ok_or_else(|| unreadable(&format!("a reset time of {provider}")))?,
        ),
    };
    Ok(Window {
        label,
        used_percent,
        reset_at,
    })
}

/// A non-empty string field, `None` for anything else.
fn text_of(value: &Value, field: &str) -> Option<String> {
    value
        .get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_owned)
}

/// How long until a window resets, as the pane draws it: "resets in 4h", or
/// "resets now" for a time that already passed. `None` when the provider reports
/// no reset time at all, which is what an explicit `null` is.
pub fn reset_label(now_ms: u64, reset_at: Option<u64>) -> Option<String> {
    let at = reset_at?;
    if at <= now_ms {
        return Some("resets now".to_owned());
    }
    Some(format!("resets in {}", time_until(now_ms, at)))
}

#[cfg(test)]
mod tests;
