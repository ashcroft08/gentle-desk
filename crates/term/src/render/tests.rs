use super::*;
use gentle_core::chat::reduce_chat;
use gentle_core::rpc::{decode_line, Decoded};

/// Feeds raw JSONL lines through the real decoder and reducer, rendering after each.
fn run(lines: &[&str]) -> Vec<Out> {
    let mut renderer = Renderer::new();
    let mut state = ChatState::default();
    let mut out = Vec::new();
    for line in lines {
        let Decoded::Event(event) = decode_line(line) else {
            panic!("not an event: {line}")
        };
        state = reduce_chat(state, &event);
        out.extend(renderer.render(&state));
    }
    out
}

const START: &str = r#"{"type":"message_start","message":{"role":"assistant","content":[]}}"#;

fn delta(text: &str) -> String {
    format!(
        r#"{{"type":"message_update","assistantMessageEvent":{{"type":"text_delta","contentIndex":0,"delta":"{text}"}}}}"#
    )
}

#[test]
fn deltas_stream_only_their_new_tail_and_the_end_closes_the_reply_once() {
    let (d1, d2) = (delta("Ho"), delta("la"));
    let end = r#"{"type":"message_end","message":{"role":"assistant","content":[{"type":"text","text":"Hola"}],"stopReason":"stop"}}"#;
    let out = run(&[START, &d1, &d2, end, r#"{"type":"turn_end"}"#]);
    assert_eq!(
        out,
        vec![
            Out::Text("Ho".into()),
            Out::Text("la".into()),
            Out::EndOfReply
        ]
    );
}

#[test]
fn text_only_present_in_message_end_is_still_printed() {
    let end = r#"{"type":"message_end","message":{"role":"assistant","content":[{"type":"text","text":"Todo junto"}]}}"#;
    assert_eq!(
        run(&[START, end]),
        vec![Out::Text("Todo junto".into()), Out::EndOfReply]
    );
}

#[test]
fn a_provider_failure_is_a_model_error_never_reply_text() {
    let end = r#"{"type":"message_end","message":{"role":"assistant","content":[{"type":"text","text":"Fable 5.1 requires usage credits."}],"stopReason":"error","errorMessage":"Claude quota exceeded"}}"#;
    let turn_end = r#"{"type":"turn_end"}"#;
    assert_eq!(
        run(&[START, end, turn_end]),
        vec![Out::ModelError("Claude quota exceeded".into())]
    );
}

#[test]
fn a_second_reply_streams_from_its_own_start() {
    let end1 = r#"{"type":"message_end","message":{"role":"assistant","content":[{"type":"text","text":"uno"}]}}"#;
    let (d1, d2) = (delta("uno"), delta("dos"));
    let end2 = r#"{"type":"message_end","message":{"role":"assistant","content":[{"type":"text","text":"dos"}]}}"#;
    assert_eq!(
        run(&[START, &d1, end1, START, &d2, end2]),
        vec![
            Out::Text("uno".into()),
            Out::EndOfReply,
            Out::Text("dos".into()),
            Out::EndOfReply
        ]
    );
}
