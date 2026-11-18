//! Applying a saved profile from the Profile tab (profiles-editor T3b, S4, S8,
//! S9, L30): the terminal's other two outcomes, inside the chat.
//!
//! What these tests pin, one rule each:
//!
//! - the intent names ONE chat and one profile, and the command it answers with
//!   goes to THAT chat's own child (`session` = use it in this chat, `default` =
//!   make it the global default);
//! - both apply buttons are disabled while an apply is in flight, and a second
//!   ask is DROPPED by the chat's own one-in-flight cap (L18, L29);
//! - the channel's dialog is carried to the page VERBATIM - its own title, its
//!   own message, its own token - because it is the child's text, like the
//!   terminal's (L30 ruling 2);
//! - Accept resends the SAME profile and mode with the token of what was shown,
//!   and nothing else is ever sent as a confirmation;
//! - every answer that wrote nothing leaves one short notice and keeps the view
//!   the tab drew, and an answer that ran installs the re-read truth.
//!
//! No child and no window run here: the executor is the recording fake of
//! `chat_seam`, and the reads arrive as the channel's documented frames.

mod chat_seam;
use chat_seam::{stored, Answer, Fake};

use gentle_app::{ChatCommand, ChatId, Input, Intent};
use gentle_core::rpc::config_store::{self, StoreId, StoreValue};
use gentle_core::rpc::profile_apply::{
    ApplyMode, ApplyProfileOutcome, ConfirmationToken, NoteLevel, ProfileAnswerError, ProfileApply,
    ProfileConfirmation, ProfileNote, ProfileParamsError,
};
use gentle_core::rpc::{DesktopError, NoAnswer};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{
    ApplyModeView, ChatProfileApplyConfirmationView, ChatProfileApplyNoticeView, UiIntent,
};
use gentle_desk_lib::effects;
use gentle_runtime::chat_runtime::{ChatConfigRead, ChatProfileApply, StoreRead};

/// The channel's own documented frames, quoted from `desktop-rpc.md` in that
/// fixture: the profiles document this tab draws and the screen's own dialog.
const FRAMES: &str =
    include_str!("../../../../crates/gentle-core/tests/fixtures/desktop-rpc-profiles.json");

/// The documented frames as one JSON document.
fn frames() -> serde_json::Value {
    serde_json::from_str(FRAMES).expect("the frames are JSON")
}

/// One profiles store read: the documented document, and the other two stores
/// as a `get_config` that was not asked (`Skipped`) - enough to be a view.
fn read() -> ChatConfigRead {
    let parsed = config_store::parse_store_value(StoreId::Profiles, &frames()["profiles_data"])
        .expect("a documented profiles document");
    let StoreValue::Profiles(document) = parsed else {
        panic!("the profiles store answered a profiles document");
    };
    ChatConfigRead {
        profiles: StoreRead::Read(document),
        ..ChatConfigRead::unasked()
    }
}

/// The documented dialog, as the child answered it: its own title, message and
/// token (L30 ruling 2).
fn asking() -> ChatProfileApply {
    let confirmation = frames()["confirmation_required"]["data"]["confirmation"].clone();
    ChatProfileApply {
        result: ApplyProfileOutcome::ConfirmationRequired(ProfileConfirmation {
            profile: "team".to_owned(),
            mode: ApplyMode::Default,
            title: confirmation["title"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            message: confirmation["message"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            token: ConfirmationToken::parse("3f1c…").expect("the documented token"),
        }),
        read: None,
    }
}

/// An apply that ran, with the configuration it re-read.
fn ran() -> ChatProfileApply {
    ChatProfileApply {
        result: ApplyProfileOutcome::Applied(ProfileApply {
            profile: "team".to_owned(),
            mode: ApplyMode::Session,
            notes: Vec::new(),
        }),
        read: Some(read()),
    }
}

/// The child's own notes, verbatim as the approved T3b probe (L32) had them: it
/// writes them in its own language, in two or three lines, and names itself.
const APPLIED_NOTES: [&str; 2] = [
    "el Gentleman applied profile \"team\" — 0 agents updated.\nNew routing takes effect on the next subagent launch.\nOrchestrator set to anthropic/claude-sonnet-4 in settings.json.",
    "anthropic/claude-sonnet-4 is not in the model catalog; this session keeps its current model.",
];

/// An apply that RAN with the child's own notes, and the configuration it re-read.
fn ran_with(notes: &[&str]) -> ChatProfileApply {
    ChatProfileApply {
        result: ApplyProfileOutcome::Applied(ProfileApply {
            profile: "team".to_owned(),
            mode: ApplyMode::Default,
            notes: notes
                .iter()
                .map(|text| ProfileNote {
                    level: NoteLevel::Info,
                    text: (*text).to_owned(),
                })
                .collect(),
        }),
        read: Some(read()),
    }
}

/// The child's own answer notes the slice carries, exactly as the page receives
/// them, `None` when it carries none.
fn answer_notes(pushed: &gentle_desk_lib::contract::ViewState) -> Option<serde_json::Value> {
    let view = serde_json::to_value(pushed.chat_profile.clone()?).expect("a view is JSON");
    Some(view.get("apply_answer")?.get("notes")?.clone())
}

/// The answer that wrote nothing for one typed reason.
fn refused(result: ApplyProfileOutcome) -> ChatProfileApply {
    ChatProfileApply { result, read: None }
}

/// The owner's own refusal, worded the way the channel words it.
fn error(code: &str) -> DesktopError {
    DesktopError {
        code: code.to_owned(),
        message: "fixed".to_owned(),
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

/// A holder whose chat already has its view, so the buttons it draws can be told
/// to disable themselves.
fn drawn() -> (AppStateHolder, ChatId) {
    let (holder, chat) = open_stored();
    holder
        .note_chat_profile(&chat, &read())
        .expect("the read that draws the tab");
    (holder, chat)
}

/// The tab's ask to apply one profile of one chat.
fn apply(chat: &ChatId, mode: ApplyModeView, confirmation: Option<&str>) -> UiIntent {
    UiIntent::ApplyChatProfile {
        chat_id: chat.as_str().to_owned(),
        profile: "team".to_owned(),
        mode,
        confirmation: confirmation.map(str::to_owned),
    }
}

/// The command the tab's ask answers with, for the same profile.
fn command(chat: &ChatId, mode: ApplyMode, confirmation: Option<ConfirmationToken>) -> ChatCommand {
    ChatCommand::ApplyChatProfile {
        chat: chat.clone(),
        profile: "team".to_owned(),
        mode,
        confirmation,
    }
}

#[test]
fn use_in_this_chat_asks_that_chats_own_child_for_the_session_mode() {
    let (holder, chat) = open_stored();

    let (_view, commands) = holder.dispatch_commands(apply(&chat, ApplyModeView::Session, None));

    assert_eq!(
        commands,
        vec![command(&chat, ApplyMode::Session, None)],
        "one apply, for the chat the intent named, in the mode the button means"
    );
}

#[test]
fn make_default_asks_for_the_channel_own_global_apply_and_carries_no_token() {
    let (holder, chat) = open_stored();

    let (_view, commands) = holder.dispatch_commands(apply(&chat, ApplyModeView::Default, None));

    assert_eq!(commands, vec![command(&chat, ApplyMode::Default, None)]);
    assert!(
        matches!(
            &commands[0],
            ChatCommand::ApplyChatProfile {
                confirmation: None,
                ..
            }
        ),
        "a first apply carries no confirmation: the answer is what asks"
    );
}

#[test]
fn the_slice_the_ask_answers_with_disables_both_buttons() {
    let (holder, chat) = drawn();

    let (view, _commands) = holder.dispatch_commands(apply(&chat, ApplyModeView::Default, None));

    let view = view.chat_profile.expect("the chat's own view");
    assert!(
        view.applying,
        "an apply is in flight: both buttons are drawn disabled until it answers"
    );
    assert_eq!(view.apply_notice, None, "nothing failed yet");
    assert_eq!(
        view.apply_confirmation, None,
        "and nothing is being asked yet"
    );
}

#[test]
fn a_second_apply_while_one_is_in_flight_is_dropped() {
    let (holder, chat) = drawn();
    let (_first, commands) = holder.dispatch_commands(apply(&chat, ApplyModeView::Session, None));
    assert_eq!(commands.len(), 1);

    let (_second, dropped) = holder.dispatch_commands(apply(&chat, ApplyModeView::Default, None));

    assert!(
        dropped.is_empty(),
        "an apply is chat work too: one in flight per chat (L18, L29)"
    );
    assert!(
        holder.view().chat_profile.expect("the view").applying,
        "and the one that is in flight is what the page still draws"
    );
}

#[test]
fn the_channels_dialog_reaches_the_page_exactly_as_it_arrived() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_apply(&chat, &asking())
        .expect("the answer is pushed");

    let view = pushed.chat_profile.expect("the chat's own view");
    let confirmation = view
        .apply_confirmation
        .expect("the screen's own dialog is what the page shows");
    let documented = frames()["confirmation_required"]["data"]["confirmation"].clone();
    assert_eq!(
        confirmation,
        ChatProfileApplyConfirmationView {
            profile: "team".to_owned(),
            mode: ApplyModeView::Default,
            title: documented["title"].as_str().unwrap_or_default().to_owned(),
            message: documented["message"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            token: "3f1c…".to_owned(),
        },
        "L30 ruling 2: the child's title, message and token, verbatim"
    );
    assert!(!view.applying, "the answer enabled the buttons again");
    assert_eq!(view.apply_notice, None, "being asked is not a failure");
}

#[test]
fn accepting_resends_the_same_profile_and_mode_with_the_token() {
    let (holder, chat) = drawn();
    let shown = holder
        .note_chat_apply(&chat, &asking())
        .and_then(|view| view.chat_profile)
        .and_then(|view| view.apply_confirmation)
        .expect("the dialog the page showed");

    let (_view, commands) =
        holder.dispatch_commands(apply(&chat, shown.mode, Some(shown.token.as_str())));

    assert_eq!(
        commands,
        vec![command(
            &chat,
            ApplyMode::Default,
            Some(ConfirmationToken::parse("3f1c…").expect("the shown token"))
        )],
        "S8: the SAME profile and mode, plus the token of what was shown"
    );
}

#[test]
fn a_confirmation_that_is_not_a_token_is_never_sent() {
    for not_a_token in ["", &"t".repeat(129)] {
        let (holder, chat) = open_stored();
        let (_view, commands) =
            holder.dispatch_commands(apply(&chat, ApplyModeView::Default, Some(not_a_token)));

        assert_eq!(
            commands,
            vec![command(&chat, ApplyMode::Default, None)],
            "an empty or oversized string is not a token: the apply is asked again, so a \
             fresh dialog (and a fresh token) answers instead of writing"
        );
    }
}

#[test]
fn an_apply_that_ran_installs_the_reread_truth_and_clears_the_notice() {
    let (holder, chat) = drawn();
    holder.note_chat_apply(
        &chat,
        &refused(ApplyProfileOutcome::Rejected(error("invalid_request"))),
    );

    let pushed = holder
        .note_chat_apply(&chat, &ran())
        .expect("the answer is pushed");

    let view = pushed.chat_profile.expect("the chat's own view");
    assert!(!view.applying, "the answer enabled the buttons again");
    assert_eq!(
        view.apply_notice, None,
        "an apply that ran says nothing: the re-read is what the page draws"
    );
    assert_eq!(
        view.profiles
            .iter()
            .map(|profile| profile.name.as_str())
            .collect::<Vec<_>>(),
        ["team"],
        "and the re-read is installed"
    );
}

#[test]
fn an_applied_answer_carries_the_children_notes_to_the_tab() {
    let (holder, chat) = drawn();

    let pushed = holder
        .note_chat_apply(&chat, &ran_with(&APPLIED_NOTES))
        .expect("the answer is pushed");

    assert_eq!(
        answer_notes(&pushed),
        Some(serde_json::json!(APPLIED_NOTES)),
        "an apply that RAN said what it did, in the child's own words: the tab draws them \
         verbatim, one line per note (L32, and the maintainer's real-window finding of \
         2026-11-17: the apply answered and the tab drew nothing)"
    );
    let view = pushed.chat_profile.expect("the chat's own view");
    assert!(!view.applying, "the answer enabled the buttons again");
    assert_eq!(
        view.apply_notice, None,
        "an apply that ran is not a failure: the child's own answer is what says it happened"
    );
}

#[test]
fn a_not_applied_answer_carries_the_children_notes_beside_its_notice() {
    let (holder, chat) = drawn();
    let answer = ChatProfileApply {
        result: ApplyProfileOutcome::NotApplied(ProfileApply {
            profile: "team".to_owned(),
            mode: ApplyMode::Default,
            notes: vec![ProfileNote {
                level: NoteLevel::Warning,
                text: APPLIED_NOTES[1].to_owned(),
            }],
        }),
        read: Some(read()),
    };

    let pushed = holder
        .note_chat_apply(&chat, &answer)
        .expect("the answer is pushed");

    assert_eq!(
        answer_notes(&pushed),
        Some(serde_json::json!([APPLIED_NOTES[1]])),
        "not_applied RAN too: its notes say why, and they are the child's own words"
    );
    assert_eq!(
        pushed.chat_profile.expect("the view").apply_notice,
        Some(ChatProfileApplyNoticeView::NotApplied),
        "and the short line that nothing stands stays exactly as it was"
    );
}

#[test]
fn the_children_answer_goes_away_on_the_next_successful_read() {
    let (holder, chat) = drawn();
    holder.note_chat_apply(&chat, &ran_with(&APPLIED_NOTES));
    assert!(
        answer_notes(&holder.view()).is_some(),
        "the answer is drawn"
    );

    let pushed = holder
        .note_chat_profile(&chat, &read())
        .expect("a read that reached a store is a view");

    assert_eq!(
        answer_notes(&pushed),
        None,
        "the next successful read clears the child's answer exactly as it clears the notice: \
         one lifecycle, no second timer"
    );
}

#[test]
fn a_read_that_reached_no_store_keeps_the_children_answer() {
    let (holder, chat) = drawn();
    holder.note_chat_apply(&chat, &ran_with(&APPLIED_NOTES));

    assert!(
        holder
            .note_chat_profile(&chat, &ChatConfigRead::unasked())
            .is_none(),
        "a read that reached nothing keeps what it drew (L11-4)"
    );
    assert_eq!(
        answer_notes(&holder.view()),
        Some(serde_json::json!(APPLIED_NOTES))
    );
}

#[test]
fn an_answer_that_wrote_nothing_and_the_children_dialog_carry_no_notes() {
    let (holder, chat) = drawn();
    holder.note_chat_apply(&chat, &ran_with(&APPLIED_NOTES));

    let wrote_nothing = holder
        .note_chat_apply(&chat, &refused(ApplyProfileOutcome::Unsupported))
        .expect("the answer is pushed");
    assert_eq!(
        answer_notes(&wrote_nothing),
        None,
        "nothing ran, so nothing of the child's answer stands: the short notice says why"
    );

    holder.note_chat_apply(&chat, &ran_with(&APPLIED_NOTES));
    let asks = holder
        .note_chat_apply(&chat, &asking())
        .expect("the answer is pushed");
    assert_eq!(
        answer_notes(&asks),
        None,
        "and the child's own dialog is the next thing said, never beside the last answer"
    );
}

#[test]
fn every_answer_that_wrote_nothing_leaves_its_own_short_notice() {
    let cases = [
        (
            ApplyProfileOutcome::UnknownProfile(error("unknown_profile")),
            ChatProfileApplyNoticeView::UnknownProfile,
        ),
        (
            ApplyProfileOutcome::Busy(error("profile_apply_busy")),
            ChatProfileApplyNoticeView::Busy,
        ),
        (
            ApplyProfileOutcome::Rejected(error("revision_conflict")),
            ChatProfileApplyNoticeView::RevisionConflict,
        ),
        (
            ApplyProfileOutcome::Rejected(error("invalid_data")),
            ChatProfileApplyNoticeView::InvalidData,
        ),
        (
            ApplyProfileOutcome::NoAnswer(NoAnswer::TimedOut),
            ChatProfileApplyNoticeView::NoAnswer,
        ),
        (
            ApplyProfileOutcome::ProfilesUnavailable(error("profiles_unavailable")),
            ChatProfileApplyNoticeView::NoAnswer,
        ),
        (
            ApplyProfileOutcome::Unsupported,
            ChatProfileApplyNoticeView::Unsupported,
        ),
        (
            ApplyProfileOutcome::NotApplied(gentle_core::rpc::profile_apply::ProfileApply {
                profile: "team".to_owned(),
                mode: ApplyMode::Default,
                notes: Vec::new(),
            }),
            ChatProfileApplyNoticeView::NotApplied,
        ),
        (
            ApplyProfileOutcome::Rejected(error("internal_error")),
            ChatProfileApplyNoticeView::Refused,
        ),
        (
            ApplyProfileOutcome::UnexpectedAnswer(ProfileAnswerError::ConfirmationForSession),
            ChatProfileApplyNoticeView::Refused,
        ),
        (
            ApplyProfileOutcome::InvalidParams(ProfileParamsError::EmptyProfile),
            ChatProfileApplyNoticeView::Refused,
        ),
        (
            ApplyProfileOutcome::NoAnswer(NoAnswer::NotConnected),
            ChatProfileApplyNoticeView::NoAnswer,
        ),
    ];

    for (answer, notice) in cases {
        let (holder, chat) = drawn();

        let pushed = holder
            .note_chat_apply(&chat, &refused(answer.clone()))
            .expect("the answer is pushed");

        let view = pushed.chat_profile.expect("the chat's own view");
        assert_eq!(view.apply_notice, Some(notice), "{answer:?}");
        assert!(!view.applying, "{answer:?}: the buttons are usable again");
        assert_eq!(
            view.profiles.len(),
            1,
            "{answer:?}: nothing was written, so the view it drew stays"
        );
    }
}

#[test]
fn a_notice_never_survives_the_next_apply_that_runs() {
    let (holder, chat) = drawn();

    holder.note_chat_apply(&chat, &refused(ApplyProfileOutcome::Unsupported));
    let _ = holder.dispatch_commands(apply(&chat, ApplyModeView::Session, None));
    let pushed = holder
        .note_chat_apply(&chat, &ran())
        .expect("the answer is pushed");

    assert_eq!(
        pushed.chat_profile.expect("the view").apply_notice,
        None,
        "the click's own answer is what clears it"
    );
}

#[test]
fn the_notice_goes_away_on_the_next_successful_read() {
    let (holder, chat) = drawn();
    holder.note_chat_apply(&chat, &refused(ApplyProfileOutcome::Unsupported));
    assert_eq!(
        holder.view().chat_profile.expect("the view").apply_notice,
        Some(ChatProfileApplyNoticeView::Unsupported)
    );

    let pushed = holder
        .note_chat_profile(&chat, &read())
        .expect("a read that reached a store is a view");

    assert_eq!(
        pushed.chat_profile.expect("the view").apply_notice,
        None,
        "the next successful read clears the notice, with no timer and no toast"
    );
}

#[test]
fn a_read_that_reached_no_store_keeps_the_notice() {
    let (holder, chat) = drawn();
    holder.note_chat_apply(&chat, &refused(ApplyProfileOutcome::Unsupported));

    assert!(
        holder
            .note_chat_profile(&chat, &ChatConfigRead::unasked())
            .is_none(),
        "a read that reached nothing keeps what it drew (L11-4)"
    );
    assert_eq!(
        holder.view().chat_profile.expect("the view").apply_notice,
        Some(ChatProfileApplyNoticeView::Unsupported)
    );
}

#[test]
fn a_read_while_an_apply_is_in_flight_keeps_the_buttons_disabled() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(apply(&chat, ApplyModeView::Default, None));
    assert!(holder.view().chat_profile.expect("the view").applying);

    let pushed = holder
        .note_chat_profile(&chat, &read())
        .expect("a read that reached a store is a view");

    assert!(
        pushed.chat_profile.expect("the view").applying,
        "the apply lives until its own answer, as the mode save does (R3-001)"
    );
}

#[test]
fn the_fold_carries_the_apply_beside_the_inputs() {
    let (_holder, chat) = drawn();
    let fake = Fake::answering(Answer::ChatApply(Box::new(asking())));

    let ran = effects::run_one(command(&chat, ApplyMode::Default, None), &fake);

    assert!(
        ran.inputs.is_empty(),
        "the reducer knows nothing about an apply (L11-1)"
    );
    assert_eq!(ran.chat_apply.len(), 1, "one command, one answer");
    let (named, attempt) = ran.chat_apply.into_iter().next().expect("the answer");
    assert_eq!(named, chat);
    assert_eq!(attempt, asking());
    assert_eq!(
        fake.asked(),
        vec![command(&chat, ApplyMode::Default, None)],
        "the command that ran is the one the intent asked for"
    );
}

#[test]
fn closing_the_chat_drops_the_apply_state_with_its_view() {
    let (holder, chat) = drawn();
    holder.dispatch_commands(apply(&chat, ApplyModeView::Default, None));
    assert!(holder.view().chat_profile.expect("the view").applying);

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
        "and a reopened chat starts with no apply in flight"
    );
}

#[test]
fn an_apply_that_answers_after_its_chat_closed_is_dropped() {
    let (holder, chat) = drawn();
    holder.fold_input(Input::Intent(Intent::CloseChat(chat.clone())));
    holder.fold_input(Input::ChatClosed {
        chat: chat.clone(),
        expected: true,
    });

    assert!(
        holder.note_chat_apply(&chat, &asking()).is_none(),
        "an answer that arrives after its chat closed changes nothing and pushes nothing"
    );
}
