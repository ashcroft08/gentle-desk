//! Editing ONE fallback chain of the used profile from the Profile tab
//! (profiles-editor T5, S2, S3, L30-3): the intent names ONE chat, ONE profile
//! and ONE chain key (an agent's name or the profile-wide `*`), the write is a
//! per-chat command over that chat's own child, the editor is disabled while the
//! write is in flight, and a write that did not happen leaves one short notice
//! that the next successful read clears.
//!
//! Two things are the contract's and are pinned here rather than in the runtime:
//! the chain the page sends is the chain the command carries (null `thinking` is
//! no thinking, not an empty effort), and a chain save holds the SAME save slot a
//! mode save does (profiles-editor L18): one save per chat in flight, whichever
//! of the two it is, and a second ask is DROPPED.
//!
//! No child and no window run here: the executor is the recording fake of
//! `chat_seam`, and the reads arrive as the channel's documented frames.

mod chat_seam;
use chat_seam::{stored, Answer, Fake};

use std::sync::{Arc, Mutex, MutexGuard};

use gentle_app::{ChatCommand, ChatId, Input, Intent};
use gentle_core::rpc::config_store::{self, ChainInvalid, FallbackStep, StoreId, StoreValue};
use gentle_core::rpc::DesktopError;
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::commands::{fold_spawn_failure, Emit};
use gentle_desk_lib::contract::{ChatProfileStepView, OrchestratorModeView, UiIntent, ViewState};
use gentle_desk_lib::effects;
use gentle_runtime::chat_runtime::{ChainWriteResult, ChatConfigRead, StoreRead};

/// The channel's own documented frames, quoted from `desktop-rpc.md` in that
/// fixture: the profiles document and the fallback chains this tab draws.
const FRAMES: &str =
    include_str!("../../../../crates/gentle-core/tests/fixtures/desktop-rpc-profiles.json");

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// One store of the documented read, typed by the channel's own parser.
fn fallbacks() -> StoreRead<config_store::FallbacksDocument> {
    let frames: serde_json::Value = serde_json::from_str(FRAMES).expect("the frames are JSON");
    let parsed =
        config_store::parse_store_value(StoreId::ProfileFallbacks, &frames["fallbacks_data"])
            .expect("a documented fallbacks document");
    let StoreValue::Fallbacks(document) = parsed else {
        panic!("the fallbacks store answered a fallbacks document");
    };
    StoreRead::Read(document)
}

/// One profiles store read, typed by the channel's own parser.
fn profiles() -> StoreRead<config_store::ProfilesDocument> {
    let frames: serde_json::Value = serde_json::from_str(FRAMES).expect("the frames are JSON");
    let parsed = config_store::parse_store_value(StoreId::Profiles, &frames["profiles_data"])
        .expect("a documented profiles document");
    let StoreValue::Profiles(document) = parsed else {
        panic!("the profiles store answered a profiles document");
    };
    StoreRead::Read(document)
}

/// The documented read: the definitions and the chains, enough to be a view with
/// something to edit. The pins store was not asked (`Skipped`).
fn read() -> ChatConfigRead {
    ChatConfigRead {
        profiles: profiles(),
        fallbacks: fallbacks(),
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

/// The chain the page sends: two steps, one of them without an effort.
fn page_steps() -> Vec<ChatProfileStepView> {
    vec![
        ChatProfileStepView {
            model: "openai/gpt-next".to_owned(),
            thinking: Some("high".to_owned()),
        },
        ChatProfileStepView {
            model: "openai/gpt-last".to_owned(),
            thinking: None,
        },
    ]
}

/// The same chain as the command carries it.
fn wire_steps() -> Vec<FallbackStep> {
    vec![
        FallbackStep {
            model: "openai/gpt-next".to_owned(),
            thinking: Some("high".to_owned()),
        },
        FallbackStep {
            model: "openai/gpt-last".to_owned(),
            thinking: None,
        },
    ]
}

/// The tab's ask to save one chain of the used profile.
fn save(chat: &ChatId, agent: &str) -> UiIntent {
    UiIntent::SaveChatProfileChain {
        chat_id: chat.as_str().to_owned(),
        profile: "team".to_owned(),
        agent: agent.to_owned(),
        steps: page_steps(),
    }
}

/// The command that ask must answer with.
fn chain_command(chat: &ChatId, agent: &str) -> ChatCommand {
    ChatCommand::SetChatProfileChain {
        chat: chat.clone(),
        profile: "team".to_owned(),
        agent: agent.to_owned(),
        steps: wire_steps(),
    }
}

/// A refusal the owner words itself, as the typed outcome names it.
fn conflict() -> ChainWriteResult {
    ChainWriteResult::failed(config_store::SetConfigOutcome::RevisionConflict(
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
fn the_chain_intent_asks_that_chats_own_child_to_write_the_chain() {
    let (holder, chat) = open_stored();

    let (_view, commands) = holder.dispatch_commands(save(&chat, "worker"));

    assert_eq!(
        commands,
        vec![chain_command(&chat, "worker")],
        "one write, for the one chat, profile and chain key the intent named"
    );
}

#[test]
fn the_profile_wide_chain_is_sent_by_its_own_key() {
    let (holder, chat) = open_stored();

    let (_view, commands) = holder.dispatch_commands(save(&chat, "*"));

    assert_eq!(
        commands,
        vec![chain_command(&chat, "*")],
        "`*` is the store's own key for the profile-wide chain: nothing rewrites it"
    );
}

#[test]
fn a_step_without_an_effort_carries_no_effort() {
    let (holder, chat) = open_stored();

    let (_view, commands) = holder.dispatch_commands(save(&chat, "worker"));

    let ChatCommand::SetChatProfileChain { steps, .. } = &commands[0] else {
        panic!("expected the chain write");
    };
    assert_eq!(
        steps[1].thinking, None,
        "the page's null is no effort, never the empty string the store would refuse"
    );
}

#[test]
fn the_slice_the_intent_answers_with_disables_the_editor() {
    let (holder, chat) = drawn();

    let (view, _commands) = holder.dispatch_commands(save(&chat, "worker"));

    let drawn = view.chat_profile.expect("the chat's own view");
    assert!(
        drawn.saving,
        "a write is in flight: the editor is drawn disabled until it answers"
    );
    assert!(
        !drawn.chain_failed,
        "nothing has failed yet: no notice before an answer"
    );
}

#[test]
fn a_failed_chain_save_says_so_and_keeps_the_reread_view() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_chain(&chat, &conflict(), &read())
        .expect("the attempt's answer is pushed");

    let view = pushed.chat_profile.expect("the chat's own view");
    assert!(
        view.chain_failed,
        "the chain could not be saved and the tab says so"
    );
    assert!(!view.saving, "the answer enabled the editor again");
    assert!(
        !view.save_failed,
        "and it says nothing about the mode switch, which was not what was written"
    );
    let team = view
        .profiles
        .iter()
        .find(|profile| profile.name == "team")
        .expect("the re-read truth is what the tab draws");
    assert_eq!(
        team.chains
            .iter()
            .map(|chain| chain.agent.as_str())
            .collect::<Vec<_>>(),
        ["*", "worker"],
        "the chains drawn are the store's own, re-read after the attempt"
    );
}

#[test]
fn a_save_that_worked_leaves_no_notice() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_chain(&chat, &ChainWriteResult::Saved, &read())
        .expect("the attempt's answer is pushed");

    let view = pushed.chat_profile.expect("the chat's own view");
    assert!(!view.chain_failed, "a save that worked says nothing");
    assert!(!view.saving);
}

#[test]
fn an_unchanged_chain_is_not_a_failure() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_chain(&chat, &ChainWriteResult::Unchanged, &read())
        .expect("the attempt's answer is pushed");

    assert!(
        !pushed.chat_profile.expect("the view").chain_failed,
        "the store already holds the chain: nothing failed"
    );
}

#[test]
fn a_chain_the_owner_refuses_is_reported_as_not_saved() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_chain(
            &chat,
            &ChainWriteResult::Invalid(ChainInvalid::TooLong { steps: 9 }),
            &read(),
        )
        .expect("the attempt's answer is pushed");

    assert!(
        pushed.chat_profile.expect("the view").chain_failed,
        "a chain the store would refuse was not saved either"
    );
}

#[test]
fn a_profile_or_chain_the_store_does_not_record_is_reported_as_not_saved() {
    for result in [
        ChainWriteResult::UnknownProfile,
        ChainWriteResult::UnknownChain,
    ] {
        let (holder, chat) = drawn();

        let pushed = holder
            .note_chat_chain(&chat, &result, &read())
            .expect("the attempt's answer is pushed");

        assert!(
            pushed.chat_profile.expect("the view").chain_failed,
            "{result:?}: a write that wrote nothing for a missing key is still not saved"
        );
    }
}

#[test]
fn the_notice_goes_away_on_the_next_successful_read() {
    let (holder, chat) = drawn();
    holder.note_chat_chain(&chat, &conflict(), &read());
    assert!(holder.view().chat_profile.expect("the view").chain_failed);

    let pushed = holder
        .note_chat_profile(&chat, &read())
        .expect("a read that reached a store is a view");

    assert!(
        !pushed.chat_profile.expect("the view").chain_failed,
        "the next successful read clears the notice, with no timer and no toast"
    );
}

#[test]
fn a_read_that_reached_no_store_does_not_clear_the_notice() {
    let (holder, chat) = drawn();
    holder.note_chat_chain(&chat, &conflict(), &read());

    assert!(holder
        .note_chat_profile(&chat, &ChatConfigRead::unasked())
        .is_none());
    assert!(
        holder.view().chat_profile.expect("the view").chain_failed,
        "and it is not the successful read the notice waits for"
    );
}

#[test]
fn the_answer_enables_the_editor_even_when_the_reread_reached_no_store() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_chain(&chat, &conflict(), &ChatConfigRead::unasked())
        .expect("the attempt's own answer is pushed");

    let view = pushed.chat_profile.expect("the previous view stays");
    assert!(!view.saving, "the editor is usable again");
    assert!(view.chain_failed, "and the notice reaches the page");
}

#[test]
fn a_read_in_flight_does_not_block_the_chain_save() {
    let (holder, chat) = open_stored();
    let (_view, reads) = holder.dispatch_commands(UiIntent::RefreshChatProfile {
        chat_id: chat.as_str().to_owned(),
    });
    assert_eq!(
        reads.len(),
        1,
        "the read is in flight, holding the READ slot"
    );

    let (_view, commands) = holder.dispatch_commands(save(&chat, "worker"));

    assert_eq!(
        commands,
        vec![chain_command(&chat, "worker")],
        "a chain save holds the SAVE slot: it never waits on a read (the same two slots a mode save already has, L18)"
    );
}

#[test]
fn a_read_is_still_allowed_while_a_chain_save_is_in_flight() {
    let (holder, chat) = drawn();
    let (_view, commands) = holder.dispatch_commands(save(&chat, "worker"));
    assert_eq!(
        commands.len(),
        1,
        "the chain save is in flight, holding the SAVE slot"
    );

    let (_view, reads) = holder.dispatch_commands(UiIntent::RefreshChatProfile {
        chat_id: chat.as_str().to_owned(),
    });

    assert_eq!(
        reads.len(),
        1,
        "the read slot is its own, exactly as it is beside a mode save (T3-mode)"
    );
}

#[test]
fn a_second_chain_save_while_one_is_in_flight_is_dropped() {
    let (holder, chat) = drawn();
    let (first, commands) = holder.dispatch_commands(save(&chat, "worker"));
    assert_eq!(commands.len(), 1, "the first chain save runs");

    let (second, dropped) = holder.dispatch_commands(save(&chat, "*"));

    assert!(
        dropped.is_empty(),
        "a second chain save while one is in flight writes nothing"
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
fn a_mode_save_while_a_chain_save_is_in_flight_is_dropped() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(save(&chat, "worker"));

    let (_view, dropped) = holder.dispatch_commands(UiIntent::SaveChatProfileMode {
        chat_id: chat.as_str().to_owned(),
        profile: "team".to_owned(),
        mode: OrchestratorModeView::FullDelegation,
    });

    assert!(
        dropped.is_empty(),
        "the mode switch and the chain editor hold the SAME save slot"
    );
}

#[test]
fn a_chain_save_that_answered_frees_the_chat_for_the_next_one() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(save(&chat, "worker"));

    holder
        .note_chat_chain(&chat, &ChainWriteResult::Saved, &read())
        .expect("the save's answer is pushed");

    let (_view, commands) = holder.dispatch_commands(save(&chat, "*"));
    assert_eq!(commands.len(), 1, "the next save is accepted again");
}

#[test]
fn the_fold_carries_the_chain_save_beside_the_inputs() {
    let (_holder, chat) = drawn();
    let fake = Fake::answering(Answer::ChatChain(
        Box::new(ChainWriteResult::Saved),
        Box::new(read()),
    ));

    let ran = effects::run_one(chain_command(&chat, "worker"), &fake);

    assert!(
        ran.inputs.is_empty(),
        "the reducer knows nothing about a profile's chains (L11-1)"
    );
    assert_eq!(ran.chat_chain.len(), 1, "one command, one answer");
    let (named, result, _read) = ran.chat_chain.into_iter().next().expect("the answer");
    assert_eq!(named, chat);
    assert_eq!(result, ChainWriteResult::Saved);
    assert_eq!(
        fake.asked(),
        vec![chain_command(&chat, "worker")],
        "the command that ran is the one the intent asked for"
    );
}

#[test]
fn closing_the_chat_drops_the_save_state_with_its_view() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(save(&chat, "worker"));
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
fn a_chain_save_that_answers_after_its_chat_closed_is_dropped() {
    let (holder, chat) = drawn();
    holder.fold_input(Input::Intent(Intent::CloseChat(chat.clone())));
    holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: true,
    });

    assert!(
        holder
            .note_chat_chain(&chat, &conflict(), &read())
            .is_none(),
        "a write that answers after its chat closed changes nothing and pushes nothing"
    );
}

#[test]
fn a_chain_save_that_could_not_start_reenables_the_editor_and_says_so() {
    let (holder, chat) = drawn();
    let (_view, commands) = holder.dispatch_commands(save(&chat, "worker"));
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
        "the click is answered: the editor may be used again"
    );
    assert!(
        view.chain_failed,
        "and the chain could not be saved, in one short line the tab draws"
    );
}
