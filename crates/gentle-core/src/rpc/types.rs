//! Pure protocol types for pi's `--mode rpc` JSON-line protocol.
//!
//! Only what a chat window needs is modeled. Any other event decodes to
//! `Decoded::Unknown` instead of being typed.

use serde_json::Value;

/// A command written to the child's stdin.
#[derive(Debug, Clone, PartialEq)]
pub enum RpcCommand {
    Prompt {
        message: String,
    },
    Abort,
    NewSession,
    SwitchSession {
        session_path: String,
    },
    GetState,
    /// `id` is the optional request/response correlation id.
    GetMessages {
        id: Option<String>,
    },
    UiResponse {
        id: String,
        answer: UiAnswer,
    },
}

/// How a blocking extension dialog is answered.
#[derive(Debug, Clone, PartialEq)]
pub enum UiAnswer {
    Value(String),
    Confirmed(bool),
    Cancelled,
}

/// An `AgentMessage`. Only `role` is modeled; `raw` keeps the full payload so
/// callers read `content` defensively instead of typing every variant.
#[derive(Debug, Clone, PartialEq)]
pub struct RpcMessage {
    pub role: String,
    pub raw: Value,
}

/// RPC-mode `message_update` delta (`assistantMessageEvent`).
#[derive(Debug, Clone, PartialEq)]
pub enum AssistantMessageEvent {
    TextStart {
        content_index: u64,
    },
    TextDelta {
        content_index: u64,
        delta: String,
    },
    TextEnd {
        content_index: u64,
        content: String,
    },
    ThinkingStart {
        content_index: u64,
    },
    ThinkingDelta {
        content_index: u64,
        delta: String,
    },
    ThinkingEnd {
        content_index: u64,
        content: String,
    },
    ToolcallStart {
        content_index: u64,
        id: String,
        tool_name: String,
    },
    ToolcallDelta {
        content_index: u64,
        delta: String,
    },
    ToolcallEnd {
        content_index: u64,
        tool_call: Value,
    },
}

/// An extension UI request. Dialog methods block for a response; the rest are
/// fire-and-forget and keep their raw payload.
#[derive(Debug, Clone, PartialEq)]
pub enum UiRequest {
    Select {
        id: String,
        title: String,
        options: Vec<String>,
        timeout: Option<u64>,
    },
    Confirm {
        id: String,
        title: String,
        message: String,
        timeout: Option<u64>,
    },
    Input {
        id: String,
        title: String,
        placeholder: Option<String>,
        timeout: Option<u64>,
    },
    Editor {
        id: String,
        title: String,
        prefill: Option<String>,
    },
    /// notify, setStatus, setWidget, setTitle, set_editor_text.
    FireAndForget {
        id: String,
        method: String,
        raw: Value,
    },
}

/// Result of a command, correlated by the optional `id`.
#[derive(Debug, Clone, PartialEq)]
pub struct RpcResponse {
    pub id: Option<String>,
    pub command: String,
    pub outcome: Result<Value, String>,
}

/// One event read from the child's stdout.
#[derive(Debug, Clone, PartialEq)]
pub enum RpcEvent {
    AgentStart,
    AgentSettled,
    TurnStart,
    TurnEnd,
    AgentEnd {
        will_retry: Option<bool>,
    },
    MessageStart(RpcMessage),
    MessageEnd(RpcMessage),
    MessageUpdate(AssistantMessageEvent),
    ToolExecutionStart {
        tool_call_id: String,
        tool_name: String,
    },
    ToolExecutionUpdate {
        tool_call_id: String,
        tool_name: String,
    },
    ToolExecutionEnd {
        tool_call_id: String,
        tool_name: String,
        is_error: bool,
    },
    UiRequest(UiRequest),
    ExtensionError {
        extension_path: String,
        event: String,
        error: String,
    },
    Response(RpcResponse),
}

/// A decoded line: a modeled event, or the raw text of anything else
/// (blank, malformed JSON, or a valid event this client does not model).
#[derive(Debug, Clone, PartialEq)]
pub enum Decoded {
    Event(RpcEvent),
    Unknown(String),
}
