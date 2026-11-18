//! Editing ONE pin layer of a chat's WORKTREE from the Profile tab
//! (profiles-editor T6, S2, S3): the intent names ONE chat, ONE layer (`local` or
//! `repo`) and the profile that layer should hold - or no profile at all, which
//! clears it. The write is a per-chat command over that chat's own child, the
//! editor is disabled while it is in flight, and a write that did not happen
//! leaves one short notice that the next successful read clears.
//!
//! Three things are the contract's and are pinned here rather than in the runtime:
//! the layer string the page sends must be one the channel names (`local`,
//! `repo`), a clear travels as `null` rather than as an empty name, and a pin save
//! holds the SAME save slot a mode save and a chain save do (profiles-editor L18):
//! one write per chat in flight, whichever of the three it is.
//!
//! No child and no window run here: the executor is the recording fake of
//! `chat_seam`, and the reads arrive as the channel's documented frames.

mod chat_seam;
use chat_seam::{stored, Answer, Fake};

use std::sync::{Arc, Mutex, MutexGuard};

use gentle_app::{ChatCommand, ChatId, Input, Intent};
use gentle_core::rpc::config_store::{self, PinInvalid, PinSource, StoreId, StoreValue};
use gentle_core::rpc::DesktopError;
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::commands::{fold_spawn_failure, Emit};
use gentle_desk_lib::contract::{ChatProfilePinStatusView, UiIntent, ViewState};
use gentle_desk_lib::effects;
use gentle_runtime::chat_runtime::{ChatConfigRead, PinWriteResult, PinsRead, StoreRead};

/// The channel's own documented frames, quoted from `desktop-rpc.md` in that
/// fixture.
const FRAMES: &str =
    include_str!("../../../../crates/gentle-core/tests/fixtures/desktop-rpc-profiles.json");

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// The documented profiles store, typed by the channel's own parser: the profile
/// this chat uses is one of them.
fn profiles() -> StoreRead<config_store::ProfilesDocument> {
    let frames: serde_json::Value = serde_json::from_str(FRAMES).expect("the frames are JSON");
    let parsed = config_store::parse_store_value(StoreId::Profiles, &frames["profiles_data"])
        .expect("a documented profiles document");
    let StoreValue::Profiles(document) = parsed else {
        panic!("the profiles store answered a profiles document");
    };
    StoreRead::Read(document)
}

/// The `pins` answer of this chat's worktree: the documented value, the two files
/// it reports and the two write targets it offers.
fn pins_value(repo: Option<&str>, local_status: &str) -> serde_json::Value {
    let repo_status = if repo.is_some() { "ok" } else { "missing" };
    let mut value = serde_json::json!({
        "resolved": repo.is_some(),
        "invalid": [],
        "stale": [],
    });
    if let Some(profile) = repo {
        value["winner"] = serde_json::json!({ "source": "repo", "profile": profile });
    }
    serde_json::json!({
        "store": "pins",
        "scope": "local+repo",
        "readOnly": false,
        "status": "ok",
        "revision": "sha256:pins",
        "value": value,
        "layers": [
            { "source": "local", "status": local_status },
            { "source": "repo", "status": repo_status },
        ],
        "writeTargets": [
            { "target": "local", "status": "ok", "writeRevision": "sha256:pin-local" },
            { "target": "repo", "status": "ok", "writeRevision": "sha256:pin-repo" },
        ],
    })
}

/// One `pins` answer, typed the way the runtime types it.
fn pins_of(answer: &serde_json::Value) -> StoreRead<PinsRead> {
    let projection =
        config_store::parse_config_projection(answer).expect("a documented pins projection");
    let Some(StoreValue::Pins(value)) = projection.value.clone() else {
        panic!("the pins store answered a pins value");
    };
    StoreRead::Read(PinsRead::of(value, &projection))
}

/// The read that draws the tab: the definitions, and the pins of this chat's
/// worktree with the repo layer holding `team` and the local one never written.
fn read() -> ChatConfigRead {
    ChatConfigRead {
        profiles: profiles(),
        pins: pins_of(&pins_value(Some("team"), "missing")),
        ..ChatConfigRead::unasked()
    }
}

/// The holder with the stored chat opened and confirmed: foreground and idle.
fn open_stored() -> (Arc<AppStateHolder>, ChatId) {
    let holder = Arc::new(AppStateHolder::default());
    let chat = stored();
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    (holder, chat)
}

/// A holder whose chat already has its view, so the editor it draws can be told
/// to disable itself.
fn drawn() -> (Arc<AppStateHolder>, ChatId) {
    let (holder, chat) = open_stored();
    holder
        .note_chat_profile(&chat, &read())
        .expect("the read that draws the tab");
    (holder, chat)
}

/// The tab's ask to write one pin layer.
fn save(chat: &ChatId, source: &str, profile: Option<&str>) -> UiIntent {
    UiIntent::SaveChatProfilePin {
        chat_id: chat.as_str().to_owned(),
        source: source.to_owned(),
        profile: profile.map(str::to_owned),
    }
}

/// The command that ask must answer with.
fn pin_command(chat: &ChatId, source: PinSource, profile: Option<&str>) -> ChatCommand {
    ChatCommand::SetChatProfilePin {
        chat: chat.clone(),
        source,
        profile: profile.map(str::to_owned),
    }
}

/// A refusal the owner words itself, as the typed outcome names it.
fn conflict() -> PinWriteResult {
    PinWriteResult::failed(config_store::SetConfigOutcome::RevisionConflict(
        DesktopError {
            code: "revision_conflict".to_owned(),
            message: "Config changed since expectedRevision was read".to_owned(),
        },
    ))
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

#[test]
fn the_pin_intent_asks_that_chats_own_child_to_write_the_layer() {
    let (holder, chat) = open_stored();

    let (_view, commands) = holder.dispatch_commands(save(&chat, "repo", Some("team")));

    assert_eq!(
        commands,
        vec![pin_command(&chat, PinSource::Repo, Some("team"))],
        "one write, for the one chat and the one layer the intent named"
    );
}

#[test]
fn a_clear_is_carried_as_no_profile_at_all() {
    let (holder, chat) = open_stored();

    let (_view, commands) = holder.dispatch_commands(save(&chat, "local", None));

    let ChatCommand::SetChatProfilePin { profile, .. } = &commands[0] else {
        panic!("expected the pin write");
    };
    assert_eq!(
        *profile, None,
        "the picker's `no pin` is `null`, the channel's explicit clear - never the empty string"
    );
}

#[test]
fn a_layer_string_that_names_no_layer_asks_for_nothing() {
    let (holder, chat) = open_stored();

    for source in ["global", "", "Repo", "local+repo"] {
        let (view, commands) = holder.dispatch_commands(save(&chat, source, Some("team")));

        assert!(
            commands.is_empty(),
            "{source:?}: the channel names two layers, so no command is built for a third"
        );
        assert_eq!(
            view,
            holder.view(),
            "{source:?}: and the slice answers as it stands"
        );
    }
}

#[test]
fn the_slice_the_intent_answers_with_disables_the_editor() {
    let (holder, chat) = drawn();

    let (view, _commands) = holder.dispatch_commands(save(&chat, "repo", Some("fast")));

    let drawn = view.chat_profile.expect("the chat's own view");
    assert!(
        drawn.saving,
        "a write is in flight: the picker and its Save are drawn disabled until it answers"
    );
    assert!(
        !drawn.pin_failed,
        "nothing has failed yet: no notice before an answer"
    );
}

#[test]
fn a_failed_pin_save_says_so_and_keeps_the_reread_view() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_pin(&chat, &conflict(), &read())
        .expect("the attempt's answer is pushed");

    let view = pushed.chat_profile.expect("the chat's own view");
    assert!(
        view.pin_failed,
        "the pin could not be saved and the tab says so"
    );
    assert!(!view.saving, "the answer enabled the editor again");
    assert!(
        !view.save_failed,
        "and it says nothing about the mode switch, which was not what was written"
    );
    assert!(
        !view.chain_failed,
        "nor about the chains: one notice per control"
    );
    let pins = view.pins.expect("the re-read truth is what the tab draws");
    assert_eq!(
        pins.layers
            .iter()
            .map(|layer| (layer.source.as_str(), layer.status))
            .collect::<Vec<_>>(),
        [
            ("repo", ChatProfilePinStatusView::Ok),
            ("local", ChatProfilePinStatusView::Missing),
        ],
        "the layers drawn are the store's own, re-read after the attempt (the winner first, then a layer it does not name)"
    );
}

#[test]
fn a_save_that_worked_leaves_no_notice() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_pin(&chat, &PinWriteResult::Saved, &read())
        .expect("the attempt's answer is pushed");

    let view = pushed.chat_profile.expect("the chat's own view");
    assert!(!view.pin_failed, "a save that worked says nothing");
    assert!(!view.saving);
}

#[test]
fn an_unchanged_pin_is_not_a_failure() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_pin(&chat, &PinWriteResult::Unchanged, &read())
        .expect("the attempt's answer is pushed");

    assert!(
        !pushed.chat_profile.expect("the view").pin_failed,
        "the layer already holds the pin: nothing failed"
    );
}

#[test]
fn a_pin_the_owner_or_the_worktree_refuses_is_reported_as_not_saved() {
    for result in [
        PinWriteResult::Invalid(PinInvalid::BlankProfile),
        PinWriteResult::Unavailable,
    ] {
        let (holder, chat) = drawn();

        let pushed = holder
            .note_chat_pin(&chat, &result, &read())
            .expect("the attempt's answer is pushed");

        assert!(
            pushed.chat_profile.expect("the view").pin_failed,
            "{result:?}: a write that never left is still not saved"
        );
    }
}

#[test]
fn the_notice_goes_away_on_the_next_successful_read() {
    let (holder, chat) = drawn();
    holder.note_chat_pin(&chat, &conflict(), &read());
    assert!(holder.view().chat_profile.expect("the view").pin_failed);

    let pushed = holder
        .note_chat_profile(&chat, &read())
        .expect("a read that reached a store is a view");

    assert!(
        !pushed.chat_profile.expect("the view").pin_failed,
        "the next successful read clears the notice, with no timer and no toast"
    );
}

#[test]
fn a_read_that_reached_no_store_does_not_clear_the_notice() {
    let (holder, chat) = drawn();
    holder.note_chat_pin(&chat, &conflict(), &read());

    assert!(holder
        .note_chat_profile(&chat, &ChatConfigRead::unasked())
        .is_none());
    assert!(
        holder.view().chat_profile.expect("the view").pin_failed,
        "and it is not the successful read the notice waits for"
    );
}

#[test]
fn the_answer_enables_the_editor_even_when_the_reread_reached_no_store() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_pin(&chat, &conflict(), &ChatConfigRead::unasked())
        .expect("the attempt's own answer is pushed");

    let view = pushed.chat_profile.expect("the previous view stays");
    assert!(!view.saving, "the picker is usable again");
    assert!(view.pin_failed, "and the notice reaches the page");
}

#[test]
fn a_read_in_flight_does_not_block_the_pin_save() {
    let (holder, chat) = open_stored();
    let (_view, reads) = holder.dispatch_commands(UiIntent::RefreshChatProfile {
        chat_id: chat.as_str().to_owned(),
    });
    assert_eq!(
        reads.len(),
        1,
        "the read is in flight, holding the READ slot"
    );

    let (_view, commands) = holder.dispatch_commands(save(&chat, "repo", Some("team")));

    assert_eq!(
        commands,
        vec![pin_command(&chat, PinSource::Repo, Some("team"))],
        "a pin save holds the SAVE slot: it never waits on a read (the same two slots a mode save has, L18)"
    );
}

#[test]
fn a_read_is_still_allowed_while_a_pin_save_is_in_flight() {
    let (holder, chat) = drawn();
    let (_view, commands) = holder.dispatch_commands(save(&chat, "repo", Some("team")));
    assert_eq!(
        commands.len(),
        1,
        "the pin save is in flight, holding the SAVE slot"
    );

    let (_view, reads) = holder.dispatch_commands(UiIntent::RefreshChatProfile {
        chat_id: chat.as_str().to_owned(),
    });

    assert_eq!(
        reads.len(),
        1,
        "the read slot is its own, exactly as it is beside a mode or chain save"
    );
}

#[test]
fn a_second_pin_save_while_one_is_in_flight_is_dropped() {
    let (holder, chat) = drawn();
    let (first, commands) = holder.dispatch_commands(save(&chat, "repo", Some("team")));
    assert_eq!(commands.len(), 1, "the first pin save runs");

    let (second, dropped) = holder.dispatch_commands(save(&chat, "local", Some("fast")));

    assert!(
        dropped.is_empty(),
        "a second pin save while one is in flight writes nothing"
    );
    assert_eq!(
        second, first,
        "and it answers the slice as it stands, its editor still disabled"
    );
    assert!(
        second.chat_profile.expect("the chat's view").saving,
        "the one save in flight is what keeps the editor disabled"
    );
}

#[test]
fn the_three_writes_of_one_chat_share_the_same_save_slot() {
    // The mode switch, the chain editor and the pin picker are three controls of
    // ONE write slot (L18): whichever went first, the others are dropped.
    let (holder, chat) = drawn();
    holder.dispatch_commands(UiIntent::SaveChatProfileChain {
        chat_id: chat.as_str().to_owned(),
        profile: "team".to_owned(),
        agent: "*".to_owned(),
        steps: vec![gentle_desk_lib::contract::ChatProfileStepView {
            model: "openai/gpt-next".to_owned(),
            thinking: None,
        }],
    });

    for (what, intent) in [
        ("pin", save(&chat, "repo", Some("team"))),
        (
            "mode",
            UiIntent::SaveChatProfileMode {
                chat_id: chat.as_str().to_owned(),
                profile: "team".to_owned(),
                mode: gentle_desk_lib::contract::OrchestratorModeView::FullDelegation,
            },
        ),
    ] {
        let (_view, dropped) = holder.dispatch_commands(intent);
        assert!(
            dropped.is_empty(),
            "a {what} save while a chain save is in flight is dropped"
        );
    }
}

#[test]
fn a_pin_save_that_answered_frees_the_chat_for_the_next_one() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(save(&chat, "repo", Some("team")));

    holder
        .note_chat_pin(&chat, &PinWriteResult::Saved, &read())
        .expect("the save's answer is pushed");

    let (_view, commands) = holder.dispatch_commands(save(&chat, "local", Some("fast")));
    assert_eq!(commands.len(), 1, "the next save is accepted again");
}

#[test]
fn the_fold_carries_the_pin_save_beside_the_inputs() {
    let (_holder, chat) = drawn();
    let fake = Fake::answering(Answer::ChatPin(
        Box::new(PinWriteResult::Saved),
        Box::new(read()),
    ));

    let ran = effects::run_one(pin_command(&chat, PinSource::Repo, Some("team")), &fake);

    assert!(
        ran.inputs.is_empty(),
        "the reducer knows nothing about a worktree's pin (L11-1)"
    );
    assert_eq!(ran.chat_pin.len(), 1, "one command, one answer");
    let (named, result, _read) = ran.chat_pin.into_iter().next().expect("the answer");
    assert_eq!(named, chat);
    assert_eq!(result, PinWriteResult::Saved);
    assert_eq!(
        fake.asked(),
        vec![pin_command(&chat, PinSource::Repo, Some("team"))],
        "the command that ran is the one the intent asked for"
    );
}

#[test]
fn closing_the_chat_drops_the_save_state_with_its_view() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(save(&chat, "repo", Some("team")));
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
    assert!(
        holder.view().chat_profile.is_none(),
        "and a reopened chat starts with no save in flight: the write died with it"
    );
}

#[test]
fn a_pin_save_that_answers_after_its_chat_closed_is_dropped() {
    let (holder, chat) = drawn();
    holder.fold_input(Input::Intent(Intent::CloseChat(chat.clone())));
    holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: true,
    });

    assert!(
        holder.note_chat_pin(&chat, &conflict(), &read()).is_none(),
        "a write that answers after its chat closed changes nothing and pushes nothing"
    );
}

#[test]
fn a_pin_save_that_could_not_start_reenables_the_editor_and_says_so() {
    let (holder, chat) = drawn();
    let (_view, commands) = holder.dispatch_commands(save(&chat, "repo", Some("team")));
    assert!(holder.view().chat_profile.expect("the view").saving);
    let (emit, pushed) = recorder();

    fold_spawn_failure(&holder, &emit, &commands[0]);

    let views = lock(&pushed);
    assert_eq!(
        views.len(),
        1,
        "one command that could not start answers the web with one slice"
    );
    let view = views[0]
        .chat_profile
        .clone()
        .expect("the slice carries that chat's own view");
    assert!(
        !view.saving,
        "the click is answered: the picker may be used again"
    );
    assert!(
        view.pin_failed,
        "and the pin could not be saved, in one short line the tab draws"
    );
}
