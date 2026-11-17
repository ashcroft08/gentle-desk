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
//! The view is SHELL-owned, next to
//! [`ProfilesView`](crate::contract::ProfilesView): `crates/app` depends only on
//! gentle-core while `gentle-runtime` depends on `gentle-app`, so the reducer
//! could not name the session-file reader even if typed profile data belonged in
//! `Input` (L11-1).
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

use gentle_app::ChatId;
use gentle_core::i18n::Language;
use gentle_core::rpc::config_store::{FallbacksDocument, PinLayer, PinsValue, ProfilesDocument};
use gentle_runtime::chat_runtime::{ChatConfigRead, StoreRead};
use gentle_runtime::session_profile::last_session_profile;

use super::{lock, AppStateHolder};
use crate::contract::{
    ChatProfileAgentView, ChatProfileAvailabilityView, ChatProfileChainView,
    ChatProfileDefinitionView, ChatProfilePinLayerView, ChatProfilePinMarkView,
    ChatProfilePinsView, ChatProfileStepView, ChatProfileStoresView, ChatProfileView, ViewState,
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
        })
        .collect()
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
    pub fn note_chat_profile(&self, chat: &ChatId, read: &ChatConfigRead) -> Option<ViewState> {
        // The app state is the authority on the open set, and it is read before
        // any shell lock is taken, as the shell's other reads are.
        if !self.is_open(chat) {
            return None;
        }
        if !reached_a_store(read) {
            return None;
        }
        // Read off every lock before the state is touched, as the shell's other
        // lines are: the seam, then the language, then the view itself.
        let session = lock(&self.session_files).clone();
        let session = session
            .as_ref()
            .map(|armed| (&armed.files, armed.store.as_path()));
        let view = view(chat, read, session, self.language());
        {
            let mut held = lock(&self.chat_profile);
            *held = Some(view);
        }
        Some(self.view())
    }

    /// Whether `chat` is still one of the app's open chats: the state's own set,
    /// which the user's Close chat takes a chat out of (`Chats::close`). Nothing
    /// else decides that an answer still belongs to a live chat.
    fn is_open(&self, chat: &ChatId) -> bool {
        lock(&self.app).chats.state(chat).is_some()
    }

    /// The Profile tab's view as it stands, for the slice. The projection onto the
    /// foreground chat is [`view_state`](crate::state_view::view_state)'s.
    pub(super) fn chat_profile(&self) -> Option<ChatProfileView> {
        lock(&self.chat_profile).clone()
    }

    /// Drops the view when it is that chat's. Read by the fold that closes a chat:
    /// a view may not outlive the chat it was read for (L11-4).
    pub(super) fn forget_chat_profile(&self, chat: &ChatId) {
        let mut held = lock(&self.chat_profile);
        if held
            .as_ref()
            .is_some_and(|view| view.chat_id == chat.as_str())
        {
            *held = None;
        }
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
        ChatProfileView,
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
