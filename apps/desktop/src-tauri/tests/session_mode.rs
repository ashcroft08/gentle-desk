//! The session permission mode, from the desktop channel to the web's view
//! (composer-dock T3b, L7, L8).
//!
//! The mode is drawn as a label and changed only through the child's own answer
//! (L7): a read shows what the owner reports, a confirmed write replaces it and
//! counts the attempt, a refusal shows its code and leaves the mode alone, and a
//! read nobody answered draws no control at all. The rules of the shell's own
//! state are what these tests pin, with no child and no window: the commands
//! arrive from the holder's own dispatch and from the recording fake of
//! `chat_seam`, and the answers arrive as the channel's typed outcomes.

mod chat_seam;
use chat_seam::{stored, Answer, Fake};

use std::sync::{Arc, Mutex, MutexGuard};

use gentle_app::{ChatCommand, ChatId, Input, Intent};
use gentle_core::rpc::permission::{PermissionOutcome, PermissionState, SessionMode};
use gentle_core::rpc::{DesktopError, NoAnswer};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::commands::{fold_spawn_failure, Emit};
use gentle_desk_lib::contract::{SessionModeChoice, SessionModeView, UiIntent, ViewState};
use gentle_desk_lib::effects;

/// The state an answered exchange carries, in the channel's own two strings.
fn answered(mode: &str) -> PermissionOutcome {
    PermissionOutcome::Answered(PermissionState {
        mode: mode.to_owned(),
        default_mode: "normal".to_owned(),
    })
}

/// The owner's refusal, as the typed outcome names it.
fn refused(code: &str) -> PermissionOutcome {
    PermissionOutcome::Rejected(DesktopError {
        code: code.to_owned(),
        message: format!("the child refused: {code}"),
    })
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// The holder with the stored chat opened and confirmed: foreground and idle.
fn open_stored() -> (AppStateHolder, ChatId) {
    let holder = AppStateHolder::default();
    let chat = stored();
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    (holder, chat)
}

/// The session mode the slice carries right now.
fn mode_view(holder: &AppStateHolder) -> Option<SessionModeView> {
    holder.view().session_mode
}

/// The one slice a failure pushed, as the page receives it.
fn recorder() -> (Emit, Arc<Mutex<Vec<ViewState>>>) {
    let views = Arc::new(Mutex::new(Vec::new()));
    let kept = Arc::clone(&views);
    let emit: Emit = Arc::new(move |view: &ViewState| {
        lock(&kept).push(view.clone());
        Ok(())
    });
    (emit, views)
}

#[test]
fn the_read_intent_asks_that_chats_own_child_and_nothing_is_drawn_yet() {
    let (holder, chat) = open_stored();

    let (view, commands) = holder.dispatch_commands(UiIntent::ReadSessionMode {
        chat_id: chat.as_str().to_owned(),
    });

    assert_eq!(
        commands,
        vec![ChatCommand::SessionMode {
            chat: chat.clone(),
            change: None,
        }],
        "one read, of the one chat the intent named"
    );
    assert_eq!(
        view.session_mode, None,
        "no answer yet, so no control is drawn"
    );
}

#[test]
fn an_answered_read_shows_the_mode_the_child_reported() {
    let (holder, chat) = open_stored();

    let pushed = holder
        .note_session_mode(&chat, None, &answered("auto"))
        .expect("the read that draws the mode");

    assert_eq!(
        pushed.session_mode,
        Some(SessionModeView {
            mode: "auto".to_owned(),
            refusal: None,
            attempt: 0,
        }),
        "the owner's own string is what the label shows, and a read is no attempt"
    );
}

#[test]
fn a_mode_this_build_does_not_know_is_shown_as_it_arrived() {
    let (holder, chat) = open_stored();

    let _ = holder.note_session_mode(&chat, None, &answered("turbo"));

    assert_eq!(
        mode_view(&holder).expect("a mode was reported").mode,
        "turbo",
        "the raw string is shown as-is, never hidden and never guessed at"
    );
}

#[test]
fn a_confirmed_write_is_the_new_mode_and_the_attempt() {
    let (holder, chat) = open_stored();
    let _ = holder.note_session_mode(&chat, None, &answered("normal"));

    let pushed = holder
        .note_session_mode(&chat, Some(SessionMode::Yolo), &answered("yolo"))
        .expect("the write's own answer");

    assert_eq!(
        pushed.session_mode,
        Some(SessionModeView {
            mode: "yolo".to_owned(),
            refusal: None,
            attempt: 1,
        }),
        "the shown mode changes only on the confirmed answer (L7), and the write counts"
    );
}

#[test]
fn a_refused_write_keeps_the_mode_and_says_the_code() {
    let (holder, chat) = open_stored();
    let _ = holder.note_session_mode(&chat, None, &answered("normal"));

    let pushed = holder
        .note_session_mode(
            &chat,
            Some(SessionMode::SuperYolo),
            &refused("permission_denied"),
        )
        .expect("a refusal is the click's answer");

    assert_eq!(
        pushed.session_mode,
        Some(SessionModeView {
            mode: "normal".to_owned(),
            refusal: Some("permission_denied".to_owned()),
            attempt: 1,
        }),
        "the failure shows its code and leaves the mode alone (L7)"
    );
}

#[test]
fn a_read_the_child_did_not_answer_draws_no_control() {
    let (holder, chat) = open_stored();

    for outcome in [
        PermissionOutcome::Unsupported,
        PermissionOutcome::NoAnswer(NoAnswer::NotConnected),
        refused("permission_unavailable"),
    ] {
        let pushed = holder.note_session_mode(&chat, None, &outcome);

        assert!(
            pushed.is_some(),
            "the read's own answer comes back to settle it: {outcome:?}"
        );
        assert_eq!(
            mode_view(&holder),
            None,
            "a read that did not answer draws nothing (no control, no invented mode)"
        );
    }
}

#[test]
fn a_write_the_child_did_not_answer_keeps_the_mode_and_says_so() {
    let (holder, chat) = open_stored();
    let _ = holder.note_session_mode(&chat, None, &answered("normal"));

    for (outcome, code) in [
        (
            PermissionOutcome::NoAnswer(NoAnswer::TimedOut),
            "no_answer",
        ),
        (PermissionOutcome::Unsupported, "unsupported"),
    ] {
        let _ =
            holder.note_session_mode(&chat, Some(SessionMode::Auto), &outcome);

        assert_eq!(
            mode_view(&holder),
            Some(SessionModeView {
                mode: "normal".to_owned(),
                refusal: Some(code.to_owned()),
                attempt: 0,
            }),
            "nothing was written, so the mode stays and the case is named"
        );
    }
}

#[test]
fn a_successful_read_clears_the_refusal() {
    let (holder, chat) = open_stored();
    let _ = holder.note_session_mode(&chat, None, &answered("normal"));
    let _ = holder.note_session_mode(&chat, Some(SessionMode::Auto), &refused("invalid_request"));

    let pushed = holder
        .note_session_mode(&chat, None, &answered("auto"))
        .expect("the read that answers again");

    assert_eq!(
        pushed.session_mode,
        Some(SessionModeView {
            mode: "auto".to_owned(),
            refusal: None,
            attempt: 1,
        }),
        "the next answer is the truth the label draws"
    );
}

#[test]
fn the_mode_is_the_foreground_chats_and_nobody_elses() {
    let (holder, chat) = open_stored();
    let other = ChatId::of_session("/sessions/b.jsonl");
    holder.fold_input(Input::Intent(Intent::OpenChat(other.clone())));
    holder.fold_input(Input::ChatOpened {
        chat: other.clone(),
    });

    let _ = holder.note_session_mode(&chat, None, &answered("auto"));

    assert_eq!(
        mode_view(&holder),
        None,
        "the background chat's mode is never drawn on the foreground one"
    );
    let _ = holder.note_session_mode(&other, None, &answered("yolo"));
    assert_eq!(
        mode_view(&holder).map(|view| view.mode),
        Some("yolo".to_owned()),
        "and the foreground chat's own answer is drawn"
    );
}

#[test]
fn a_late_answer_for_a_closed_chat_is_dropped() {
    let (holder, chat) = open_stored();
    holder.fold_input(Input::Intent(Intent::CloseChat(chat.clone())));
    holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: true,
    });

    let pushed = holder.note_session_mode(&chat, None, &answered("auto"));

    assert!(
        pushed.is_none(),
        "a chat that closed answers nothing, and pushes nothing"
    );
    assert_eq!(mode_view(&holder), None, "and nothing of it comes back");
}

#[test]
fn a_second_command_while_one_is_in_flight_is_dropped() {
    let (holder, chat) = open_stored();
    let read = UiIntent::ReadSessionMode {
        chat_id: chat.as_str().to_owned(),
    };
    let (_, first) = holder.dispatch_commands(read.clone());
    assert_eq!(first.len(), 1, "the first read runs");

    let (_, second) = holder.dispatch_commands(read);
    assert!(
        second.is_empty(),
        "one session-mode command per chat at a time: the second is dropped, not queued"
    );

    let _ = holder.note_session_mode(&chat, None, &answered("auto"));
    let (_, after) = holder.dispatch_commands(UiIntent::SetSessionMode {
        chat_id: chat.as_str().to_owned(),
        mode: SessionModeChoice::Auto,
    });
    assert_eq!(
        after,
        vec![ChatCommand::SessionMode {
            chat: chat.clone(),
            change: Some(SessionMode::Auto),
        }],
        "the answer frees the slot, so the next ask runs"
    );
}

#[test]
fn the_write_intent_carries_the_channels_own_mode() {
    let (holder, chat) = open_stored();

    let (_view, commands) = holder.dispatch_commands(UiIntent::SetSessionMode {
        chat_id: chat.as_str().to_owned(),
        mode: SessionModeChoice::SuperYolo,
    });

    assert_eq!(
        commands,
        vec![ChatCommand::SessionMode {
            chat: chat.clone(),
            change: Some(SessionMode::SuperYolo),
        }],
        "the closed choice maps to the channel's own mode, which spells super-yolo"
    );
    assert_eq!(SessionMode::SuperYolo.as_str(), "super-yolo");
}

#[test]
fn the_fold_carries_the_answer_beside_the_inputs() {
    let (_holder, chat) = open_stored();
    let fake = Fake::answering(Answer::SessionMode(Box::new(answered("yolo"))));

    let ran = effects::run_one(
        ChatCommand::SessionMode {
            chat: chat.clone(),
            change: None,
        },
        &fake,
    );

    assert!(
        ran.inputs.is_empty(),
        "the reducer knows nothing about a permission mode"
    );
    assert_eq!(ran.session_mode.len(), 1, "one command, one answer");
    let (named, change, outcome) = ran.session_mode.into_iter().next().expect("the answer");
    assert_eq!(named, chat);
    assert_eq!(change, None);
    assert_eq!(outcome, answered("yolo"));
}

#[test]
fn a_write_that_could_not_start_says_no_answer_and_a_read_says_nothing() {
    let (holder, chat) = open_stored();
    let holder = Arc::new(holder);
    let _ = holder.note_session_mode(&chat, None, &answered("normal"));
    // The intent took the chat's one in-flight slot before the worker thread
    // that would have run the write could not be created.
    let _ = holder.dispatch_commands(UiIntent::SetSessionMode {
        chat_id: chat.as_str().to_owned(),
        mode: SessionModeChoice::Yolo,
    });
    let (emit, pushed) = recorder();

    fold_spawn_failure(
        &holder,
        &emit,
        &ChatCommand::SessionMode {
            chat: chat.clone(),
            change: Some(SessionMode::Yolo),
        },
    );

    let views = lock(&pushed);
    assert_eq!(views.len(), 1, "the click is answered with one slice");
    assert_eq!(
        views[0].session_mode,
        Some(SessionModeView {
            mode: "normal".to_owned(),
            refusal: Some("no_answer".to_owned()),
            attempt: 0,
        }),
        "nothing was written, so the mode stays and the case is named"
    );
    drop(views);
    let (_, again) = holder.dispatch_commands(UiIntent::SetSessionMode {
        chat_id: chat.as_str().to_owned(),
        mode: SessionModeChoice::Yolo,
    });
    assert_eq!(again.len(), 1, "and the slot came free with that answer");

    let (emit, pushed) = recorder();
    fold_spawn_failure(
        &holder,
        &emit,
        &ChatCommand::SessionMode {
            chat: chat.clone(),
            change: None,
        },
    );
    assert!(
        lock(&pushed).is_empty(),
        "a read that never ran changes nothing the page draws"
    );
    assert_eq!(
        mode_view(&holder).map(|view| view.refusal),
        Some(Some("no_answer".to_owned())),
        "and it says nothing about the mode it never read"
    );
}
