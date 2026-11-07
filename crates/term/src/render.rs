//! Turns chat-state changes into terminal output.
//!
//! Pure: it only decides *what* to print; `main` owns the actual stdout and
//! stderr. Streaming works by remembering how much of the open assistant
//! message was already printed and emitting just the new tail.

use gentle_core::chat::{ChatRole, ChatState};
use gentle_core::rpc::{RpcEvent, UiRequest};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Out {
    /// Reply text for stdout, with no newline added.
    Text(String),
    /// End of a reply: a newline on stdout.
    EndOfReply,
    /// A model failure (credits, rate limit...), shown on stderr as a failure.
    ModelError(String),
    /// A warning or info notice from an extension, for stderr.
    Notice { level: String, text: String },
}

#[derive(Debug, Default)]
pub struct Renderer {
    /// Id of the message being streamed and how many bytes of it went out.
    printed: Option<(String, usize)>,
    error_reported: Vec<String>,
    closed: Vec<String>,
}

impl Renderer {
    pub fn new() -> Self {
        Self::default()
    }

    /// What to print now that the chat state is `state`.
    pub fn render(&mut self, state: &ChatState) -> Vec<Out> {
        let mut out = Vec::new();
        let Some(message) = state
            .messages
            .iter()
            .rev()
            .find(|m| m.role == ChatRole::Assistant)
        else {
            return out;
        };

        // A failed message is reported once, as a failure, never as reply text.
        if let Some(error) = &message.error {
            if !self.error_reported.contains(&message.id) {
                self.error_reported.push(message.id.clone());
                out.push(Out::ModelError(error.clone()));
            }
            self.printed = None;
            return out;
        }

        let already = match &self.printed {
            Some((id, len)) if *id == message.id => *len,
            _ => 0,
        };
        if message.text.len() > already {
            out.push(Out::Text(message.text[already..].to_owned()));
        }
        self.printed = Some((message.id.clone(), message.text.len()));
        if !message.streaming && !message.text.is_empty() && !self.finished(&message.id) {
            out.push(Out::EndOfReply);
            self.closed.push(message.id.clone());
        }
        out
    }

    fn finished(&self, id: &str) -> bool {
        self.closed.iter().any(|closed| closed == id)
    }
}

/// An extension `notify` worth showing; every other fire-and-forget request
/// (status bars, widgets, titles) is chrome the terminal has no place for.
pub fn notice(event: &RpcEvent) -> Option<Out> {
    let RpcEvent::UiRequest(UiRequest::FireAndForget { method, raw, .. }) = event else {
        return None;
    };
    if method != "notify" {
        return None;
    }
    let text = raw.get("message").and_then(Value::as_str)?;
    let level = raw
        .get("notifyType")
        .and_then(Value::as_str)
        .unwrap_or("info");
    Some(Out::Notice {
        level: level.to_owned(),
        text: text.to_owned(),
    })
}

#[cfg(test)]
mod tests;
