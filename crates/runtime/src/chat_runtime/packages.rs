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
//! ONE exchange, ONE deadline. A plain read asks the method exactly once with
//! `{refresh: false}`, a memory-only read of what pi already knows, under the
//! channel's own deadline (5 s). The EXPLICIT check (S7,
//! [`check_packages`]) asks it once with `{refresh: true}`, which the channel
//! defines as RUNNING pi's checks once under a 15 s bound of its own, so that one
//! ask names its own longer deadline ([`REFRESH_DEADLINE`]) instead of failing
//! inside a plain read's bound. A read that cannot answer packages leaves the
//! honest state [`PackagesStatus::unknown`], never a forced check and never "up
//! to date".
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

use std::time::Duration;

use serde_json::Value;

use super::ChatRuntime;
use gentle_app::ChatId;
use gentle_core::rpc::{DesktopChannel, DesktopClient, DesktopOutcome, NoAnswer};

/// The method that reports pi's own update check, packages included.
pub const GET_UPDATE_STATUS: &str = "get_update_status";

/// How long the EXPLICIT check (packages-status T4, S7) waits for its answer.
///
/// The one ask that names `refresh: true` makes the shell run pi's own checks,
/// which the contract bounds at 15 s; the channel's 5 s default is a plain read's
/// bound and would cut the refresh short on a slow machine. Twenty seconds is
/// that bound plus the answer's own trip, and it is still a bound: a child that
/// never answers costs this one wait and nothing more.
pub const REFRESH_DEADLINE: Duration = Duration::from_secs(20);

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
    /// When pi's check last read the machine, in epoch milliseconds. The shell
    /// reports it as an ISO instant (`isoTime`, e.g. `2026-11-22T02:10:03.000Z`,
    /// what `checkedAt` is in its answer) or as a plain number; any other shape
    /// is `None`, never a guess.
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
        let checked_ms = checked_ms(packages.get("checkedAt"));
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

    /// Whether pi's own check has NOT run for the whole listing (S6): the child
    /// produced a listing, but at least one extension it names is still
    /// [`PackagesState::Unknown`].
    ///
    /// The shell reports a package as `update_available` only after its own check
    /// ran, and reports every checkable package as `unknown` until it does - so a
    /// listing with one `unknown` item is a listing nobody has checked, not a
    /// machine that is up to date. An item whose state this build has not learned
    /// reads as `unknown` too, and is not a claim that the item is current. A
    /// listing that could not be read at all is [`Self::unknown`], never this: a
    /// read that failed is not a check that has not run.
    pub fn unchecked(&self) -> bool {
        self.available
            && self
                .items
                .iter()
                .any(|item| item.state == PackagesState::Unknown)
    }

    /// Whether the listing says this machine is UP TO DATE (S6): the child
    /// produced a listing, nothing is behind, and pi's own check ran for every
    /// item. Anything else is not this claim - one `unknown` item is "not checked
    /// yet", a `skipped` item is simply not checkable, and a listing that could
    /// not be read is "the read could not run".
    pub fn up_to_date(&self) -> bool {
        self.available && !self.unchecked() && self.behind().next().is_none()
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
    let outcome = client.ask(GET_UPDATE_STATUS, plain_read());
    read_of(outcome)
}

/// Runs pi's own update check over `client` and answers its listing (S7): the
/// ONE ask that names `refresh: true`, on its own [`REFRESH_DEADLINE`].
///
/// This is the explicit check the user asks for ("Comprobar de nuevo") and the
/// one the app runs once when the first chat's child is ready. It is still only a
/// READ of pi's check: nothing here installs anything (S4).
///
/// Asks `capabilities` first when the client has none, the rule every other
/// desktop request follows: the check is what makes the answer exist, so it is
/// worth the one extra bounded exchange, and a failed connection is retried on
/// the next check instead of poisoning the chat.
///
/// That connection is asked on [`REFRESH_DEADLINE`] too (packages-status T7): a
/// freshly spawned child answers NO desktop request until it is ready - measured
/// at ~14 s after spawn, with everything sent earlier answered at once at that
/// moment - so a connect on the channel's 5 s default fails on a child that was
/// only a few seconds away, and the run's one automatic check would be spent on
/// that failure.
pub fn check_packages<C: DesktopChannel>(client: &mut DesktopClient<C>) -> PackagesRead {
    if client.capabilities().is_none() {
        client.connect_within(REFRESH_DEADLINE);
    }
    let outcome = client.ask_within(GET_UPDATE_STATUS, refresh(), REFRESH_DEADLINE);
    read_of(outcome)
}

/// What one `get_update_status` answer became: the one mapping both the plain
/// read and the explicit check share.
fn read_of(outcome: DesktopOutcome) -> PackagesRead {
    match outcome {
        DesktopOutcome::Data { data, .. } => PackagesRead::Read(PackagesStatus::of(&data)),
        DesktopOutcome::Rejected { error, .. } => PackagesRead::Refused(error.code),
        // Nothing was asked: no contract was resolved, or the child does not
        // advertise the method (an older shell, S5).
        DesktopOutcome::NoAnswer(NoAnswer::NotConnected) | DesktopOutcome::Unsupported { .. } => {
            PackagesRead::Unasked
        }
        // A timeout, an inactive host, a closed child, a payload that is not this
        // contract's: one bounded failure, never a state.
        DesktopOutcome::NoAnswer(_) => PackagesRead::Deadline,
        // The request broke a limit before it left, which the contract's own code
        // for that is: nothing was asked, so nothing is behind.
        DesktopOutcome::Invalid(_) => PackagesRead::Refused("invalid_request".to_owned()),
    }
}

/// The one params this app ever sends: a plain read, which the channel documents
/// as memory-only. See the module's own note on why `refresh: true` is never sent
/// on a plain read.
fn plain_read() -> Value {
    serde_json::json!({ "refresh": false })
}

/// The explicit check's params: pi runs its own checks once for this answer
/// (S7), bounded by the shell at 15 s - which is what [`REFRESH_DEADLINE`]
/// tolerates.
fn refresh() -> Value {
    serde_json::json!({ "refresh": true })
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

    /// Runs pi's own update check over `chat`'s OWN child (packages-status T4,
    /// S7) and answers the listing it produced.
    ///
    /// Blocking, and for longer than a plain read: the ask names
    /// [`REFRESH_DEADLINE`], so the caller runs it on a thread of its own and
    /// never on the window's, and never inside a snapshot load. A chat the host
    /// has no key for answers [`PackagesRead::Unasked`] and touches nothing.
    pub fn check_packages(&self, chat: &ChatId) -> PackagesRead {
        let Some(client) = self.desktop_client(chat) else {
            return PackagesRead::Unasked;
        };
        // One exchange per chat at a time: the check numbers its request on the
        // same sequence as that chat's other desktop requests.
        let mut client = super::lock(&client);
        check_packages(&mut client)
    }

    /// The chat a packages read or check must ask, named the way the RUNTIME
    /// names chats (packages-status T7), or `None` when no open chat can answer.
    ///
    /// `preferred` - the chat in front - wins whenever the host has a key for
    /// it: a stored chat answers the session path the host knows, and a FRESH
    /// `new-N` chat answers the key registered when it was opened, so a chat with
    /// no session file yet is askable from its first second. Re-deriving a
    /// session path from an id is exactly what made a new chat unaskable, so
    /// nothing here reads the id text.
    ///
    /// When the host has no key for the chat in front (never opened, or this run
    /// cannot name it), another open chat is the answer instead of nothing: one
    /// the host has a live child for, else one the registry already holds a
    /// desktop client for. `None` only when there is no such chat, which is what
    /// keeps the zero-chat rule.
    pub fn packages_chat(&self, preferred: Option<&ChatId>) -> Option<ChatId> {
        {
            let keys = self.keys();
            if let Some(chat) = preferred {
                if keys.key_of(chat).is_some() {
                    return Some(chat.clone());
                }
            }
        }
        // Read the registry lock last and drop it before the clients lock is
        // taken: no lock here is ever held across another.
        let live = {
            let keys = self.keys();
            let live = self.host.live_keys();
            live.iter()
                .filter_map(|key| keys.id_of(key))
                .find(|chat| Some(chat) != preferred)
        };
        if let Some(live) = live {
            return Some(live);
        }
        let clients = super::lock(&self.clients);
        clients
            .keys()
            .find(|chat| Some(*chat) != preferred)
            .cloned()
    }
}
/// The check time as the shell reports it: `isoTime`'s ISO instant, or a plain
/// epoch-ms number. Anything else is nothing, never a guess.
fn checked_ms(value: Option<&Value>) -> Option<f64> {
    match value {
        Some(Value::Number(n)) => n.as_f64(),
        Some(Value::String(s)) => parse_instant(s).map(|ms| ms as f64),
        _ => None,
    }
}

/// One ISO 8601 instant (`2026-11-22T02:10:03.000Z`, optional fraction, `Z` or
/// an offset) into epoch milliseconds. The instant is only ever DRAWN as an age,
/// so a hand-rolled read with one month of headroom instead of a leap rule can
/// never flip that.
fn parse_instant(s: &str) -> Option<u128> {
    let b = s.as_bytes();
    if b.len() < 20
        || b[4] != b'-'
        || b[7] != b'-'
        || b[10] != b'T'
        || b[13] != b':'
        || b[16] != b':'
    {
        return None;
    }
    let num = |from: usize, len: usize| -> Option<u32> {
        let slice = b.get(from..from + len)?;
        if !slice.iter().all(u8::is_ascii_digit) {
            return None;
        }
        std::str::from_utf8(slice).ok()?.parse().ok()
    };
    let (year, month, day) = (num(0, 4)?, num(5, 2)?, num(8, 2)?);
    let (hour, minute, second) = (num(11, 2)?, num(14, 2)?, num(17, 2)?);
    if !(1..=12).contains(&month)
        || !(1..=31).contains(&day)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return None;
    }
    let secs = days_from_civil(i64::from(year), i64::from(month), i64::from(day)) * 86_400
        + i64::from(hour) * 3_600
        + i64::from(minute) * 60
        + i64::from(second);
    let (frac_ms, zone) = read_rest(&s[19..])?;
    let ms = u128::try_from(secs).ok()? * 1_000 + frac_ms;
    // The offset arrives as MINUTES (hh*60+mm); a millisecond is 60_000 of
    // them, not 1_000 - the first cut read it as seconds and shifted every
    // non-Z instant by 60x too little.
    let off_ms = u128::from(zone.1) * 60_000;
    // UTC = local − offset: a `+03:00` clock that says 02:10 is 23:10 the day
    // before in UTC, so '+' SUBTRACTS and '-' adds.
    match zone.0 {
        Some('+') => ms.checked_sub(off_ms),
        Some('-') | None => ms.checked_add(off_ms),
        _ => None,
    }
}

/// The fraction and the zone the rest after `HH:MM:SS` carries.
fn read_rest(rest: &str) -> Option<(u128, (Option<char>, u32))> {
    let (whole, zone) = match rest.split_once('.') {
        Some((_w, after)) => {
            let digits: String = after.chars().take_while(|c| c.is_ascii_digit()).collect();
            let mut ms: u128 = 0;
            for c in digits.chars() {
                ms = ms * 10 + u128::from(c.to_digit(10)?);
            }
            let ms = match digits.len() {
                0..=3 => ms * 10u128.pow(3 - digits.len() as u32),
                n => ms / 10u128.pow(n as u32 - 3),
            };
            (ms, &after[digits.len()..])
        }
        None => (0, rest),
    };
    let zone = zone.trim_start();
    let zone = match zone.as_bytes().first() {
        Some(b'Z') | Some(b'z') => Some((None, 0u32)),
        Some(b'+') | Some(b'-') => {
            let sign = char::from(zone.as_bytes()[0]);
            let mut it = zone[1..].split(':');
            let hh = it.next()?.parse::<u32>().ok()?;
            if hh > 23 {
                return None;
            }
            let mm = it.next().unwrap_or("0").parse::<u32>().ok()?;
            if mm > 59 {
                return None;
            }
            Some((Some(sign), hh * 60 + mm))
        }
        _ => None,
    }?;
    Some((whole, zone))
}

/// Howard Hinnant's `days_from_civil`: the one calendar rule this needs.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}
