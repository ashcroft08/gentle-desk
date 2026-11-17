//! A chat command whose worker thread cannot be created still answers the web
//! (profiles-editor L17(a), the liveness hole, L18).
//!
//! `run_chat_commands` starts ONE worker thread per chat command, and creating a
//! thread is an OS call that can fail on a real run ("access denied", os error
//! 5, L17). A command that never runs would otherwise be dropped with a log
//! line: the Profile tab's read would never answer at all, and a mode save would
//! leave the switch disabled for ever. So the failure is folded as a first-class
//! answer, and these tests drive that path with no child and no window: the
//! recorder is the emit seam the running app pushes `state-changed` through.
//!
//! Nothing is asserted about the holder's own storage: every expectation is read
//! off the pushed slice, which is what the page receives.

mod chat_seam;
use chat_seam::stored;

use std::sync::{Arc, Mutex, MutexGuard};

use gentle_app::{ChatCommand, ChatId, Input, Intent};
use gentle_core::rpc::config_store::{self, OrchestratorMode, StoreId, StoreValue};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::commands::{fold_spawn_failure, Emit};
use gentle_desk_lib::contract::{
    ChatProfileAvailabilityView, ChatProfileView, OrchestratorModeView, UiIntent, ViewState,
};
use gentle_runtime::chat_runtime::{ChatConfigRead, StoreRead};

/// The channel's own documented frames, quoted from `desktop-rpc.md` in that
/// fixture: the profiles document the tab draws.
const FRAMES: &str =
    include_str!("../../../../crates/gentle-core/tests/fixtures/desktop-rpc-profiles.json");

/// One profiles store read: the documented document, and the other two stores as
/// a `get_config` that was not asked (`Skipped`).
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

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// The holder with the stored chat opened and confirmed.
fn open_stored() -> (Arc<AppStateHolder>, ChatId) {
    let holder = Arc::new(AppStateHolder::default());
    let chat = stored();
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    (holder, chat)
}

/// A holder whose chat already has its view, so the switch it draws can be told
/// to disable itself.
fn drawn() -> (Arc<AppStateHolder>, ChatId) {
    let (holder, chat) = open_stored();
    holder
        .note_chat_profile(&chat, &read())
        .expect("the read that draws the tab");
    (holder, chat)
}

/// The emit seam of a test: every slice pushed, in the order it was pushed.
fn recorder() -> (Emit, Arc<Mutex<Vec<ViewState>>>) {
    let views = Arc::new(Mutex::new(Vec::new()));
    let kept = Arc::clone(&views);
    let emit: Emit = Arc::new(move |view: &ViewState| {
        lock(&kept).push(view.clone());
        Ok(())
    });
    (emit, views)
}

/// The one slice a folder failure pushed, as the page receives it.
fn pushed_profile(pushed: &Arc<Mutex<Vec<ViewState>>>) -> ChatProfileView {
    let views = lock(pushed);
    assert_eq!(
        views.len(),
        1,
        "one command that could not start answers the web with one slice"
    );
    views[0]
        .chat_profile
        .clone()
        .expect("the slice carries that chat's own view")
}

/// The two commands whose failure the page must learn about, named for the chat.
fn read_command(chat: &ChatId) -> ChatCommand {
    ChatCommand::ReadChatProfile { chat: chat.clone() }
}

fn mode_command(chat: &ChatId) -> ChatCommand {
    ChatCommand::SetChatProfileMode {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: OrchestratorMode::FullDelegation,
    }
}

#[test]
fn a_profile_read_that_could_not_start_answers_the_web() {
    let (holder, chat) = open_stored();
    let (emit, pushed) = recorder();

    fold_spawn_failure(&holder, &emit, &read_command(&chat));

    let view = pushed_profile(&pushed);
    assert_eq!(view.chat_id, chat.as_str());
    assert_eq!(
        view.stores.profiles,
        ChatProfileAvailabilityView::Unavailable,
        "the read never ran, so no store was answered: the tab draws its unavailable state"
    );
    assert_eq!(
        view.stores.fallbacks,
        ChatProfileAvailabilityView::Unavailable
    );
    assert_eq!(view.stores.pins, ChatProfileAvailabilityView::Unavailable);
    assert!(
        view.profiles.is_empty(),
        "a read that never ran sketches no profile and no default"
    );
    assert_eq!(view.default_profile, None);
    assert_eq!(view.pins, None);
    assert!(!view.saving, "a read is not a save: nothing is in flight");
    assert!(
        !view.save_failed,
        "and a read says nothing about the switch"
    );
}

#[test]
fn a_mode_save_that_could_not_start_reenables_the_switch_and_says_so() {
    let (holder, chat) = drawn();
    let (_, commands) = holder.dispatch_commands(UiIntent::SaveChatProfileMode {
        chat_id: chat.as_str().to_owned(),
        profile: "team".to_owned(),
        mode: OrchestratorModeView::FullDelegation,
    });
    assert!(
        holder.view().chat_profile.expect("the view").saving,
        "the intent disabled the switch before the write was started"
    );
    let (emit, pushed) = recorder();

    fold_spawn_failure(&holder, &emit, &commands[0]);

    let view = pushed_profile(&pushed);
    assert!(
        !view.saving,
        "the click is answered: the switch may be used again"
    );
    assert!(
        view.save_failed,
        "and the mode could not be saved, in one short line the tab draws"
    );
    assert_eq!(
        view.profiles
            .iter()
            .map(|profile| profile.name.as_str())
            .collect::<Vec<_>>(),
        ["team"],
        "a write that never ran re-read nothing, so what the tab drew stays"
    );
}

#[test]
fn the_next_read_that_answers_replaces_the_failed_one() {
    let (holder, chat) = open_stored();
    let (emit, _pushed) = recorder();
    fold_spawn_failure(&holder, &emit, &read_command(&chat));

    let answer = holder
        .note_chat_profile(&chat, &read())
        .expect("the read Try again asks for");

    let view = answer.chat_profile.expect("the chat's own view");
    assert_eq!(view.stores.profiles, ChatProfileAvailabilityView::Read);
    assert_eq!(
        view.profiles
            .iter()
            .map(|profile| profile.name.as_str())
            .collect::<Vec<_>>(),
        ["team"],
        "Try again draws the store's own truth, exactly as before the failure"
    );
}

#[test]
fn a_command_for_a_closed_chat_folds_nothing() {
    let (holder, chat) = drawn();
    holder.fold_input(Input::Intent(Intent::CloseChat(chat.clone())));
    holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: true,
    });
    let (emit, pushed) = recorder();

    fold_spawn_failure(&holder, &emit, &read_command(&chat));
    fold_spawn_failure(&holder, &emit, &mode_command(&chat));

    assert!(
        lock(&pushed).is_empty(),
        "a forced command's failure is dropped with its chat, as the close drops the view"
    );
    assert!(holder.view().chat_profile.is_none());
}

#[test]
fn a_command_with_nothing_to_answer_folds_nothing() {
    let (holder, chat) = open_stored();
    let (emit, pushed) = recorder();

    fold_spawn_failure(
        &holder,
        &emit,
        &ChatCommand::OpenProcess { chat: chat.clone() },
    );

    assert!(
        lock(&pushed).is_empty(),
        "only the two profile commands answer the page; the rest keep their log line and change nothing"
    );
    assert!(
        holder.view().chat_profile.is_none(),
        "and no view is invented for a chat nobody asked a profile of"
    );
}
