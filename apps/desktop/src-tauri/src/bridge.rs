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

use gentle_app::Notice;
use tauri_specta::Event;

use crate::contract::{NoticeLevel, NoticeView, StateChanged, UiIntent, ViewState};

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
        self.fold(gentle_app::Input::Intent(to_intent(intent)))
    }

    /// Folds one reducer input through the same path. The intents arrive as
    /// [`UiIntent`]; this is the slot the later slices fold the backend and
    /// chat events through (`Input::ChatEvent`, `Input::SessionsPrepared`),
    /// and it is what the tests seed state with.
    pub fn fold_input(&self, input: gentle_app::Input) -> ViewState {
        self.fold(input)
    }

    /// The slice the web side holds right now, without folding anything.
    pub fn view(&self) -> ViewState {
        view_state(&self.state())
    }

    fn fold(&self, input: gentle_app::Input) -> ViewState {
        let app = self.state();
        // This slice executes NO Commands/Effects from the reducer: with no
        // child or refresh wired yet there is nothing to run them on, so they
        // are dropped here. A later slice receives and runs them (task T3's
        // remaining halves).
        let (state, _effects) = gentle_app::update_chats(app, input);
        *self.app.lock().expect("app state mutex poisoned") = state.clone();
        view_state(&state)
    }

    fn state(&self) -> gentle_app::AppState {
        self.app.lock().expect("app state mutex poisoned").clone()
    }
}

/// Maps one intent of the contract to the reducer's intent. Explicit so the
/// contract stays free of `crates/app` types (S2).
fn to_intent(intent: UiIntent) -> gentle_app::Intent {
    match intent {
        UiIntent::SendPrompt { text } => gentle_app::Intent::SendPrompt(text),
        UiIntent::NewChat => gentle_app::Intent::NewChat,
        UiIntent::DismissNotice { id } => gentle_app::Intent::DismissNotice(u64::from(id)),
    }
}

/// The `ViewState` slice of an `AppState`: the pure rule that decides what the
/// web side sees. `foreground_chat` is the id the app names the chat by (the
/// way the UI holds it); notices arrive flattened, newest last.
pub fn view_state(state: &gentle_app::AppState) -> ViewState {
    ViewState {
        foreground_chat: state
            .chats
            .foreground()
            .map(|chat| chat.as_str().to_owned()),
        notices: state.notices.iter().map(NoticeView::of).collect(),
    }
}

impl NoticeView {
    /// Flattens one app notice for display.
    pub(crate) fn of(notice: &Notice) -> Self {
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
fn view_notice_id(notice: &Notice) -> u32 {
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

/// The one command this slice exposes: fold the intent, then emit the new
/// slice on `state-changed`. The emitted event is typed through tauri-specta
/// (its rc.26 in the lock), so the Svelte side gets the same [`ViewState`]
/// type for the command return and the event payload.
#[tauri::command]
#[specta::specta]
pub fn dispatch_intent(
    intent: UiIntent,
    holder: tauri::State<'_, AppStateHolder>,
    app: tauri::AppHandle<tauri::Wry>,
) -> Result<ViewState, String> {
    let view = holder.dispatch(intent);
    StateChanged(view.clone())
        .emit(&app)
        .map_err(|error| error.to_string())?;
    Ok(view)
}
