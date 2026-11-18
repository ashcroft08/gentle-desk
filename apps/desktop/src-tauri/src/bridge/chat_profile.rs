//! The Profile tab's view of ONE chat (profiles-editor T2b-2, L8, L9, L11).
//!
//! Two halves, and neither is the reducer's:
//!
//! - what the chat's OWN child answered, through the three reads a helper stop
//!   already makes over that chat's client ([`ChatConfigRead`]): the profiles,
//!   their fallback chains, and the pin of this chat's own worktree;
//! - which profile this chat BOUND, which no channel read exposes (L8). It comes
//!   from the chat's own session file, read through the [`FileRead`] seam, and the
//!   view says so ([`ChatProfileView::bound_from_session_file`]) because it is the
//!   approximation the maintainer accepted in L9: gentle-pi's branch replay and
//!   corroboration are not reimplemented here.
//!
//! The view is SHELL-owned, next to the auth and usage slices: `crates/app`
//! depends only on gentle-core while `gentle-runtime` depends on `gentle-app`, so
//! the reducer could not name the session-file reader even if typed profile data
//! belonged in `Input` (L11-1). It is the ONLY profile view the shell holds now
//! that Settings > Profiles is gone (profiles-editor T2d).
//!
//! A read that reached NO store changes nothing: the previous view stays and no
//! slice is pushed (L11-4). That is helper-stop's rule (`NotStopped` folds
//! nothing) and the one `get_session_stats` already follows, so a chat whose
//! child stopped answering leaves the tab exactly as it was drawn.
//!
//! Two guards keep the session-file half honest (T2b-3): the file is only read
//! when the chat's own id names a path inside this run's sessions store, by the
//! path's SPELLING (the seam is fake-able, so the filesystem is never asked);
//! and a read whose chat closed while it was in flight installs nothing, so a
//! late answer cannot bring back the view the close dropped.

use std::ffi::OsStr;
use std::path::{Component, Path, PathBuf};

use gentle_app::{ChatCommand, ChatId};
use gentle_core::i18n::Language;
use gentle_core::rpc::config_store::{
    FallbacksDocument, OrchestratorMode, PinLayer, PinsValue, ProfilesDocument, SetConfigOutcome,
};
use gentle_core::rpc::profile_apply::ApplyProfileOutcome;
use gentle_core::rpc::NoAnswer;
use gentle_runtime::chat_runtime::{
    ChainWriteResult, ChatConfigRead, ChatProfileApply, ModeWriteResult, StoreRead,
};
use gentle_runtime::session_profile::last_session_profile;

use super::{lock, AppStateHolder};
use crate::contract::{
    ApplyModeView, ChatProfileAgentView, ChatProfileApplyAnswerView,
    ChatProfileApplyConfirmationView, ChatProfileApplyNoticeView, ChatProfileAvailabilityView,
    ChatProfileChainView, ChatProfileDefinitionView, ChatProfilePinLayerView,
    ChatProfilePinMarkView, ChatProfilePinsView, ChatProfileStepView, ChatProfileStoresView,
    ChatProfileView, OrchestratorModeView, ViewState,
};
use crate::settings::FileRead;

/// [`ChatProfileView`] for `chat`, from what its child answered and from the text
/// of its own session file.
///
/// `session` is the session file's text: `None` for a chat that has no file (a
/// fresh `new-N` chat) and for one whose file could not be read, which both mean
/// the chat uses the default - never an error, and never a guess.
pub fn compose(chat: &ChatId, read: &ChatConfigRead, session: Option<&str>) -> ChatProfileView {
    // The last entry of the profile family is what the chat bound (L9): the
    // reader already reads an absent, cleared or unusable one as no binding.
    let bound = session.and_then(last_session_profile);
    let profiles = content(&read.profiles);
    let fallbacks = content(&read.fallbacks);
    let pins = content(&read.pins);
    ChatProfileView {
        chat_id: chat.as_str().to_owned(),
        bound: bound.as_ref().map(|bind| bind.name.clone()),
        bound_from_session_file: bound.is_some(),
        bound_origin: bound.as_ref().map(|bind| bind.origin.as_str().to_owned()),
        default_profile: profiles.and_then(|document| document.active.clone()),
        profiles: definitions(profiles, fallbacks),
        pins: pins.map(pins_view),
        stores: ChatProfileStoresView {
            profiles: availability(&read.profiles),
            fallbacks: availability(&read.fallbacks),
            pins: availability(&read.pins),
        },
        // The save, apply and chain states are the SHELL's, not the read's:
        // `chat_profile` overlays them from `profile_work` every time the slice
        // is built (T3-mode, T3b, T5).
        saving: false,
        save_failed: false,
        applying: false,
        apply_notice: None,
        apply_answer: None,
        apply_confirmation: None,
        chain_failed: false,
    }
}

/// One chat's profile work state (profiles-editor T3-mode, T5, L18): whether a
/// READ of that chat's configuration is in flight, whether a SAVE is (the mode
/// switch and the chain editor are drawn disabled while it is), and whether the
/// last save failed and wrote nothing - one notice for the mode and one for the
/// chains, because they are different controls that say different things.
///
/// At most one of each is in flight per chat, and a second ask while one is in
/// flight is DROPPED rather than queued ([`AppStateHolder::begin_chat_command`]):
/// a web bug that asked in a loop must never again queue thousands of
/// `chat-command` threads against one child (L18: 4,570 asks in 25 s, ~4,300 OS
/// threads alive, 22 s per read). Shell-owned, and dropped with the chat.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct ProfileWork {
    pub(super) reading: bool,
    pub(super) saving: bool,
    pub(super) failed: bool,
    /// An APPLY of one of this chat's profiles is in flight (profiles-editor
    /// T3b): both buttons are drawn disabled while it is, and a second ask is
    /// dropped rather than queued - the same rule the read and the save keep.
    pub(super) applying: bool,
    /// The one short line the last apply that wrote nothing left, `None` when it
    /// worked and when nothing failed.
    pub(super) notice: Option<ChatProfileApplyNoticeView>,
    /// The child's own answer to the last apply that RAN (`applied` or
    /// `not_applied`), verbatim: a profile whose routing already matched answers
    /// `applied` with no dialog at all, so this is the only thing that tells the
    /// user the click happened (T3b, L32).
    pub(super) answer: Option<ChatProfileApplyAnswerView>,
    /// The child's own dialog while it is waiting for the human, verbatim: the
    /// shell keeps it so every later slice carries the same text and the same
    /// token (L30 ruling 2).
    pub(super) confirmation: Option<ChatProfileApplyConfirmationView>,
    /// The chain editor's own notice: the parallel slot to `failed`, so a chain
    /// write that did not happen never lights the mode switch's line (T5).
    pub(super) chain_failed: bool,
}

/// Which profile work a per-chat command does, and so which of its chat's two
/// slots it takes (profiles-editor L18).
///
/// Only the Profile tab's commands hold one: every other per-chat command runs as
/// often as it is asked for. The mode switch and the chain editor SHARE the save
/// slot (profiles-editor T5): one write per chat in flight, whichever control
/// started it, so a mode save can never race a chain save of the same profile.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChatWork {
    Read,
    Save,
    Apply,
}

impl ChatWork {
    /// The slot `command` takes, `None` for a command that takes none.
    pub(crate) fn of(command: &ChatCommand) -> Option<Self> {
        match command {
            ChatCommand::ReadChatProfile { .. } => Some(Self::Read),
            ChatCommand::SetChatProfileMode { .. } | ChatCommand::SetChatProfileChain { .. } => {
                Some(Self::Save)
            }
            ChatCommand::ApplyChatProfile { .. } => Some(Self::Apply),
            _ => None,
        }
    }
}

/// The session-file seam of this run: the reader the Settings panes share and
/// the sessions store a chat's own path must stay inside (T2b-3).
#[derive(Clone)]
pub(super) struct SessionFiles {
    pub(super) files: FileRead,
    pub(super) store: PathBuf,
}

/// [`compose`], with the chat's own session file read through `files` - the seam
/// the Settings panes read through - from the sessions store this run owns:
/// `session` is that reader and that store together, `None` for a run that
/// resolved no store (nothing can be proven a session file, so nothing is read).
///
/// A chat with no session file yet asks nothing: no file could have bound a
/// profile. A file that cannot be read is no session file, so the chat uses the
/// default.
pub fn view(
    chat: &ChatId,
    read: &ChatConfigRead,
    session: Option<(&FileRead, &Path)>,
    language: Language,
) -> ChatProfileView {
    let session = session_text(chat, session, language);
    compose(chat, read, session.as_deref())
}

/// The text of `chat`'s own session file, or `None` for a chat that has none,
/// one whose id names no path or a path outside this run's sessions store, and
/// one this run cannot read.
fn session_text(
    chat: &ChatId,
    session: Option<(&FileRead, &Path)>,
    language: Language,
) -> Option<String> {
    let path = chat.session_path()?;
    let (files, store) = session?;
    let path = Path::new(path);
    // The id may be any string the web sent (`chat_for`'s own fallback turns
    // one into a session path), so the read is confined to the store before the
    // seam is reached (review R1-chat-for-path-fallback).
    if !session_file(store, path) {
        return None;
    }
    match files(path, language) {
        Ok(text) => text,
        // A file that is there and could not be read is the default, never an
        // error (L9); the line is for an operator, not for the pane (review
        // R4-unreadable-session-silent).
        Err(error) => {
            eprintln!(
                "chat profile: session file could not be read: {}: {error}",
                path.display()
            );
            None
        }
    }
}

/// Whether `path` may be a session file of `store`, by SPELLING alone: inside
/// the store, ending in `.jsonl` (the extension the session list itself matches
/// exactly), and with no `..` component anywhere in it.
///
/// The filesystem is never asked: the reader is a seam a test fakes, and a
/// lexical check is the rule the session index already applies to what it
/// enumerates (`gentle_backend::session_index::paths`).
fn session_file(store: &Path, path: &Path) -> bool {
    // Before anything else: `<store>/../elsewhere.jsonl` still STARTS with the
    // store, so the climb has to be refused on its own.
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return false;
    }
    if path.extension() != Some(OsStr::new("jsonl")) {
        return false;
    }
    path.starts_with(store)
}

/// One store's content, or `None` when it was not read.
fn content<T>(store: &StoreRead<T>) -> Option<&T> {
    match store {
        StoreRead::Read(value) => Some(value),
        StoreRead::Unread(_) | StoreRead::Skipped => None,
    }
}

/// How one store answered, as the page draws it.
fn availability<T>(store: &StoreRead<T>) -> ChatProfileAvailabilityView {
    match store {
        StoreRead::Read(_) => ChatProfileAvailabilityView::Read,
        StoreRead::Unread(_) => ChatProfileAvailabilityView::Unavailable,
        StoreRead::Skipped => ChatProfileAvailabilityView::Skipped,
    }
}

/// Whether the read reached even one store: the rule that decides whether it may
/// replace what the tab drew (L11-4).
fn reached_a_store(read: &ChatConfigRead) -> bool {
    content(&read.profiles).is_some()
        || content(&read.fallbacks).is_some()
        || content(&read.pins).is_some()
}

/// Every profile the store defines, each with the chains the fallbacks store
/// records for it.
fn definitions(
    profiles: Option<&ProfilesDocument>,
    fallbacks: Option<&FallbacksDocument>,
) -> Vec<ChatProfileDefinitionView> {
    let Some(profiles) = profiles else {
        return Vec::new();
    };
    profiles
        .profiles
        .iter()
        .map(|(name, agents)| ChatProfileDefinitionView {
            name: name.clone(),
            agents: agents
                .iter()
                .map(|(agent, route)| ChatProfileAgentView {
                    agent: agent.clone(),
                    model: route.model.clone(),
                    thinking: route.thinking.clone(),
                    tier: route.tier.clone(),
                })
                .collect(),
            chains: chains(fallbacks, name),
            mode: mode(profiles.workflow.get(name)),
        })
        .collect()
}

/// One profile's orchestration mode: the `workflow` entry the profiles store holds
/// for it, or gentle-pi's own default when it names none (L22: absent = organic).
///
/// The entry is stored PER PROFILE, which is why a change in the terminal reaches
/// every chat that uses the profile (L20): the mode is the profile's, never one
/// chat's.
fn mode(recorded: Option<&OrchestratorMode>) -> OrchestratorModeView {
    match recorded {
        Some(OrchestratorMode::FullDelegation) => OrchestratorModeView::FullDelegation,
        Some(OrchestratorMode::Organic) | None => OrchestratorModeView::Organic,
    }
}

/// The chains one profile has, `*` included, in the store's own order. A chain
/// the store records with no steps is not drawn: an empty list is what it says.
fn chains(fallbacks: Option<&FallbacksDocument>, profile: &str) -> Vec<ChatProfileChainView> {
    let Some(chains) = fallbacks.and_then(|document| document.profiles.get(profile)) else {
        return Vec::new();
    };
    chains
        .iter()
        .filter(|(_, steps)| !steps.is_empty())
        .map(|(agent, steps)| ChatProfileChainView {
            agent: agent.clone(),
            steps: steps
                .iter()
                .map(|step| ChatProfileStepView {
                    model: step.model.clone(),
                    thinking: step.thinking.clone(),
                })
                .collect(),
        })
        .collect()
}

/// The pins value as the page draws it: the winner first, then the layers that
/// name a profile the store dropped, then the ones that are no pin at all.
fn pins_view(pins: &PinsValue) -> ChatProfilePinsView {
    let mut layers: Vec<ChatProfilePinLayerView> = Vec::new();
    if let Some(winner) = &pins.winner {
        layers.push(layer(winner, ChatProfilePinMarkView::Winner));
    }
    for stale in &pins.stale {
        layers.push(layer(stale, ChatProfilePinMarkView::Stale));
    }
    for invalid in &pins.invalid {
        layers.push(layer(invalid, ChatProfilePinMarkView::Invalid));
    }
    ChatProfilePinsView {
        resolved: pins.resolved,
        winner: pins
            .winner
            .as_ref()
            .map(|winner| layer(winner, ChatProfilePinMarkView::Winner)),
        layers,
        reason: pins.reason.clone(),
        worktree_unresolved: pins.worktree_unresolved(),
    }
}

/// The apply mode the contract names, for a dialog the child answered with.
fn apply_mode_view(mode: gentle_core::rpc::profile_apply::ApplyMode) -> ApplyModeView {
    match mode {
        gentle_core::rpc::profile_apply::ApplyMode::Session => ApplyModeView::Session,
        gentle_core::rpc::profile_apply::ApplyMode::Default => ApplyModeView::Default,
    }
}

/// The one short line an answer that wrote nothing leaves, `None` for an attempt
/// that ran (its answer is the re-read) and for the child's own question.
///
/// The channel's own codes are the cases this slice models; every other refusal
/// is `Refused`, so the page always has something short to say and nothing is
/// invented.
fn notice_of(result: &ApplyProfileOutcome) -> Option<ChatProfileApplyNoticeView> {
    match result {
        // The attempt ran: the re-read is what the page draws.
        ApplyProfileOutcome::Applied(_) => None,
        ApplyProfileOutcome::NotApplied(_) => Some(ChatProfileApplyNoticeView::NotApplied),
        ApplyProfileOutcome::UnknownProfile(_) => Some(ChatProfileApplyNoticeView::UnknownProfile),
        ApplyProfileOutcome::Busy(_) => Some(ChatProfileApplyNoticeView::Busy),
        ApplyProfileOutcome::ProfilesUnavailable(_) | ApplyProfileOutcome::NoAnswer(_) => {
            Some(ChatProfileApplyNoticeView::NoAnswer)
        }
        ApplyProfileOutcome::Unsupported => Some(ChatProfileApplyNoticeView::Unsupported),
        ApplyProfileOutcome::Rejected(error) => Some(match error.code.as_str() {
            "revision_conflict" => ChatProfileApplyNoticeView::RevisionConflict,
            "invalid_data" => ChatProfileApplyNoticeView::InvalidData,
            _ => ChatProfileApplyNoticeView::Refused,
        }),
        // A request this app could not build, one the channel refused, and an
        // `ok` answer that is not this contract's: all of them wrote nothing.
        ApplyProfileOutcome::InvalidParams(_)
        | ApplyProfileOutcome::InvalidRequest(_)
        | ApplyProfileOutcome::UnexpectedAnswer(_) => Some(ChatProfileApplyNoticeView::Refused),
        // The child is asking, and `note_chat_apply` carries its dialog: being
        // asked is not a failure.
        ApplyProfileOutcome::ConfirmationRequired(_) => None,
    }
}

/// The child's own answer to an attempt that RAN, `None` for one that wrote
/// nothing and for one the child is asking about.
///
/// `applied` and `not_applied` both ran, and both carry the screen's notes: its
/// own words, in its own language, several lines each - never a sentence this
/// shell composes, which is why the texts travel as they arrived (L30 ruling 2).
/// An answer with no note has nothing to say and is not one.
fn answer_of(result: &ApplyProfileOutcome) -> Option<ChatProfileApplyAnswerView> {
    let ran = match result {
        ApplyProfileOutcome::Applied(ran) | ApplyProfileOutcome::NotApplied(ran) => ran,
        _ => return None,
    };
    let notes: Vec<String> = ran.notes.iter().map(|note| note.text.clone()).collect();
    (!notes.is_empty()).then_some(ChatProfileApplyAnswerView { notes })
}

/// One pin layer, as the store named it and the page draws it.
fn layer(pin: &PinLayer, mark: ChatProfilePinMarkView) -> ChatProfilePinLayerView {
    ChatProfilePinLayerView {
        source: pin.source.as_str().to_owned(),
        profile: pin.profile.clone(),
        mark,
    }
}

impl AppStateHolder {
    /// Installs the session-file seam of this run, once, at startup: the reader
    /// and the sessions store it may read from. The same kind of step as the
    /// command sink's, and it is armed only when a store resolves, so a run
    /// without a home reads no chat's file and every chat says it uses the
    /// default.
    pub fn arm_session_files(&self, files: FileRead, store: PathBuf) {
        *lock(&self.session_files) = Some(SessionFiles { files, store });
    }

    /// Records what one chat's own child answered, and answers the slice to push.
    ///
    /// `None` - and no push at all - when the chat is no longer open: a read
    /// that answers after its chat closed may not re-install the view the close
    /// dropped (review R4-late-read-reinstalls-view, R1-closed-chat-view-restored).
    /// `None` too when the read reached no store: the tab then keeps exactly what
    /// it drew (L11-4).
    ///
    /// Either way the read's slot comes free here: this is the command's answer,
    /// so the chat may be asked again (L18).
    pub fn note_chat_profile(&self, chat: &ChatId, read: &ChatConfigRead) -> Option<ViewState> {
        // The app state is the authority on the open set, and it is read before
        // any shell lock is taken, as the shell's other reads are.
        if !self.is_open(chat) {
            // A closed chat drops its work state with its view, so an unanswered
            // read's slot goes with it rather than waiting for ever (L18).
            lock(&self.profile_work).remove(chat.as_str());
            return None;
        }
        let reached = reached_a_store(read);
        {
            let mut held = lock(&self.profile_work);
            if let Some(state) = held.get_mut(chat.as_str()) {
                // Nothing of this read is in flight any more, whatever it
                // reached: a read that answered nothing still answered (L18).
                state.reading = false;
                // A read that answered IS the truth the tab draws: it clears the
                // A read that answered IS the truth the tab draws: it clears the
                // notices of a failed save - the mode's and the chains' - and the
                // ones a failed apply left (T3-mode, T3b, T5: the next successful
                // read, no timer). A write still in flight is NOT touched, and
                // neither is the child's pending dialog: the write lives until its
                // own answer, so a mid-write read may not re-enable a control
                // (review R3-001, correction of ab09880), and a dialog is answered
                // by a human.
                if reached {
                    state.failed = false;
                    state.chain_failed = false;
                    state.notice = None;
                    // The child's own answer to an apply that RAN goes with the
                    // notice: the next successful read is the truth the tab
                    // draws, and it is the same one lifecycle (T3b).
                    state.answer = None;
                }
            }
        }
        if !reached {
            return None;
        }
        let view = self.compose_view(chat, read);
        self.install_chat_profile(view);
        Some(self.view())
    }

    /// Records that a read of `chat`'s profile could not even START, and answers
    /// the slice to push (profiles-editor L17(a), the liveness hole of L18).
    ///
    /// The read never reached that chat's child - the worker thread its command
    /// runs on could not be created - so no store will ever answer: without this
    /// the tab waits in its reading state for ever and every click of the user's
    /// is another read that never runs. The view installed is the one a chat whose
    /// child is not answering already draws (every store `Unread`, which the page
    /// turns into its unavailable state and its Try again); the next read that
    /// answers replaces it, so the retry the page then offers works. A chat that
    /// closed in the meantime is dropped with its state, as its view is
    /// (L11-4 - which is about a read that ANSWERED nothing, while this one did
    /// not answer at all).
    pub fn note_chat_profile_unread(&self, chat: &ChatId) -> Option<ViewState> {
        if !self.is_open(chat) {
            lock(&self.profile_work).remove(chat.as_str());
            return None;
        }
        // The read never ran, so the slot its command took (the one-in-flight
        // cap, L18) is already given back by run_chat_commands's branch; the
        // view here is the answer the web draws.
        let view = self.compose_view(chat, &ChatConfigRead::unasked());
        self.install_chat_profile(view);
        Some(self.view())
    }

    /// Records that an attempt to save `chat`'s profile mode could not even
    /// START: the worker thread its command runs on could not be created, so the
    /// write never ran and will never answer (profiles-editor L17(a)).
    ///
    /// It is [`Self::note_chat_mode`] with the failure a child that did not answer
    /// leaves, which is the honest answer here too: nothing was written. The
    /// switch is usable again and one short line says the mode could not be
    /// saved, and a chat that closed meanwhile is dropped with its state.
    pub fn note_chat_mode_unstarted(&self, chat: &ChatId) -> Option<ViewState> {
        self.note_chat_mode(
            chat,
            &ModeWriteResult::failed(SetConfigOutcome::NoAnswer(NoAnswer::NotConnected)),
            &ChatConfigRead::unasked(),
        )
    }

    /// Records that an attempt to save one of `chat`'s fallback chains could not
    /// even START (profiles-editor T5, the same liveness hole of L17(a)): the
    /// worker thread its command runs on could not be created, so the write never
    /// ran and will never answer.
    ///
    /// It is [`Self::note_chat_chain`] with the failure a child that did not
    /// answer leaves: nothing was written, the editor is usable again, and one
    /// short line says the chain could not be saved. A chat that closed meanwhile
    /// is dropped with its state.
    pub fn note_chat_chain_unstarted(&self, chat: &ChatId) -> Option<ViewState> {
        self.note_chat_chain(
            chat,
            &ChainWriteResult::failed(SetConfigOutcome::NoAnswer(NoAnswer::NotConnected)),
            &ChatConfigRead::unasked(),
        )
    }

    /// Records what one attempt to save a fallback chain came to, with the
    /// configuration the attempt re-read, and answers the slice to push
    /// (profiles-editor T5, S2, S3, L30-3).
    ///
    /// The same lifecycle as [`Self::note_chat_mode`]: the re-read is installed
    /// exactly as a plain read installs it (a read that reached no store leaves
    /// what the tab drew standing, L11-4), the save slot comes free either way,
    /// and the attempt's own answer - whether the chain could not be saved -
    /// travels in the chain editor's own notice, never in the mode switch's. A
    /// chat that closed while the write ran is dropped with its state.
    pub fn note_chat_chain(
        &self,
        chat: &ChatId,
        result: &ChainWriteResult,
        read: &ChatConfigRead,
    ) -> Option<ViewState> {
        if !self.is_open(chat) {
            lock(&self.profile_work).remove(chat.as_str());
            return None;
        }
        // One lock, one entry: the save's own answer, and nothing about a read
        // that may be in flight at the same time (L18). `failed` is the mode
        // switch's notice and is not touched: this write was not that one.
        {
            let mut held = lock(&self.profile_work);
            let state = held.entry(chat.as_str().to_owned()).or_default();
            state.saving = false;
            state.chain_failed = !result.saved();
        }
        if reached_a_store(read) {
            let view = self.compose_view(chat, read);
            self.install_chat_profile(view);
        }
        Some(self.view())
    }

    /// Records what one attempt to save a profile's orchestration mode came to,
    /// with the configuration the attempt re-read, and answers the slice to push
    /// (profiles-editor T3-mode, S2, S3).
    ///
    /// The re-read is installed exactly as a plain read installs it, and a read
    /// that reached no store leaves what the tab drew standing (L11-4). The
    /// attempt's own answer - that the switch may be used again, and whether the
    /// mode could not be saved - travels either way, because it is the user's
    /// click that has to be answered. A chat that closed while the write ran is
    /// dropped with its state, as its view is.
    pub fn note_chat_mode(
        &self,
        chat: &ChatId,
        result: &ModeWriteResult,
        read: &ChatConfigRead,
    ) -> Option<ViewState> {
        if !self.is_open(chat) {
            lock(&self.profile_work).remove(chat.as_str());
            return None;
        }
        // One lock, one entry: the save's own answer, and nothing about a read
        // that may be in flight at the same time (L18).
        {
            let mut held = lock(&self.profile_work);
            let state = held.entry(chat.as_str().to_owned()).or_default();
            state.saving = false;
            state.failed = !result.saved();
        }
        if reached_a_store(read) {
            let view = self.compose_view(chat, read);
            self.install_chat_profile(view);
        }
        Some(self.view())
    }

    /// Records that an attempt to APPLY one of `chat`'s profiles could not even
    /// START: the worker thread its command runs on could not be created, so the
    /// request never left and will never answer (profiles-editor T3b, L17(a)).
    ///
    /// It is [`Self::note_chat_apply`] with the failure a child that did not
    /// answer leaves - nothing was applied - so both buttons come back and one
    /// short line says why. A chat that closed meanwhile is dropped with its
    /// state.
    pub fn note_chat_apply_unstarted(&self, chat: &ChatId) -> Option<ViewState> {
        self.note_chat_apply(chat, &ChatProfileApply::unasked())
    }

    /// Records what one attempt to APPLY a profile came to, and answers the slice
    /// to push (profiles-editor T3b, S4, S8, S9, L30).
    ///
    /// Three shapes, and nothing else goes to the page:
    ///
    /// - the attempt RAN: both buttons are enabled again, any notice and any
    ///   pending dialog are dropped, the child's own notes travel as the answer
    ///   ([`ChatProfileView::apply_answer`]: `applied` on a profile whose routing
    ///   already matched asks nothing, so without them the click says nothing at
    ///   all), and the configuration the attempt re-read is installed exactly as a
    ///   plain read installs it (a read that reached no store leaves what the tab
    ///   drew, L11-4);
    /// - the child is ASKING: its own `title` and `message` travel verbatim, with
    ///   the token the acceptance carries back, and no notice - being asked is not
    ///   a failure, and nothing was written (L30 ruling 2);
    /// - the attempt wrote NOTHING: one short notice names the case, no answer
    ///   travels, and the view the tab drew stays as it is.
    ///
    /// A chat that closed while the apply ran is dropped with its state, as its
    /// view is.
    pub fn note_chat_apply(&self, chat: &ChatId, attempt: &ChatProfileApply) -> Option<ViewState> {
        if !self.is_open(chat) {
            lock(&self.profile_work).remove(chat.as_str());
            return None;
        }
        // One lock, one entry: the apply's own answer, and nothing about a read
        // or a save that may be in flight at the same time (L18).
        {
            let mut held = lock(&self.profile_work);
            let state = held.entry(chat.as_str().to_owned()).or_default();
            state.applying = false;
            match &attempt.result {
                ApplyProfileOutcome::ConfirmationRequired(confirmation) => {
                    state.confirmation = Some(ChatProfileApplyConfirmationView {
                        profile: confirmation.profile.clone(),
                        mode: apply_mode_view(confirmation.mode),
                        title: confirmation.title.clone(),
                        message: confirmation.message.clone(),
                        token: confirmation.token.as_str().to_owned(),
                    });
                    state.notice = None;
                    state.answer = None;
                }
                // Every other answer ends the flow that produced it: nothing is
                // pending any more, and what the page says is said below.
                result => {
                    state.confirmation = None;
                    state.notice = notice_of(result);
                    // An attempt that RAN said what it did, in the child's own
                    // words; one that wrote nothing has its short notice instead.
                    state.answer = answer_of(result);
                }
            }
        }
        // An attempt that RAN re-read the configuration, and that read is the
        // truth the tab draws; one that wrote nothing re-read nothing, so what it
        // drew stays.
        if let Some(read) = attempt.read.as_ref().filter(|read| reached_a_store(read)) {
            let view = self.compose_view(chat, read);
            self.install_chat_profile(view);
        }
        Some(self.view())
    }

    /// Gives back the slot `work` of `chat` when the worker thread that would
    /// have run its command could not be CREATED (profiles-editor L18, L17(a)):
    /// a command that never ran left nothing in flight, so the chat's next ask
    /// must not be dropped for ever.
    ///
    /// Only the slot comes free: what the tab is told about the failure is the
    /// fold's own business, never this.
    pub fn note_chat_work_unstarted(&self, chat: &ChatId, work: ChatWork) {
        let mut held = lock(&self.profile_work);
        let Some(state) = held.get_mut(chat.as_str()) else {
            return;
        };
        match work {
            ChatWork::Read => state.reading = false,
            ChatWork::Save => state.saving = false,
            ChatWork::Apply => state.applying = false,
        }
    }

    /// Takes the profile WORK `command` does for its chat, answering whether the
    /// command may be handed to a worker at all (profiles-editor L18).
    ///
    /// One read and one mode save are in flight per chat at most, and a duplicate
    /// ask is DROPPED here - not queued - because the tab's coalesced read and
    /// its disabled switch already make the second one unnecessary, and because
    /// the point of the cap is to leave that chat's child free. Every other
    /// command takes no slot and is always run.
    pub(super) fn begin_chat_command(&self, command: &ChatCommand) -> bool {
        let Some(work) = ChatWork::of(command) else {
            return true;
        };
        let mut held = lock(&self.profile_work);
        let state = held.entry(command.chat().as_str().to_owned()).or_default();
        let slot = match work {
            ChatWork::Read => &mut state.reading,
            ChatWork::Save => &mut state.saving,
            ChatWork::Apply => &mut state.applying,
        };
        if *slot {
            return false;
        }
        *slot = true;
        true
    }

    /// The view of one read, built off every lock it needs.
    fn compose_view(&self, chat: &ChatId, read: &ChatConfigRead) -> ChatProfileView {
        let session = lock(&self.session_files).clone();
        let session = session
            .as_ref()
            .map(|armed| (&armed.files, armed.store.as_path()));
        view(chat, read, session, self.language())
    }

    /// Stores one chat's view. Nothing else decides what the tab draws: the slice
    /// reads it back through [`Self::chat_profile`].
    fn install_chat_profile(&self, view: ChatProfileView) {
        let mut held = lock(&self.chat_profile);
        *held = Some(view);
    }

    /// Whether `chat` is still one of the app's open chats: the state's own set,
    /// which the user's Close chat takes a chat out of (`Chats::close`). Nothing
    /// else decides that an answer still belongs to a live chat.
    fn is_open(&self, chat: &ChatId) -> bool {
        lock(&self.app).chats.state(chat).is_some()
    }

    /// The Profile tab's view as it stands, for the slice. The projection onto the
    /// foreground chat is [`view_state`](crate::state_view::view_state)'s.
    ///
    /// The save state is overlaid here, from the shell's own map, so every slice
    /// is built with the same truth about a write in flight: a push, an intent's
    /// answer and the first read a view makes all carry it (profiles-editor
    /// T3-mode, T5). Only a SAVE disables the controls: a read in flight is the
    /// shell's own business and travels in no slice (L18).
    /// The two locks are taken one after another, never together.
    pub(super) fn chat_profile(&self) -> Option<ChatProfileView> {
        let mut view = lock(&self.chat_profile).clone()?;
        let work = lock(&self.profile_work);
        let state = work.get(&view.chat_id).cloned().unwrap_or_default();
        view.saving = state.saving;
        view.save_failed = state.failed;
        view.applying = state.applying;
        view.apply_notice = state.notice;
        view.apply_answer = state.answer;
        view.apply_confirmation = state.confirmation;
        view.chain_failed = state.chain_failed;
        Some(view)
    }

    /// Drops the view when it is that chat's, and that chat's work state with it
    /// (which also frees any slot an unanswered command held). Read by the fold
    /// that closes a chat: a view may not outlive the chat it was read for
    /// (L11-4), and neither may a write that will never answer.
    pub(super) fn forget_chat_profile(&self, chat: &ChatId) {
        {
            let mut held = lock(&self.chat_profile);
            if held
                .as_ref()
                .is_some_and(|view| view.chat_id == chat.as_str())
            {
                *held = None;
            }
        }
        lock(&self.profile_work).remove(chat.as_str());
    }
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::sync::{Arc, Mutex};

    use gentle_app::ChatId;
    use gentle_core::i18n::Language;
    use gentle_core::rpc::config_store::{self, GetConfigOutcome, StoreId, StoreValue};
    use gentle_runtime::chat_runtime::{ChatConfigRead, StoreRead};

    use super::{compose, lock, view};
    use crate::contract::{
        ChatProfileAvailabilityView, ChatProfilePinLayerView, ChatProfilePinMarkView,
        ChatProfileView, OrchestratorModeView,
    };
    use crate::settings::FileRead;

    /// The channel's own documented frames, quoted from `desktop-rpc.md` in that
    /// fixture: the profiles, the fallback chains and the pins.
    const FRAMES: &str =
        include_str!("../../../../../crates/gentle-core/tests/fixtures/desktop-rpc-profiles.json");

    /// One session file that bound a profile: the documented fixture of the
    /// reader this view reuses (L9), whose newest bind entry is `team`.
    const SESSION: &str =
        include_str!("../../../../../crates/runtime/tests/fixtures/session-profile.jsonl");

    /// A session file whose profile family ends in a `clear`: the chat uses the
    /// default from then on.
    const CLEARED: &str = concat!(
        r#"{"type":"custom","customType":"gentle-pi.session-profile/v1","data":{"kind":"bind","origin":"user","name":"work","modelProfiles":{}}}"#,
        "\n",
        r#"{"type":"custom","customType":"gentle-pi.session-profile/v1","data":{"kind":"clear"}}"#,
        "\n",
    );

    /// The fixture's frames as one JSON document.
    fn frames() -> serde_json::Value {
        serde_json::from_str(FRAMES).expect("the documented frames are JSON")
    }

    /// One store's value, read by the channel's own parser - the same parse a
    /// real child's answer goes through.
    fn typed<T>(
        value: &serde_json::Value,
        store: StoreId,
        of: impl FnOnce(StoreValue) -> Option<T>,
    ) -> StoreRead<T> {
        let parsed = config_store::parse_store_value(store, value).expect("a documented value");
        StoreRead::Read(of(parsed).expect("the value of the store that was asked for"))
    }

    /// The documented read: all three stores answered, each from its own frame.
    fn documented() -> ChatConfigRead {
        let frames = frames();
        ChatConfigRead {
            profiles: typed(
                &frames["profiles_data"],
                StoreId::Profiles,
                |value| match value {
                    StoreValue::Profiles(document) => Some(document),
                    _ => None,
                },
            ),
            fallbacks: typed(
                &frames["fallbacks_data"],
                StoreId::ProfileFallbacks,
                |value| match value {
                    StoreValue::Fallbacks(document) => Some(document),
                    _ => None,
                },
            ),
            pins: typed(
                &frames["pins_value"]["value"],
                StoreId::Pins,
                |value| match value {
                    StoreValue::Pins(pins) => Some(pins),
                    _ => None,
                },
            ),
        }
    }

    /// The chat of a stored session, named by its path.
    fn stored() -> ChatId {
        ChatId::of_session("/sessions/a.jsonl")
    }

    /// The sessions store `stored` lives in: the folder a `SessionIndex` of this
    /// run's agent directory would enumerate.
    fn store() -> PathBuf {
        PathBuf::from("/sessions")
    }

    /// A reader that records every path it was asked for and answers
    /// [`SESSION`].
    fn recording() -> (FileRead, Arc<Mutex<Vec<PathBuf>>>) {
        let asked: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(Vec::new()));
        let kept = Arc::clone(&asked);
        let files: FileRead = Arc::new(move |path: &Path, _language: Language| {
            lock(&kept).push(path.to_path_buf());
            Ok(Some(SESSION.to_owned()))
        });
        (files, asked)
    }

    /// The documented read, seen as one chat's own.
    fn view_of(read: &ChatConfigRead, session: Option<&str>) -> ChatProfileView {
        compose(&stored(), read, session)
    }

    #[test]
    fn the_view_names_the_chat_its_default_and_every_definition() {
        let view = view_of(&documented(), None);

        assert_eq!(view.chat_id, "session:/sessions/a.jsonl");
        assert_eq!(
            view.default_profile.as_deref(),
            Some("team"),
            "the profiles store's own `active` is the default"
        );
        assert_eq!(view.profiles.len(), 1, "one profile is defined");
        let team = &view.profiles[0];
        assert_eq!(team.name, "team");
        assert_eq!(
            team.agents
                .iter()
                .map(|agent| agent.agent.as_str())
                .collect::<Vec<_>>(),
            vec!["explore", "orchestrator"],
            "every agent of the profile, in the store's own order"
        );
        assert_eq!(
            team.agents[0].model.as_deref(),
            Some("anthropic/claude-sonnet-4")
        );
        assert_eq!(
            team.agents[0].thinking, None,
            "a row the store leaves partial draws nothing rather than a guess"
        );
        assert_eq!(team.agents[1].model.as_deref(), Some("openai/gpt-5"));
        assert_eq!(
            team.chains
                .iter()
                .map(|chain| chain.agent.as_str())
                .collect::<Vec<_>>(),
            vec!["*", "worker"],
            "each chain of the profile, the profile-wide `*` included"
        );
        assert_eq!(
            team.chains[1]
                .steps
                .iter()
                .map(|step| (step.model.as_str(), step.thinking.as_deref()))
                .collect::<Vec<_>>(),
            vec![("openai/gpt-fallback", Some("low"))]
        );
        assert_eq!(view.stores.profiles, ChatProfileAvailabilityView::Read);
        assert_eq!(view.stores.fallbacks, ChatProfileAvailabilityView::Read);
        assert_eq!(view.stores.pins, ChatProfileAvailabilityView::Read);
    }

    #[test]
    fn an_agent_row_carries_the_effort_and_the_tier_the_store_stores() {
        let routes: serde_json::Value = serde_json::from_str(
            r#"{
                "kind": "gentle-pi.agent_model_profiles",
                "version": 1,
                "profiles": {"fast": {"worker": {"model": "p/m", "thinking": "high", "tier": "cheap"}}}
            }"#,
        )
        .expect("a profile document");
        let read = ChatConfigRead {
            profiles: typed(&routes, StoreId::Profiles, |value| match value {
                StoreValue::Profiles(document) => Some(document),
                _ => None,
            }),
            ..ChatConfigRead::unasked()
        };

        let view = view_of(&read, None);

        let worker = &view.profiles[0].agents[0];
        assert_eq!(worker.agent, "worker");
        assert_eq!(worker.thinking.as_deref(), Some("high"));
        assert_eq!(worker.tier.as_deref(), Some("cheap"));
    }

    /// The documented read, with one `workflow` map instead of the frame's own.
    fn with_workflow(workflow: serde_json::Value) -> ChatConfigRead {
        let document = serde_json::json!({
            "kind": "gentle-pi.agent_model_profiles",
            "version": 1,
            "profiles": {
                "quiet": {"orchestrator": {"model": "p/m"}},
                "busy": {"orchestrator": {"model": "p/n"}}
            },
            "workflow": workflow,
        });
        ChatConfigRead {
            profiles: typed(&document, StoreId::Profiles, |value| match value {
                StoreValue::Profiles(document) => Some(document),
                _ => None,
            }),
            ..ChatConfigRead::unasked()
        }
    }

    /// The mode the view carries for one profile of `read`.
    fn mode_of(read: &ChatConfigRead, name: &str) -> OrchestratorModeView {
        view_of(read, None)
            .profiles
            .iter()
            .find(|profile| profile.name == name)
            .expect("the store defines the profile")
            .mode
    }

    #[test]
    fn a_profile_whose_workflow_names_full_delegation_carries_that_mode() {
        let view = view_of(&documented(), None);

        assert_eq!(
            view.profiles[0].mode,
            OrchestratorModeView::FullDelegation,
            "the documented frames name full-delegation for this profile (L22)"
        );
    }

    #[test]
    fn a_profile_the_workflow_does_not_name_is_organic() {
        let read =
            with_workflow(serde_json::json!({"busy": {"orchestratorMode": "full-delegation"}}));

        assert_eq!(
            mode_of(&read, "quiet"),
            OrchestratorModeView::Organic,
            "an absent entry is gentle-pi's own default, never a guess (L22)"
        );
        assert_eq!(
            mode_of(&read, "busy"),
            OrchestratorModeView::FullDelegation,
            "and the profile that names one keeps its own"
        );
    }

    #[test]
    fn a_workflow_entry_that_names_organic_is_organic() {
        let read = with_workflow(serde_json::json!({"quiet": {"orchestratorMode": "organic"}}));

        assert_eq!(mode_of(&read, "quiet"), OrchestratorModeView::Organic);
    }

    #[test]
    fn a_workflow_entry_of_another_profile_is_not_this_profiles() {
        let read =
            with_workflow(serde_json::json!({"gone": {"orchestratorMode": "full-delegation"}}));

        assert_eq!(
            mode_of(&read, "quiet"),
            OrchestratorModeView::Organic,
            "the mode is stored PER PROFILE: another name's entry says nothing about this one"
        );
        assert_eq!(mode_of(&read, "busy"), OrchestratorModeView::Organic);
    }

    #[test]
    fn a_session_that_bound_a_profile_carries_it_as_the_approximation_it_is() {
        let view = view_of(&documented(), Some(SESSION));

        assert_eq!(
            view.bound.as_deref(),
            Some("team"),
            "the newest bind entry of the chat's own session file (L9)"
        );
        assert!(
            view.bound_from_session_file,
            "the tab labels the answer: it is read from a file, not from the channel (L8)"
        );
        assert_eq!(
            view.bound_origin.as_deref(),
            Some("local"),
            "the layer the entry recorded, as the record names it"
        );
    }

    #[test]
    fn a_chat_with_no_session_file_uses_the_default() {
        let view = view_of(&documented(), None);

        assert_eq!(view.bound, None, "nothing was ever bound");
        assert!(!view.bound_from_session_file);
        assert_eq!(view.bound_origin, None);
        assert_eq!(
            view.default_profile.as_deref(),
            Some("team"),
            "the chat follows the global default"
        );
    }

    #[test]
    fn a_session_file_that_cleared_the_binding_uses_the_default() {
        let view = view_of(&documented(), Some(CLEARED));

        assert_eq!(
            view.bound, None,
            "a cleared binding is the default, never the binding it replaced"
        );
        assert!(!view.bound_from_session_file);
        assert_eq!(view.bound_origin, None);
    }

    #[test]
    fn a_store_that_was_not_answered_is_unavailable_and_adds_nothing() {
        let read = ChatConfigRead {
            profiles: StoreRead::Unread(GetConfigOutcome::Unsupported),
            ..documented()
        };

        let view = view_of(&read, None);

        assert_eq!(
            view.stores.profiles,
            ChatProfileAvailabilityView::Unavailable
        );
        assert!(
            view.profiles.is_empty(),
            "a store that carried no content draws nothing, never an invented profile"
        );
        assert_eq!(view.default_profile, None);
    }

    #[test]
    fn a_store_that_was_never_asked_is_skipped() {
        let read = ChatConfigRead {
            pins: StoreRead::Skipped,
            ..documented()
        };

        let view = view_of(&read, None);

        assert_eq!(
            view.stores.pins,
            ChatProfileAvailabilityView::Skipped,
            "an earlier store got no answer, so this one was not asked: state 10, with no channel error"
        );
        assert_eq!(view.pins, None, "nothing of the pins is known");
        assert_eq!(
            view.stores.profiles,
            ChatProfileAvailabilityView::Read,
            "one store being skipped never hides another"
        );
    }

    #[test]
    fn a_worktree_that_cannot_be_resolved_is_carried_as_its_own_code() {
        let unresolvable: serde_json::Value = serde_json::from_str(
            r#"{"resolved": false, "reason": "worktree_unresolved", "invalid": [], "stale": []}"#,
        )
        .expect("the no-worktree value of the pins store");
        let read = ChatConfigRead {
            pins: typed(&unresolvable, StoreId::Pins, |value| match value {
                StoreValue::Pins(pins) => Some(pins),
                _ => None,
            }),
            ..documented()
        };

        let view = view_of(&read, None);

        let pins = view.pins.expect("the pins store answered");
        assert!(!pins.resolved);
        assert!(
            pins.worktree_unresolved,
            "no working directory inside a resolvable Git worktree"
        );
        assert_eq!(pins.reason.as_deref(), Some("worktree_unresolved"));
        assert_eq!(pins.winner, None);
        assert!(pins.layers.is_empty(), "no layer named a profile");
    }

    #[test]
    fn each_named_pin_layer_carries_its_own_mark() {
        let view = view_of(&documented(), None);

        let pins = view.pins.expect("the pins store answered");
        assert!(pins.resolved);
        assert_eq!(
            pins.winner,
            Some(ChatProfilePinLayerView {
                source: "repo".to_owned(),
                profile: Some("team".to_owned()),
                mark: ChatProfilePinMarkView::Winner,
            })
        );
        assert_eq!(
            pins.layers,
            vec![
                ChatProfilePinLayerView {
                    source: "repo".to_owned(),
                    profile: Some("team".to_owned()),
                    mark: ChatProfilePinMarkView::Winner,
                },
                ChatProfilePinLayerView {
                    source: "local".to_owned(),
                    profile: Some("retired".to_owned()),
                    mark: ChatProfilePinMarkView::Stale,
                },
            ],
            "the winner and the layer that names a profile the store dropped"
        );
        assert!(!pins.worktree_unresolved);
    }

    #[test]
    fn the_session_file_is_read_for_that_chats_own_path() {
        let (files, asked) = recording();
        let store = store();

        let view = view(
            &stored(),
            &documented(),
            Some((&files, store.as_path())),
            Language::En,
        );

        assert_eq!(
            *lock(&asked),
            vec![PathBuf::from("/sessions/a.jsonl")],
            "the chat's own session file, and nothing else"
        );
        assert_eq!(view.bound.as_deref(), Some("team"));
    }

    #[test]
    fn a_session_file_outside_the_store_is_not_read() {
        let (files, asked) = recording();
        let store = store();

        let view = view(
            &ChatId::of_session("/etc/a.jsonl"),
            &documented(),
            Some((&files, store.as_path())),
            Language::En,
        );

        assert!(
            lock(&asked).is_empty(),
            "the seam is never asked for a file outside the sessions store"
        );
        assert_eq!(
            view.bound, None,
            "nothing was read, so the chat uses the default - never an error"
        );
        assert_eq!(view.default_profile.as_deref(), Some("team"));
    }

    #[test]
    fn a_path_that_climbs_out_of_the_store_is_not_read() {
        let (files, asked) = recording();
        let store = store();

        let view = view(
            &ChatId::of_session("/sessions/../etc/a.jsonl"),
            &documented(),
            Some((&files, store.as_path())),
            Language::En,
        );

        assert!(
            lock(&asked).is_empty(),
            "a `..` component is refused by spelling, before the file is touched"
        );
        assert_eq!(view.bound, None);
    }

    #[test]
    fn a_file_of_another_kind_inside_the_store_is_not_read() {
        let (files, asked) = recording();
        let store = store();

        let view = view(
            &ChatId::of_session("/sessions/notes.txt"),
            &documented(),
            Some((&files, store.as_path())),
            Language::En,
        );

        assert!(
            lock(&asked).is_empty(),
            "only the `.jsonl` files the session list enumerates are session files"
        );
        assert_eq!(view.bound, None);
    }

    #[test]
    fn a_windows_style_path_outside_the_store_is_not_read() {
        let (files, asked) = recording();
        let store = store();

        let view = view(
            &ChatId::of_session(r"C:\Users\someone\.pi\agent\sessions\a.jsonl"),
            &documented(),
            Some((&files, store.as_path())),
            Language::En,
        );

        assert!(
            lock(&asked).is_empty(),
            "a drive letter names another place: nothing is asked, on either platform"
        );
        assert_eq!(view.bound, None);
    }

    #[test]
    fn a_fresh_chat_has_no_session_file_to_read() {
        let (files, asked) = recording();
        let store = store();

        let view = view(
            &ChatId::fresh(),
            &documented(),
            Some((&files, store.as_path())),
            Language::En,
        );

        assert!(
            lock(&asked).is_empty(),
            "a new chat has no session file yet, so there is nothing to ask"
        );
        assert_eq!(view.bound, None);
        assert_eq!(view.default_profile.as_deref(), Some("team"));
    }

    #[test]
    fn a_session_file_that_cannot_be_read_uses_the_default() {
        let denied: FileRead =
            Arc::new(|_path: &Path, _language: Language| Err("permission denied".to_owned()));
        let store = store();

        let view = view(
            &stored(),
            &documented(),
            Some((&denied, store.as_path())),
            Language::En,
        );

        assert_eq!(
            view.bound, None,
            "an unreadable file is the default, never an error and never a guess"
        );
        assert!(!view.bound_from_session_file);
    }
}
