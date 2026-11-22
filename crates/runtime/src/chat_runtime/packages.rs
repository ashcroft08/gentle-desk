//! Which extensions are behind, read over one chat's own child (packages-status
//! T1, S1, S2, S4, S5).
//!
//! pi already runs this check for its terminal notice ("Package updates are
//! available"), and the same facts are one method away: `get_update_status`
//! reports pi's own packages slice, with a state per extension. The question is
//! asked where the answer lives - over the chat's own [`DesktopClient`], the same
//! client a helper stop and the profile read use, which is what gives one chat ONE
//! `requestId` counter - and never over a client of its own.
//!
//! ONE exchange, ONE deadline. The method is asked exactly once with
//! `{refresh: false}`, a plain read of what pi already knows, under the channel's
//! own deadline. `refresh: true` is deliberately never sent: the channel defines it
//! as RUNNING the checks once, with a 15 s deadline of its own - longer than this
//! channel's bound - and this app is a reader of pi's check, never the thing that
//! starts it (S2, S4). A plain read that cannot answer packages leaves the honest
//! state [`PackagesStatus::unknown`], never a forced check and never "up to date".
//!
//! Every outcome is typed and nothing is an error string on the wire: the chat has
//! no child to ask ([`PackagesRead::Unasked`], which is also what a shell that does
//! not advertise the method answers, S5), the child did not answer in time
//! ([`PackagesRead::Deadline`]), the owner refused and worded the code itself
//! ([`PackagesRead::Refused`]), or the listing was read
//! ([`PackagesRead::Read`]).
//!
//! The listing is DERIVED before it travels: only `update_available` is behind
//! ([`PackagesStatus::behind`]), every other state - `no_update_reported`,
//! `skipped`, `unknown`, a state this build has not learned - never is, and a
//! listing the child could not produce (`available: false`) is
//! [`PackagesStatus::unknown`]. `latest` is not carried at all (it is always null
//! today), and neither is a raw source, a `kind` or a `scope` (S4).

use serde_json::Value;

use super::ChatRuntime;
use gentle_app::ChatId;
use gentle_core::rpc::{DesktopChannel, DesktopClient, DesktopOutcome, NoAnswer};

/// The method that reports pi's own update check, packages included.
pub const GET_UPDATE_STATUS: &str = "get_update_status";

/// The contract's own bound on one packages listing. The child reports
/// `truncated` for what IT cut; this app never draws more than the same hundred,
/// so a child that ignores its bound cannot make the slice unbounded.
pub const MAX_ITEMS: usize = 100;

/// What one packages read came to (S1, S4, S5). Every variant is a state the
/// view can truthfully word; none of them is a message.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum PackagesRead {
    /// Nothing was asked: the chat has no child to ask (never opened, already
    /// closed, a run with no host), or the child does not advertise the method
    /// (an older shell, S5). Today's honest sentence, never "up to date".
    #[default]
    Unasked,
    /// The child did not answer within the channel's own deadline (it does not
    /// know the command, its host is inactive, or it closed first).
    Deadline,
    /// The owner answered `ok: false` and named the reason itself.
    Refused(String),
    /// The child's listing, typed and derived.
    Read(PackagesStatus),
}

/// pi's packages slice, as `get_update_status` reports it, reduced to what the
/// Extensions block draws.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct PackagesStatus {
    /// Whether the child could produce its listing at all. `false` is "the read
    /// could not run": what pi's check knows is not reported, and nothing here
    /// turns that into "up to date" (S1).
    pub available: bool,
    /// One entry per extension the child named, in its own order.
    pub items: Vec<PackagesItem>,
    /// Whether the listing this app holds is cut: the child's own `truncated`, or
    /// the contract's [`MAX_ITEMS`] bound cutting what the child sent.
    pub truncated: bool,
    /// When pi's check last read the machine, in epoch milliseconds, when the
    /// child reports it as a number. Any other shape is `None`, never a guess.
    pub checked_ms: Option<f64>,
}

impl PackagesStatus {
    /// The listing `data` carries, read down to the facts this slice draws.
    ///
    /// An answer with no `packages` object is a listing the child does not
    /// report: `available: false`, which the block words as a read that could not
    /// run rather than as "nothing is behind".
    pub fn of(data: &Value) -> Self {
        let Some(packages) = data.get("packages").and_then(Value::as_object) else {
            return Self::default();
        };
        let available = packages
            .get("available")
            .and_then(Value::as_bool)
            // An `available` this contract does not read is not a claim that the
            // listing exists.
            .unwrap_or(false);
        let child_truncated = packages
            .get("truncated")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let checked_ms = packages.get("checkedAt").and_then(Value::as_f64);
        let named = packages
            .get("items")
            .and_then(Value::as_array)
            .map(|items| items.iter().filter_map(item).collect::<Vec<_>>())
            .unwrap_or_default();
        // The contract's own bound, kept here as well: an entry this app does not
        // draw is a listing this app cut, and says so. A listing the child could
        // not produce carries nothing to draw: an answer that names extensions
        // while saying it has no listing is not knowledge.
        let cut = named.len() > MAX_ITEMS;
        let mut items = if available { named } else { Vec::new() };
        items.truncate(MAX_ITEMS);
        Self {
            available,
            items,
            truncated: available && (child_truncated || cut),
            checked_ms,
        }
    }

    /// The packages that are behind: ONLY `update_available` is (S1). Every other
    /// state pi reports - and every state this build has not learned - is not an
    /// update.
    pub fn behind(&self) -> impl Iterator<Item = &PackagesItem> {
        self.items
            .iter()
            .filter(|item| item.state == PackagesState::UpdateAvailable)
    }

    /// Whether the child's listing could not be read, so nothing may claim to be
    /// up to date (S1): the block says the read could not run.
    pub fn unknown(&self) -> bool {
        !self.available
    }
}

/// One extension pi's check reports, reduced to its display name, the version it
/// runs and its derived state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackagesItem {
    /// The shell's own display name for the package (`@scope/name`), never a raw
    /// source such as a git URL or a path (S4).
    pub name: String,
    /// The version installed, when the child reports one.
    pub current: Option<String>,
    /// The state the child reports, typed. `latest` is not carried: it is always
    /// null today.
    pub state: PackagesState,
}

/// The state of one extension, as far as this app lets it mean anything.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackagesState {
    /// pi's check says a newer version exists: the only state that is behind.
    UpdateAvailable,
    /// pi's own check reported nothing newer for it.
    NoUpdateReported,
    /// The user skipped it, or pi skipped the check for it.
    Skipped,
    /// pi does not know: `unknown`, and every state this build has not learned.
    Unknown,
}

/// Reads one extension listing over `client`, which is that chat's own: one
/// `get_update_status` ask with the plain-read params, under the channel's
/// deadline.
pub fn packages_status<C: DesktopChannel>(client: &mut DesktopClient<C>) -> PackagesRead {
    match client.ask(GET_UPDATE_STATUS, plain_read()) {
        DesktopOutcome::Data { data, .. } => PackagesRead::Read(PackagesStatus::of(&data)),
        DesktopOutcome::Rejected { error, .. } => PackagesRead::Refused(error.code),
        // Nothing was asked: no contract was resolved, or the child does not
        // advertise the method (an older shell, S5).
        DesktopOutcome::NoAnswer(NoAnswer::NotConnected) | DesktopOutcome::Unsupported { .. } => {
            PackagesRead::Unasked
        }
        DesktopOutcome::NoAnswer(_) => PackagesRead::Deadline,
        // The request broke a limit before it left, which the contract's own code
        // for that is: nothing was asked, so nothing is behind.
        DesktopOutcome::Invalid(_) => PackagesRead::Refused("invalid_request".to_owned()),
    }
}

/// The one params this app ever sends: a plain read, which the channel documents
/// as memory-only. See the module's own note on why `refresh: true` is never sent.
fn plain_read() -> Value {
    serde_json::json!({ "refresh": false })
}

/// One entry of the child's `items[]`, or `None` for one this app cannot draw
/// (no display name at all).
fn item(raw: &Value) -> Option<PackagesItem> {
    let name = raw.get("name")?.as_str()?.trim();
    if name.is_empty() {
        return None;
    }
    Some(PackagesItem {
        name: name.to_owned(),
        current: raw
            .get("current")
            .and_then(Value::as_str)
            .map(str::to_owned),
        state: state(raw.get("state").and_then(Value::as_str)),
    })
}

/// The state `raw` names, typed: the four the contract defines, and
/// [`PackagesState::Unknown`] for every other string - never behind by accident.
fn state(raw: Option<&str>) -> PackagesState {
    match raw {
        Some("update_available") => PackagesState::UpdateAvailable,
        Some("no_update_reported") => PackagesState::NoUpdateReported,
        Some("skipped") => PackagesState::Skipped,
        _ => PackagesState::Unknown,
    }
}

impl ChatRuntime {
    /// Reads which extensions are behind over `chat`'s OWN child.
    ///
    /// Blocking: the exchange waits for the child's answer under the channel's own
    /// deadline, so the caller runs it off the window's thread. A chat the host
    /// has no key for (a fresh chat that was never opened) has no child and
    /// answers [`PackagesRead::Unasked`] without writing anything.
    pub fn read_packages(&self, chat: &ChatId) -> PackagesRead {
        let Some(client) = self.desktop_client(chat) else {
            return PackagesRead::Unasked;
        };
        // One exchange per chat at a time, which is what makes its request ids
        // meaningful; another chat's read runs in parallel.
        let mut client = super::lock(&client);
        packages_status(&mut client)
    }
}
