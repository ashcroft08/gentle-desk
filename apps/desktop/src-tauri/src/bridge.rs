//! The bridge that owns the app state (S2 of the migration).
//!
//! [`AppStateHolder`] holds one `gentle_app::AppState` behind a mutex,
//! created at startup and managed by Tauri. Every intent the web sends is
//! folded through the pure reducer in `crates/app`, the same `update_chats`
//! the iced shell folds, and the answer to the web is a [`ViewState`] slice,
//! carried by both the command return and the `state-changed` event.
//!
//! The mapping is explicit: the contract's [`UiIntent`] turns into
//! `gentle_app::Intent` by hand, and the state turns into [`ViewState`] by
//! [`view_state`]. Nothing derives on a `crates/app` type, so the two sides
//! stay independently defined and generator-checked
//! (`tests/contract_bindings.rs`).
//!
//! The holder only answers what the reducer asked for; running those per-chat
//! commands is `effects`' job and the Tauri command layer's.

use gentle_app::{ChatCommand, ChatId};
use gentle_runtime::badge::{Badge, BadgeMark};

use crate::contract::{
    ChatActivityMark, ChatActivityView, NoticeLevel, NoticeView, UiIntent, ViewState,
};

/// The app state the Tauri shell owns. Created at startup, managed by Tauri;
/// nothing else anywhere holds an `AppState`.
#[derive(Default)]
pub struct AppStateHolder {
    app: std::sync::Mutex<gentle_app::AppState>,
}

impl AppStateHolder {
    /// Folds one intent of the contract through the reducer and answers the
    /// slice to push. Also the path the `state-changed` event payload takes.
    pub fn dispatch(&self, intent: UiIntent) -> ViewState {
        self.dispatch_commands(intent).0
    }

    /// [`Self::dispatch`] with the per-chat commands the fold asked for, for the
    /// caller to run through an executor (`effects`). The web sees the same
    /// slice either way: the commands are the shell's business, not the view's.
    pub fn dispatch_commands(&self, intent: UiIntent) -> (ViewState, Vec<ChatCommand>) {
        let state = self.state();
        let input = gentle_app::Input::Intent(to_intent(&state, intent));
        self.apply(input)
    }

    /// Folds one reducer input through the same path. The intents arrive as
    /// [`UiIntent`]; this is the slot the later slices fold the backend and
    /// chat events through (`Input::ChatEvent`, `Input::SessionsPrepared`),
    /// and it is what the tests seed state with.
    pub fn fold_input(&self, input: gentle_app::Input) -> ViewState {
        self.apply(input).0
    }

    /// Folds several inputs in order, answering the slice after the last one.
    /// What an outcome came to is usually more than one input.
    pub fn fold_inputs(&self, inputs: impl IntoIterator<Item = gentle_app::Input>) -> ViewState {
        let mut view = self.view();
        for input in inputs {
            view = self.fold_input(input);
        }
        view
    }

    /// The slice the web side holds right now, without folding anything.
    pub fn view(&self) -> ViewState {
        view_state(&self.state())
    }

    fn apply(&self, input: gentle_app::Input) -> (ViewState, Vec<ChatCommand>) {
        let state = self.state();
        // The shared commands (clipboard, refresh) are still dropped: this
        // slice runs the per-chat commands only.
        let (state, effects) = gentle_app::update_chats(state, input);
        *self.app.lock().expect("app state mutex poisoned") = state.clone();
        (view_state(&state), effects.chat_commands)
    }

    fn state(&self) -> gentle_app::AppState {
        self.app.lock().expect("app state mutex poisoned").clone()
    }
}

/// Maps one intent of the contract to the reducer's intent. Explicit so the
/// contract stays free of `crates/app` types (S2).
fn to_intent(state: &gentle_app::AppState, intent: UiIntent) -> gentle_app::Intent {
    match intent {
        UiIntent::SendPrompt { text } => gentle_app::Intent::SendPrompt(text),
        UiIntent::Abort => gentle_app::Intent::Abort,
        UiIntent::NewChat => gentle_app::Intent::NewChat,
        UiIntent::OpenChat { chat_id } => gentle_app::Intent::OpenChat(chat_for(state, &chat_id)),
        UiIntent::OpenNewChat => gentle_app::Intent::OpenNewChat,
        UiIntent::RestartChat { chat_id } => {
            gentle_app::Intent::RestartChat(chat_for(state, &chat_id))
        }
        UiIntent::DismissNotice { id } => gentle_app::Intent::DismissNotice(u64::from(id)),
    }
}

/// The app's chat for an id the web side sent back.
///
/// The web only ever holds ids the state named - [`ViewState`] names every chat
/// by `ChatId::as_str` - so the lookup is the round trip of an id the view was
/// given, and a chat of no session file (`new-N`, which no path can rebuild)
/// travels back unchanged. An id the state has not heard of is read as the
/// session path of a stored chat, which is how the app builds that chat's id:
/// the sidebar names a row by its path long before any chat exists.
fn chat_for(state: &gentle_app::AppState, chat_id: &str) -> ChatId {
    state
        .chats
        .ids()
        .into_iter()
        .find(|chat| chat.as_str() == chat_id)
        .cloned()
        .unwrap_or_else(|| ChatId::of_session(chat_id))
}

/// The `ViewState` slice of an `AppState`: the pure rule that decides what the
/// web side sees. `foreground_chat` is the id the app names the chat by (the
/// way the UI holds it); notices arrive flattened, newest last, and the
/// activity list carries only the chats that are not idle.
pub fn view_state(state: &gentle_app::AppState) -> ViewState {
    ViewState {
        foreground_chat: state
            .chats
            .foreground()
            .map(|chat| chat.as_str().to_owned()),
        notices: state.notices.iter().map(NoticeView::of).collect(),
        activity: activity_view(state),
    }
}

/// The marks of the chats with something to show, in id order. The rule that
/// turns a chat's activity into a mark is `crates/runtime`'s (`Badge::on`), so
/// an idle chat is simply absent and this list stays as small as the chats that
/// are working, waiting or failed.
fn activity_view(state: &gentle_app::AppState) -> Vec<ChatActivityView> {
    state
        .chats
        .ids()
        .into_iter()
        .filter_map(|chat| {
            let badge = Badge::on(state.chats.activity(chat)?)?;
            Some(ChatActivityView {
                chat_id: chat.as_str().to_owned(),
                activity: view_mark(badge.mark),
            })
        })
        .collect()
}

/// The badge mark as the contract names it: the two enums carry the same six
/// meanings, and the contract's names are the ones already on the wire.
fn view_mark(mark: BadgeMark) -> ChatActivityMark {
    match mark {
        BadgeMark::Working => ChatActivityMark::Working,
        BadgeMark::NewReply => ChatActivityMark::NewReply,
        BadgeMark::Queued => ChatActivityMark::Queued,
        BadgeMark::Opening => ChatActivityMark::Opening,
        BadgeMark::NeedsAttention => ChatActivityMark::NeedsAttention,
        BadgeMark::Ended => ChatActivityMark::Ended,
    }
}

impl NoticeView {
    /// Flattens one app notice for display.
    pub(crate) fn of(notice: &gentle_app::Notice) -> Self {
        Self {
            id: view_notice_id(notice),
            text: notice.text.clone(),
            level: view_notice_level(notice.level),
        }
    }
}

/// The id the dismiss intent refers back to. The app's ids are a never-reused
/// counter that stays far below `u32::MAX` in practice; the contract narrows
/// both sides of the id to `u32`, and the saturating value would mark a
/// collapse, never silently aliasing another notice (each id is unique).
fn view_notice_id(notice: &gentle_app::Notice) -> u32 {
    u32::try_from(notice.id).unwrap_or(u32::MAX)
}

/// The contract's level enum matches the app's notice kinds one to one.
fn view_notice_level(level: gentle_app::NoticeLevel) -> NoticeLevel {
    match level {
        gentle_app::NoticeLevel::Info => NoticeLevel::Info,
        gentle_app::NoticeLevel::Warning => NoticeLevel::Warning,
        gentle_app::NoticeLevel::Error => NoticeLevel::Error,
    }
}
