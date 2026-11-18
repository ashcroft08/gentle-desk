//! At most ONE profile read - and one mode save - in flight per chat
//! (profiles-editor L18: the measured storm was 4,570 asks in 25 s, each a
//! `chat-command` thread serialised on one child's client, ~4,300 OS threads
//! alive).
//!
//! The cap is the APP's, so a web bug can never again queue thousands of worker
//! threads against one child, and a duplicate ask is DROPPED rather than queued:
//! the switch's disabled state and the tab's single ask already make the second
//! one unnecessary, and dropping keeps the child free. Both entry paths keep the
//! same rule - the intent path (`dispatch_commands`, which answers the command
//! the caller then runs) and the child-event path (`deliver_chat_commands`,
//! which hands the fold's commands to the sink).
//!
//! No child and no window run here: the commands are asserted as the values each
//! dispatch answers, and the slices as the pushed `ViewState`.

mod chat_seam;
use chat_seam::stored;

use std::sync::{Arc, Mutex, MutexGuard};

use gentle_app::{ChatCommand, ChatId, Input, Intent};
use gentle_core::rpc::config_store::{self, StoreId, StoreValue};
use gentle_core::rpc::RpcCommand;
use gentle_desk_lib::bridge::{AppStateHolder, ChatCommandSink, ChatWork};
use gentle_desk_lib::contract::{OrchestratorModeView, UiIntent};
use gentle_runtime::chat_runtime::{ChatConfigRead, ModeWriteResult, StoreRead};

/// The channel's own documented frames, quoted from `desktop-rpc.md` in that
/// fixture: the profiles document the tab draws.
const FRAMES: &str =
    include_str!("../../../../crates/gentle-core/tests/fixtures/desktop-rpc-profiles.json");

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

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

/// The Profile tab's ask for one chat's configuration.
fn refresh(chat: &ChatId) -> UiIntent {
    UiIntent::RefreshChatProfile {
        chat_id: chat.as_str().to_owned(),
    }
}

/// The tab's ask to save one profile's mode.
fn save(chat: &ChatId, mode: OrchestratorModeView) -> UiIntent {
    UiIntent::SaveChatProfileMode {
        chat_id: chat.as_str().to_owned(),
        profile: "team".to_owned(),
        mode,
    }
}

/// The read of one chat, as the command the intent path answers with.
fn read_command(chat: &ChatId) -> ChatCommand {
    ChatCommand::ReadChatProfile { chat: chat.clone() }
}

/// The sink a run arms with its executor, recording what it was handed instead
/// of starting a worker: no child, no pipe.
fn recording_sink() -> (ChatCommandSink, Arc<Mutex<Vec<ChatCommand>>>) {
    let delivered: Arc<Mutex<Vec<ChatCommand>>> = Arc::new(Mutex::new(Vec::new()));
    let kept = Arc::clone(&delivered);
    let sink: ChatCommandSink = Arc::new(move |commands: Vec<ChatCommand>| {
        lock(&kept).extend(commands);
    });
    (sink, delivered)
}

/// The one command a dispatch answered, as the list it must be.
fn only(commands: &[ChatCommand], expected: ChatCommand) {
    assert_eq!(
        commands,
        [expected],
        "exactly one command runs for the ask that was accepted"
    );
}

#[test]
fn a_second_read_while_one_is_in_flight_is_dropped() {
    let (holder, chat) = open_stored();

    let (first, commands) = holder.dispatch_commands(refresh(&chat));
    only(&commands, read_command(&chat));

    let (second, dropped) = holder.dispatch_commands(refresh(&chat));

    assert!(
        dropped.is_empty(),
        "a second read while one is in flight queues NOTHING: the tab's coalesced read does not need it, and the child must stay free (L18)"
    );
    assert_eq!(
        second, first,
        "the web keeps what it has: the slice of a dropped ask is the slice as it stands"
    );
}

#[test]
fn a_read_that_answered_frees_the_chat_for_the_next_one() {
    let (holder, chat) = open_stored();
    holder.dispatch_commands(refresh(&chat));

    holder
        .note_chat_profile(&chat, &read())
        .expect("the read's answer draws the tab");

    let (_view, commands) = holder.dispatch_commands(refresh(&chat));
    only(&commands, read_command(&chat));
}

#[test]
fn a_read_that_reached_no_store_also_frees_the_chat() {
    let (holder, chat) = open_stored();
    holder.dispatch_commands(refresh(&chat));

    assert!(
        holder
            .note_chat_profile(&chat, &ChatConfigRead::unasked())
            .is_none(),
        "a read that reached nothing keeps what the tab drew (L11-4)"
    );

    let (_view, commands) = holder.dispatch_commands(refresh(&chat));
    only(&commands, read_command(&chat));
}

#[test]
fn a_second_mode_save_while_one_is_saving_is_dropped() {
    let (holder, chat) = drawn();

    let (first, commands) = holder.dispatch_commands(save(&chat, OrchestratorModeView::Organic));
    assert_eq!(commands.len(), 1, "the first save runs");

    let (second, dropped) = holder.dispatch_commands(save(&chat, OrchestratorModeView::Organic));

    assert!(
        dropped.is_empty(),
        "a second save while one is in flight writes nothing (review R1-002/R3-002)"
    );
    assert_eq!(
        second, first,
        "and it answers the slice as it stands, its switch still disabled"
    );
    assert!(
        second.chat_profile.expect("the chat's view").saving,
        "the one save in flight is what keeps the switch disabled"
    );
}

#[test]
fn a_mode_save_that_answered_frees_the_chat_for_the_next_one() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(save(&chat, OrchestratorModeView::FullDelegation));

    holder
        .note_chat_mode(&chat, &ModeWriteResult::Saved, &read())
        .expect("the save's answer is pushed");

    let (_view, commands) = holder.dispatch_commands(save(&chat, OrchestratorModeView::Organic));
    assert_eq!(commands.len(), 1, "the next save is accepted again");
}

#[test]
fn a_read_that_never_started_frees_the_chat_for_the_next_ask() {
    let (holder, chat) = open_stored();
    holder.dispatch_commands(refresh(&chat));
    assert!(
        holder.dispatch_commands(refresh(&chat)).1.is_empty(),
        "the read is in flight"
    );

    holder.note_chat_work_unstarted(&chat, ChatWork::Read);

    let (_view, commands) = holder.dispatch_commands(refresh(&chat));
    only(&commands, read_command(&chat));
}

#[test]
fn a_mode_save_that_never_started_frees_the_switch() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(save(&chat, OrchestratorModeView::FullDelegation));

    holder.note_chat_work_unstarted(&chat, ChatWork::Save);

    let (view, commands) = holder.dispatch_commands(save(&chat, OrchestratorModeView::Organic));
    assert_eq!(
        commands.len(),
        1,
        "a save whose worker thread could not be created left nothing in flight"
    );
    assert!(view.chat_profile.expect("the chat's view").saving);
}

#[test]
fn the_event_sink_drops_a_read_while_one_is_in_flight() {
    let (holder, chat) = open_stored();
    let (sink, delivered) = recording_sink();
    holder.arm_chat_commands(sink);
    holder.dispatch_commands(refresh(&chat));

    holder.deliver_chat_commands(vec![read_command(&chat)]);

    assert!(
        lock(&delivered).is_empty(),
        "the child-event path keeps the same cap: the read in flight is still the ONE read"
    );
}

#[test]
fn the_event_sink_drops_a_save_while_one_is_saving() {
    let (holder, chat) = drawn();
    let (sink, delivered) = recording_sink();
    holder.arm_chat_commands(sink);
    holder.dispatch_commands(save(&chat, OrchestratorModeView::FullDelegation));

    holder.deliver_chat_commands(vec![ChatCommand::SetChatProfileMode {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode: gentle_core::rpc::config_store::OrchestratorMode::Organic,
    }]);

    assert!(
        lock(&delivered).is_empty(),
        "one mode save in flight per chat, through either entry path"
    );
}

#[test]
fn the_event_sink_still_hands_over_every_other_command() {
    let (holder, chat) = open_stored();
    let (sink, delivered) = recording_sink();
    holder.arm_chat_commands(sink);
    holder.dispatch_commands(refresh(&chat));

    let sent = ChatCommand::Send {
        chat: chat.clone(),
        rpc: RpcCommand::GetSessionStats,
    };
    holder.deliver_chat_commands(vec![sent.clone()]);

    assert_eq!(
        *lock(&delivered),
        vec![sent],
        "a command that holds no profile work slot is never dropped"
    );
}

#[test]
fn another_chats_read_is_not_dropped() {
    let (holder, chat) = open_stored();
    let other = ChatId::fresh();
    holder.fold_input(Input::Intent(Intent::OpenChat(other.clone())));
    holder.fold_input(Input::ChatOpened {
        chat: other.clone(),
    });
    holder.dispatch_commands(refresh(&chat));

    let (_view, commands) = holder.dispatch_commands(refresh(&other));

    assert_eq!(
        commands,
        vec![read_command(&other)],
        "the cap is ONE read per chat, never one read for the whole app"
    );
}

/// The dropped ask answers the slice the shell already holds: nothing about the
/// read cap travels in the contract, so the page is answered with what it has
/// and no field it would have to learn (this fix changes no web code).
#[test]
fn the_view_a_dropped_read_answers_with_is_the_view_the_bridge_holds() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(refresh(&chat));

    let (view, dropped) = holder.dispatch_commands(refresh(&chat));

    assert!(dropped.is_empty());
    assert_eq!(
        view,
        holder.view(),
        "the dropped ask pushes nothing the shell does not already have"
    );
}
