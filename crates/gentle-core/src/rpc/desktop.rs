//! The `/gentle:desktop` channel: gentle-shell's desktop RPC, as its own
//! contract (`docs/desktop-rpc.md`, `gentle-shell.desktop-rpc/v1`).
//!
//! The channel is activated by `GENTLE_SHELL_INTERACTIVE_HOST=1`, takes a
//! request as an ordinary Pi `prompt` whose message is `/gentle:desktop`
//! followed by one JSON envelope, and answers through an Extension UI widget
//! (`widgetKey: "gentle-desktop-rpc"`) whose one `widgetLines` entry is the
//! answer envelope. The widget's own `requestId` is what correlates an answer
//! with its request; the outer Pi prompt id has nothing to do with it, so
//! control traffic never enters the conversation.
//!
//! This module is the pure half: the request the client writes, the typed
//! answer it reads, the correlation, and the capabilities contract it caches.
//! [`DesktopChannel`] is the seam the real I/O implements (the backend's host
//! channel); an in-memory fake is all a test needs.
//!
//! Degrading is a first-class outcome: a child that ignores the prompt, a host
//! that is not interactive, a closed child, or an answer that does not parse is
//! a typed [`DesktopOutcome::NoAnswer`], never a panic and never something the
//! user sees. Nothing here reaches for a conversation, a UI or a clock.

use std::collections::BTreeSet;
use std::fmt;
use std::time::Duration;

use serde_json::{json, Map, Value};

use super::types::RpcCommand;

/// The contract this client speaks.
pub const SCHEMA: &str = "gentle-shell.desktop-rpc/v1";

/// The extension command a request is sent through.
pub const COMMAND: &str = "/gentle:desktop";

/// The one widget key the domain answers arrive under.
pub const WIDGET_KEY: &str = "gentle-desktop-rpc";

/// The contract's request limit: one request, envelope and all, is 64 KiB.
pub const MAX_REQUEST_BYTES: usize = 64 * 1024;

/// The contract's request id limit, in characters.
pub const MAX_REQUEST_ID_CHARS: usize = 128;

/// The contract's method name limit, in characters.
pub const MAX_METHOD_CHARS: usize = 64;

/// The method that reports what the child implements.
const CAPABILITIES: &str = "capabilities";

/// One `/gentle:desktop` request, validated against the contract's own limits.
#[derive(Debug, Clone, PartialEq)]
pub struct DesktopRequest {
    request_id: String,
    method: String,
    params: Value,
}

impl DesktopRequest {
    /// A request the channel will accept, or the reason it would not.
    ///
    /// The limits are the contract's, checked before anything is written: a
    /// request id of 1..=128 characters, a lowercase snake case method of at
    /// most 64 characters, and an envelope of at most 64 KiB.
    pub fn new(
        request_id: impl Into<String>,
        method: &str,
        params: Value,
    ) -> Result<Self, RequestError> {
        let request_id = request_id.into();
        let chars = request_id.chars().count();
        if chars == 0 {
            return Err(RequestError::EmptyRequestId);
        }
        if chars > MAX_REQUEST_ID_CHARS {
            return Err(RequestError::RequestIdTooLong { chars });
        }
        validate_method(method)?;
        let request = Self {
            request_id,
            method: method.to_owned(),
            params,
        };
        let bytes = request.json().len();
        if bytes > MAX_REQUEST_BYTES {
            return Err(RequestError::TooLarge { bytes });
        }
        Ok(request)
    }

    pub fn request_id(&self) -> &str {
        &self.request_id
    }

    pub fn method(&self) -> &str {
        &self.method
    }

    pub fn params(&self) -> &Value {
        &self.params
    }

    /// The compact JSON envelope the command carries.
    pub fn json(&self) -> String {
        json!({
            "schema": SCHEMA,
            "requestId": self.request_id,
            "method": self.method,
            "params": self.params,
        })
        .to_string()
    }

    /// The `prompt` message: the extension command, a space, then the envelope.
    pub fn prompt(&self) -> String {
        format!("{COMMAND} {}", self.json())
    }

    /// The ordinary Pi command that carries this request.
    pub fn command(&self) -> RpcCommand {
        RpcCommand::prompt(self.prompt())
    }
}

/// A method name the contract accepts: lowercase snake case, 1..=64 characters.
fn validate_method(method: &str) -> Result<(), RequestError> {
    let chars = method.chars().count();
    if chars == 0 {
        return Err(RequestError::EmptyMethod);
    }
    if chars > MAX_METHOD_CHARS {
        return Err(RequestError::MethodTooLong { chars });
    }
    let mut letters = method.chars();
    let first = letters.next().expect("a non-empty method");
    let snake_case = first.is_ascii_lowercase()
        && letters
            .all(|letter| letter.is_ascii_lowercase() || letter.is_ascii_digit() || letter == '_');
    if !snake_case {
        return Err(RequestError::MethodNotSnakeCase {
            method: method.to_owned(),
        });
    }
    Ok(())
}

/// Why a request never left: the contract's limits, checked early.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestError {
    EmptyRequestId,
    RequestIdTooLong { chars: usize },
    EmptyMethod,
    MethodTooLong { chars: usize },
    MethodNotSnakeCase { method: String },
    TooLarge { bytes: usize },
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyRequestId => f.write_str("a desktop request needs a request id"),
            Self::RequestIdTooLong { chars } => write!(
                f,
                "the request id is {chars} characters; the channel allows {MAX_REQUEST_ID_CHARS}"
            ),
            Self::EmptyMethod => f.write_str("a desktop request needs a method name"),
            Self::MethodTooLong { chars } => write!(
                f,
                "the method name is {chars} characters; the channel allows {MAX_METHOD_CHARS}"
            ),
            Self::MethodNotSnakeCase { method } => {
                write!(f, "the method {method:?} is not lowercase snake case")
            }
            Self::TooLarge { bytes } => write!(
                f,
                "the request is {bytes} bytes; the channel allows {MAX_REQUEST_BYTES}"
            ),
        }
    }
}

impl std::error::Error for RequestError {}

/// The `error` object of a rejected request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopError {
    pub code: String,
    pub message: String,
}

/// One answer envelope (`kind: "response"`), correlated by `request_id`.
#[derive(Debug, Clone, PartialEq)]
pub struct DesktopResponse {
    /// `None` when the child answered `requestId: null` (it could not
    /// correlate), which is never an answer to a specific request.
    pub request_id: Option<String>,
    pub outcome: Result<Value, DesktopError>,
}

/// One pushed envelope (`kind: "event"`). Events carry a sequence, never a
/// request id: they belong to the channel, not to a question.
#[derive(Debug, Clone, PartialEq)]
pub struct DesktopEvent {
    pub seq: u64,
    pub event: String,
    pub data: Value,
}

/// What one `gentle-desktop-rpc` widget payload is.
#[derive(Debug, Clone, PartialEq)]
pub enum WidgetPayload {
    Response(DesktopResponse),
    Event(DesktopEvent),
}

/// The one JSON line the desktop widget carries, from the raw
/// `extension_ui_request` frame. `None` for another widget key, a clearing
/// frame (no lines), or a line that is not a string.
pub fn widget_payload(raw: &Value) -> Option<&str> {
    if raw.get("widgetKey").and_then(Value::as_str) != Some(WIDGET_KEY) {
        return None;
    }
    raw.get("widgetLines")?.as_array()?.first()?.as_str()
}

/// Parses one widget payload. `None` for a foreign schema, a payload that is
/// not JSON, or an envelope this contract does not model: an answer we cannot
/// trust is no answer, never a guess.
pub fn parse_widget_payload(payload: &str) -> Option<WidgetPayload> {
    let value: Value = serde_json::from_str(payload).ok()?;
    let object = value.as_object()?;
    if object.get("schema").and_then(Value::as_str) != Some(SCHEMA) {
        return None;
    }
    match object.get("kind").and_then(Value::as_str)? {
        "response" => parse_response(object).map(WidgetPayload::Response),
        "event" => parse_event(object).map(WidgetPayload::Event),
        _ => None,
    }
}

fn parse_response(object: &Map<String, Value>) -> Option<DesktopResponse> {
    let ok = object.get("ok")?.as_bool()?;
    let request_id = object
        .get("requestId")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let outcome = if ok {
        Ok(object.get("data").cloned().unwrap_or(Value::Null))
    } else {
        let error = object.get("error")?.as_object()?;
        Err(DesktopError {
            code: error.get("code")?.as_str()?.to_owned(),
            message: error.get("message")?.as_str()?.to_owned(),
        })
    };
    Some(DesktopResponse {
        request_id,
        outcome,
    })
}

fn parse_event(object: &Map<String, Value>) -> Option<DesktopEvent> {
    Some(DesktopEvent {
        seq: object.get("seq")?.as_u64()?,
        event: object.get("event")?.as_str()?.to_owned(),
        data: object.get("data").cloned().unwrap_or(Value::Null),
    })
}

/// What one child's `capabilities` answer says it implements.
///
/// Clients must not infer support from a later specification version, so this
/// is the only source of truth and an empty contract supports nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DesktopCapabilities {
    pub methods: BTreeSet<String>,
    pub events: BTreeSet<String>,
}

impl DesktopCapabilities {
    /// The contract a `capabilities` answer's `data` reports.
    pub fn from_data(data: &Value) -> Option<Self> {
        let object = data.as_object()?;
        Some(Self {
            methods: names(object.get("methods")),
            events: names(object.get("events")),
        })
    }

    /// Whether the child said it implements `method`.
    pub fn supports(&self, method: &str) -> bool {
        self.methods.contains(method)
    }
}

fn names(list: Option<&Value>) -> BTreeSet<String> {
    list.and_then(Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// What one bounded round trip brought back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelReply {
    /// The payload of the widget frame that carried this request's `requestId`.
    Widget(String),
    /// The child never answered before the deadline.
    Timeout,
    /// The host is not an interactive RPC host: the channel is inactive there.
    Inactive,
    /// The child closed, or the request could not be written to it.
    Closed,
}

/// Where a request really goes: the one seam the real child and an in-memory
/// fake both implement.
///
/// [`DesktopChannel::exchange`] writes the request as an ordinary prompt and
/// waits, bounded, for the widget frame whose payload correlates with it. It
/// never blocks past its own deadline, never panics, and reports every failure
/// as a [`ChannelReply`] so the caller decides what to do.
pub trait DesktopChannel: Send + Sync {
    fn exchange(&self, request: &DesktopRequest) -> ChannelReply;

    /// [`DesktopChannel::exchange`] under a deadline THIS ask names.
    ///
    /// The channel's own bound is one default for every method; a method whose
    /// bound is longer (pi's `get_update_status` with `refresh: true` is bounded
    /// at 15 s, while the desktop channel's default is 5 s) names its own here.
    /// The default ignores the deadline and is `exchange`, so a channel with one
    /// fixed bound - a test's fake - keeps working and keeps its own timeout;
    /// the real channel overrides it, and an ask that overruns answers
    /// [`ChannelReply::Timeout`] exactly like a plain one.
    fn exchange_within(&self, request: &DesktopRequest, deadline: Duration) -> ChannelReply {
        let _ = deadline;
        self.exchange(request)
    }
}

/// A shared channel is a channel, so a test (or a caller that shares one) can
/// hold the real thing behind an `Arc` without a wrapper type.
impl<T: DesktopChannel + ?Sized> DesktopChannel for std::sync::Arc<T> {
    fn exchange(&self, request: &DesktopRequest) -> ChannelReply {
        (**self).exchange(request)
    }

    /// Forwarded, so a shared channel keeps the deadline an ask names instead of
    /// silently falling back to its default.
    fn exchange_within(&self, request: &DesktopRequest, deadline: Duration) -> ChannelReply {
        (**self).exchange_within(request, deadline)
    }
}

/// Why a request produced no usable answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoAnswer {
    /// The child never answered before the deadline (it does not know the
    /// command, or the prompt never reached the extension).
    TimedOut,
    /// The host is not an interactive RPC host.
    Inactive,
    /// The child closed before answering.
    Closed,
    /// A widget payload arrived but was not a `gentle-shell.desktop-rpc/v1`
    /// envelope.
    Malformed,
    /// A response envelope arrived for another request id (or none).
    Uncorrelated,
    /// `capabilities` was never resolved, so no method is known to work.
    NotConnected,
}

/// What one ask produced. Every failure is typed; nothing here is an error the
/// user sees.
#[derive(Debug, Clone, PartialEq)]
pub enum DesktopOutcome {
    /// A correlated `ok: true` answer.
    Data { request_id: String, data: Value },
    /// A correlated `ok: false` answer: the child rejected the request and says
    /// why (`invalid_request`, `unsupported_method`, `revision_conflict`, ...).
    Rejected {
        request_id: Option<String>,
        error: DesktopError,
    },
    /// The child's own `capabilities` does not list this method.
    Unsupported { method: String },
    /// No usable answer, for one of the reasons [`NoAnswer`] names.
    NoAnswer(NoAnswer),
    /// The request itself broke a limit, so it was never sent.
    Invalid(RequestError),
}

/// One chat's client: it numbers requests, caches what the child reports, and
/// refuses methods the child does not advertise.
pub struct DesktopClient<C: DesktopChannel> {
    channel: C,
    capabilities: Option<DesktopCapabilities>,
    asked: u64,
}

impl<C: DesktopChannel> DesktopClient<C> {
    pub fn new(channel: C) -> Self {
        Self {
            channel,
            capabilities: None,
            asked: 0,
        }
    }

    /// The contract the child reported, once [`DesktopClient::connect`] got it.
    pub fn capabilities(&self) -> Option<&DesktopCapabilities> {
        self.capabilities.as_ref()
    }

    /// Whether the cached contract advertises `method`.
    pub fn supports(&self, method: &str) -> bool {
        self.capabilities
            .as_ref()
            .is_some_and(|capabilities| capabilities.supports(method))
    }

    /// Asks `capabilities` and caches what the child reports.
    ///
    /// This is the first call of a session: until it succeeds no method is
    /// known to exist, and [`DesktopClient::ask`] refuses everything (the spec's
    /// "clients must not infer support"). A failed call leaves the client
    /// unconnected, exactly as it was.
    pub fn connect(&mut self) -> DesktopOutcome {
        self.resolve(None)
    }

    /// Asks `capabilities` under the deadline named for THIS ask, for a caller
    /// whose child may still be starting.
    ///
    /// A freshly spawned child answers no desktop request until it is ready, and
    /// every request sent before that is answered at once when it becomes ready -
    /// so a plain 5 s connect fails on a child that is a few seconds away, and a
    /// caller with its own longer bound (the explicit packages check) names it
    /// here. Everything else is [`DesktopClient::connect`]'s: the same cache, the
    /// same leave-it-unconnected failure.
    pub fn connect_within(&mut self, deadline: Duration) -> DesktopOutcome {
        self.resolve(Some(deadline))
    }

    /// The one `capabilities` exchange both [`DesktopClient::connect`] and
    /// [`DesktopClient::connect_within`] make.
    fn resolve(&mut self, deadline: Option<Duration>) -> DesktopOutcome {
        let outcome = self.send(CAPABILITIES, json!({}), deadline);
        self.capabilities = match &outcome {
            DesktopOutcome::Data { data, .. } => DesktopCapabilities::from_data(data),
            _ => None,
        };
        outcome
    }

    /// Asks one method with `params`.
    ///
    /// Refused, without writing anything, when the request breaks a contract
    /// limit, when `capabilities` was never resolved, or when the cached
    /// contract does not list the method. Everything else is one round trip.
    pub fn ask(&mut self, method: &str, params: Value) -> DesktopOutcome {
        self.send(method, params, None)
    }

    /// Asks one method with `params` under the deadline named for THIS ask, for a
    /// method whose own bound is longer than the channel's default.
    ///
    /// Everything else is [`DesktopClient::ask`]'s: the same contract limits, the
    /// same cached capabilities, one round trip - and an overrun is the same
    /// [`NoAnswer::TimedOut`], so a caller that words its own deadline reads it
    /// without learning a second failure shape.
    pub fn ask_within(
        &mut self,
        method: &str,
        params: Value,
        deadline: Duration,
    ) -> DesktopOutcome {
        self.send(method, params, Some(deadline))
    }

    fn send(&mut self, method: &str, params: Value, deadline: Option<Duration>) -> DesktopOutcome {
        let request =
            match DesktopRequest::new(format!("desktop-{}", self.asked + 1), method, params) {
                Ok(request) => request,
                Err(error) => return DesktopOutcome::Invalid(error),
            };
        if method != CAPABILITIES {
            match &self.capabilities {
                None => return DesktopOutcome::NoAnswer(NoAnswer::NotConnected),
                Some(capabilities) if !capabilities.supports(method) => {
                    return DesktopOutcome::Unsupported {
                        method: method.to_owned(),
                    }
                }
                Some(_) => {}
            }
        }
        self.asked += 1;
        let reply = match deadline {
            Some(deadline) => self.channel.exchange_within(&request, deadline),
            None => self.channel.exchange(&request),
        };
        interpret(&request, reply)
    }
}

/// Reads one reply into a typed outcome, correlating by the payload's own
/// `requestId` (never the outer Pi prompt id).
fn interpret(request: &DesktopRequest, reply: ChannelReply) -> DesktopOutcome {
    let payload = match reply {
        ChannelReply::Widget(payload) => payload,
        ChannelReply::Timeout => return DesktopOutcome::NoAnswer(NoAnswer::TimedOut),
        ChannelReply::Inactive => return DesktopOutcome::NoAnswer(NoAnswer::Inactive),
        ChannelReply::Closed => return DesktopOutcome::NoAnswer(NoAnswer::Closed),
    };
    match parse_widget_payload(&payload) {
        Some(WidgetPayload::Response(response)) => {
            if response.request_id.as_deref() != Some(request.request_id()) {
                return DesktopOutcome::NoAnswer(NoAnswer::Uncorrelated);
            }
            match response.outcome {
                Ok(data) => DesktopOutcome::Data {
                    request_id: request.request_id().to_owned(),
                    data,
                },
                Err(error) => DesktopOutcome::Rejected {
                    request_id: response.request_id,
                    error,
                },
            }
        }
        // An event belongs to the channel, not to this question; a payload we
        // cannot parse is no answer either.
        Some(WidgetPayload::Event(_)) => DesktopOutcome::NoAnswer(NoAnswer::Uncorrelated),
        None => DesktopOutcome::NoAnswer(NoAnswer::Malformed),
    }
}
