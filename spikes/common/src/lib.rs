//! Shared input for the UI spikes: a scripted RPC stream that exercises the
//! real `gentle-core` decoder and reducer, so both frameworks render exactly
//! the same chat states in the same order and at the same pace.

use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

use gentle_core::chat::{reduce_chat, ChatMessage, ChatRole, ChatState};
use gentle_core::rpc::{decode_line, Decoded};

pub const REPLY_1: &str = "# Plan\n\nPara **medir** el spike necesito:\n\n- un hilo de mensajes con `scroll`\n- markdown con *énfasis* y [links](https://example.com)\n- streaming sin parpadeos\n\n```rust\nfn main() {\n    println!(\"hola gentle-desk\");\n}\n```\n\n> Una cita para ver el estilo de blockquote.\n";

pub const REPLY_2: &str =
    "Segunda respuesta, más corta: `cargo test` pasa. ✅ Acentos, ñ y emoji 🚀 incluidos.\n";

/// One scripted step: wait, then the child "prints" `line`.
pub struct Step {
    pub delay: Duration,
    pub line: String,
}

fn json_string(text: &str) -> String {
    let mut out = String::from("\"");
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The wire lines of one streamed assistant reply, split in small deltas.
fn reply_steps(text: &str, chunk_chars: usize, tick: Duration) -> Vec<Step> {
    let step = |delay, line: String| Step { delay, line };
    let mut steps = vec![
        step(
            Duration::from_millis(300),
            r#"{"type":"agent_start"}"#.into(),
        ),
        step(
            Duration::ZERO,
            r#"{"type":"message_start","message":{"role":"assistant","content":[]}}"#.into(),
        ),
    ];
    let chars: Vec<char> = text.chars().collect();
    for piece in chars.chunks(chunk_chars) {
        let delta: String = piece.iter().collect();
        steps.push(step(
            tick,
            format!(
                r#"{{"type":"message_update","assistantMessageEvent":{{"type":"text_delta","contentIndex":0,"delta":{}}}}}"#,
                json_string(&delta)
            ),
        ));
    }
    steps.push(step(
        tick,
        format!(
            r#"{{"type":"message_end","message":{{"role":"assistant","content":[{{"type":"text","text":{}}}]}}}}"#,
            json_string(text)
        ),
    ));
    steps.push(step(Duration::ZERO, r#"{"type":"agent_settled"}"#.into()));
    steps
}

/// The whole conversation: reply 1, a pause, reply 2.
pub fn script(chunk_chars: usize, tick: Duration) -> Vec<Step> {
    let mut steps = reply_steps(REPLY_1, chunk_chars, tick);
    let mut second = reply_steps(REPLY_2, chunk_chars, tick);
    second[0].delay = Duration::from_millis(1200);
    steps.extend(second);
    steps
}

/// The app layer, not the reducer, owns user messages: it appends them when
/// the user sends a prompt.
pub fn push_user(mut state: ChatState, text: &str) -> ChatState {
    state.messages.push(ChatMessage {
        id: format!("msg-{}", state.messages.len()),
        role: ChatRole::User,
        text: text.to_owned(),
        streaming: false,
        error: None,
    });
    state
}

/// Runs the script on a thread, folding every line through the real decoder
/// and reducer, and sends each resulting state to `on_state`.
pub fn spawn_feed<F>(on_state: F, chunk_chars: usize, tick: Duration) -> thread::JoinHandle<()>
where
    F: Fn(ChatState) + Send + 'static,
{
    thread::spawn(move || {
        let mut state = push_user(ChatState::default(), "Armá el plan del spike de UI");
        on_state(state.clone());
        let mut second_user_pushed = false;
        for step in script(chunk_chars, tick) {
            thread::sleep(step.delay);
            if step.delay >= Duration::from_millis(1200) && !second_user_pushed {
                second_user_pushed = true;
                state = push_user(state, "Ahora algo más corto");
            }
            if let Decoded::Event(event) = decode_line(&step.line) {
                state = reduce_chat(state, &event);
                on_state(state.clone());
            }
        }
    })
}

/// For a channel-based UI loop.
pub fn feed_to_channel(tx: Sender<ChatState>, chunk_chars: usize, tick: Duration) {
    spawn_feed(
        move |state| {
            let _ = tx.send(state);
        },
        chunk_chars,
        tick,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;

    #[test]
    fn every_scripted_line_decodes_to_an_event() {
        for step in script(4, Duration::ZERO) {
            assert!(
                matches!(decode_line(&step.line), Decoded::Event(_)),
                "{}",
                step.line
            );
        }
    }

    #[test]
    fn the_feed_ends_with_both_replies_complete_and_two_user_messages() {
        let (tx, rx) = mpsc::channel();
        feed_to_channel(tx, 7, Duration::ZERO);
        // Zero delays except the pauses: wait for the sender to hang up.
        let last = rx.iter().last().expect("states");
        let texts: Vec<_> = last
            .messages
            .iter()
            .map(|m| (m.role, m.text.as_str()))
            .collect();
        assert_eq!(texts.len(), 4);
        assert_eq!(texts[1], (ChatRole::Assistant, REPLY_1));
        assert_eq!(texts[3], (ChatRole::Assistant, REPLY_2));
        assert!(last.messages.iter().all(|m| !m.streaming));
        assert!(!last.working);
    }
}
