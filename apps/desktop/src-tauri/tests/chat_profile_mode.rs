//! Saving the used profile's orchestration mode from the Profile tab
//! (profiles-editor T3-mode, L23, L24): the intent names ONE chat and ONE
//! profile, the write is a per-chat command over that chat's own child, the
//! switch is disabled while the write is in flight, and a write that did not
//! happen leaves one short notice that the next successful read clears.
//!
//! No child and no window run here: the executor is the recording fake of
//! `chat_seam`, and the reads arrive as the channel's documented frame.

mod chat_seam;
use chat_seam::{stored, Answer, Fake};

use gentle_app::{ChatCommand, ChatId, Input, Intent};
use gentle_core::rpc::config_store::{
    self, OrchestratorMode, SetConfigOutcome, StoreId, StoreValue,
};
use gentle_core::rpc::{DesktopError, NoAnswer};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{OrchestratorModeView, UiIntent};
use gentle_desk_lib::effects;
use gentle_runtime::chat_runtime::{ChatConfigRead, ModeWriteResult, StoreRead};

/// The channel's own documented frames, quoted from `desktop-rpc.md` in that
/// fixture: the profiles document this tab draws.
const FRAMES: &str =
    include_str!("../../../../crates/gentle-core/tests/fixtures/desktop-rpc-profiles.json");

/// One profiles store read: the documented document, and the other two stores
/// as a `get_config` that was not asked (`Skipped`) - enough to be a view.
fn read() -> ChatConfigRead {
    let frames: serde_json::Value = serde_json::from_str(FRAMES).expect("the frames are JSON");
    let parsed = config_store::parse_store_value(StoreId::Profiles, &frames["profiles_data"])
        .expect("a documented profiles document");
    let StoreValue::Profiles(document) = parsed else {
        panic!("the profiles store answered a profiles document");
    };
    ChatConfigRead {
        profiles: StoreRead::Read(document),
        ..ChatConfigRead::unasked()
    }
}

/// The holder with the stored chat opened and confirmed: foreground and idle.
fn open_stored() -> (AppStateHolder, ChatId) {
    let holder = AppStateHolder::default();
    let chat = stored();
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    (holder, chat)
}

/// A holder whose chat already has its view, so the switch it draws can be told
/// to disable itself.
fn drawn() -> (AppStateHolder, ChatId) {
    let (holder, chat) = open_stored();
    holder
        .note_chat_profile(&chat, &read())
        .expect("the read that draws the tab");
    (holder, chat)
}

/// A refusal the owner words itself, as the typed outcome names it.
fn conflict() -> ModeWriteResult {
    ModeWriteResult::failed(SetConfigOutcome::RevisionConflict(DesktopError {
        code: "revision_conflict".to_owned(),
        message: "Config changed since expectedRevision was read".to_owned(),
    }))
}

#[test]
fn the_save_intent_asks_that_chats_own_child_to_write_the_profiles_mode() {
    let (holder, chat) = open_stored();

    let (_view, commands) = holder.dispatch_commands(UiIntent::SaveChatProfileMode {
        chat_id: chat.as_str().to_owned(),
        profile: "team".to_owned(),
        mode: OrchestratorModeView::FullDelegation,
    });

    assert_eq!(
        commands,
        vec![ChatCommand::SetChatProfileMode {
            chat: chat.clone(),
            profile: "team".to_owned(),
            mode: OrchestratorMode::FullDelegation,
        }],
        "one write, for the one chat and the one profile the intent named"
    );
}

#[test]
fn the_profiles_own_words_are_the_mode_the_command_carries() {
    let (holder, chat) = open_stored();

    let (_view, commands) = holder.dispatch_commands(UiIntent::SaveChatProfileMode {
        chat_id: chat.as_str().to_owned(),
        profile: "team".to_owned(),
        mode: OrchestratorModeView::Organic,
    });

    assert!(matches!(
        &commands[0],
        ChatCommand::SetChatProfileMode {
            mode: OrchestratorMode::Organic,
            ..
        }
    ));
    assert_eq!(
        OrchestratorMode::Organic.as_str(),
        "organic",
        "the two enums spell the value the store holds, so nothing is translated"
    );
}

#[test]
fn the_slice_the_intent_answers_with_disables_the_switch() {
    let (holder, chat) = drawn();

    let (view, _commands) = holder.dispatch_commands(UiIntent::SaveChatProfileMode {
        chat_id: chat.as_str().to_owned(),
        profile: "team".to_owned(),
        mode: OrchestratorModeView::FullDelegation,
    });

    let drawn = view.chat_profile.expect("the chat's own view");
    assert!(
        drawn.saving,
        "a write is in flight: the switch is drawn disabled until it answers"
    );
    assert!(
        !drawn.save_failed,
        "nothing has failed yet: no notice before an answer"
    );
}

#[test]
fn a_failed_save_says_so_and_keeps_the_reread_view() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_mode(&chat, &conflict(), &read())
        .expect("the attempt's answer is pushed");

    let view = pushed.chat_profile.expect("the chat's own view");
    assert!(
        view.save_failed,
        "the mode could not be saved and the tab says so"
    );
    assert!(!view.saving, "the answer enabled the switch again");
    assert_eq!(
        view.profiles
            .iter()
            .map(|profile| profile.name.as_str())
            .collect::<Vec<_>>(),
        ["team"],
        "and the re-read truth is what the tab draws"
    );
}

#[test]
fn a_save_that_worked_leaves_no_notice() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_mode(&chat, &ModeWriteResult::Saved, &read())
        .expect("the attempt's answer is pushed");

    let view = pushed.chat_profile.expect("the chat's own view");
    assert!(!view.save_failed, "a save that worked says nothing");
    assert!(!view.saving);
}

#[test]
fn an_unchanged_mode_is_not_a_failure() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_mode(&chat, &ModeWriteResult::Unchanged, &read())
        .expect("the attempt's answer is pushed");

    let view = pushed.chat_profile.expect("the chat's own view");
    assert!(
        !view.save_failed,
        "the store already holds the chosen mode: nothing failed"
    );
}

#[test]
fn a_profile_the_store_does_not_define_is_reported_as_not_saved() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_mode(&chat, &ModeWriteResult::UnknownProfile, &read())
        .expect("the attempt's answer is pushed");

    assert!(
        pushed.chat_profile.expect("the view").save_failed,
        "a write that wrote nothing because the profile is gone is still not saved"
    );
}

#[test]
fn the_notice_goes_away_on_the_next_successful_read() {
    let (holder, chat) = drawn();
    holder.note_chat_mode(&chat, &conflict(), &read());
    assert!(holder.view().chat_profile.expect("the view").save_failed);

    let pushed = holder
        .note_chat_profile(&chat, &read())
        .expect("a read that reached a store is a view");

    assert!(
        !pushed.chat_profile.expect("the view").save_failed,
        "the next successful read clears the notice, with no timer and no toast"
    );
}

#[test]
fn a_read_that_reached_no_store_does_not_clear_the_notice() {
    let (holder, chat) = drawn();
    holder.note_chat_mode(&chat, &conflict(), &read());

    assert!(
        holder
            .note_chat_profile(&chat, &ChatConfigRead::unasked())
            .is_none(),
        "a read that reached nothing keeps what it drew (L11-4)"
    );
    assert!(
        holder.view().chat_profile.expect("the view").save_failed,
        "and it is not the successful read the notice waits for"
    );
}

#[test]
fn the_answer_enables_the_switch_even_when_the_reread_reached_no_store() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_mode(&chat, &conflict(), &ChatConfigRead::unasked())
        .expect("the attempt's own answer is pushed");

    let view = pushed.chat_profile.expect("the previous view stays");
    assert!(!view.saving, "the switch is usable again");
    assert!(view.save_failed, "and the notice reaches the page");
}

#[test]
fn the_fold_carries_the_save_beside_the_inputs() {
    let (_holder, chat) = drawn();
    let fake = Fake::answering(Answer::ChatMode(
        Box::new(ModeWriteResult::Saved),
        Box::new(read()),
    ));

    let ran = effects::run_one(
        ChatCommand::SetChatProfileMode {
            chat: chat.clone(),
            profile: "team".to_owned(),
            mode: OrchestratorMode::FullDelegation,
        },
        &fake,
    );

    assert!(
        ran.inputs.is_empty(),
        "the reducer knows nothing about a profile's mode (L11-1)"
    );
    assert_eq!(ran.chat_mode.len(), 1, "one command, one answer");
    let (named, result, _read) = ran.chat_mode.into_iter().next().expect("the answer");
    assert_eq!(named, chat);
    assert_eq!(result, ModeWriteResult::Saved);
    assert_eq!(
        fake.asked(),
        vec![ChatCommand::SetChatProfileMode {
            chat: chat.clone(),
            profile: "team".to_owned(),
            mode: OrchestratorMode::FullDelegation,
        }],
        "the command that ran is the one the intent asked for"
    );
}

#[test]
fn closing_the_chat_drops_the_save_state_with_its_view() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(UiIntent::SaveChatProfileMode {
        chat_id: chat.as_str().to_owned(),
        profile: "team".to_owned(),
        mode: OrchestratorModeView::FullDelegation,
    });
    assert!(holder.view().chat_profile.expect("the view").saving);

    holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: true,
    });

    assert!(
        holder.view().chat_profile.is_none(),
        "a closed chat's view is dropped (L11-4)"
    );
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    let view = holder.view().chat_profile;
    assert!(
        view.is_none(),
        "and a reopened chat starts with no save in flight: the write died with it"
    );
}

#[test]
fn a_save_that_answers_after_its_chat_closed_is_dropped() {
    let (holder, chat) = drawn();
    holder.fold_input(Input::Intent(Intent::CloseChat(chat.clone())));
    holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: true,
    });

    assert!(
        holder.note_chat_mode(&chat, &conflict(), &read()).is_none(),
        "a write that answers after its chat closed changes nothing and pushes nothing"
    );
}

#[test]
fn a_chat_with_no_child_answers_the_not_connected_reason() {
    let (holder, chat) = open_stored();

    let pushed = holder
        .note_chat_mode(
            &chat,
            &ModeWriteResult::failed(SetConfigOutcome::NoAnswer(NoAnswer::NotConnected)),
            &ChatConfigRead::unasked(),
        )
        .expect("the attempt's answer is pushed");

    assert!(
        pushed.chat_profile.is_none(),
        "that chat has no view yet, so there is no switch to complain at"
    );
}

#[test]
fn a_read_that_answers_while_a_save_is_in_flight_keeps_the_switch_disabled() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(UiIntent::SaveChatProfileMode {
        chat_id: chat.as_str().to_owned(),
        profile: "explore".to_owned(),
        mode: OrchestratorModeView::FullDelegation,
    });
    assert!(holder.view().chat_profile.expect("the view").saving);

    let pushed = holder
        .note_chat_profile(&chat, &read())
        .expect("a read that reached a store is a view");

    assert!(
        pushed.chat_profile.expect("the view").saving,
        "the write lives until its own answer (R3-001)"
    );
}
