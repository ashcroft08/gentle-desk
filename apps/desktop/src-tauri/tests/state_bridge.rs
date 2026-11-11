//! Windowless tests of the state bridge: the intents of the contract go
//! through the same pure reducer in `gentle-app`, and what comes back is the
//! slice the contract pushes to the web side. No Tauri app, no child process:
//! the holder is a `Mutex<AppState>` the reducer folds into.
//!
//! The reducer is ODD-source-of-truth: nothing here re-implements what
//! `Input` does; the tests seed state through the reducer's public inputs.

use gentle_app::{ChatId, Input, Intent};
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{
    InstallView, ListStatusView, NoticeLevel, ShellView, ThemeView, UiIntent, ViewState,
};

#[test]
fn new_chat_clears_the_foreground_chat() {
    let holder = AppStateHolder::default();
    // Seed through the reducer: a later slice emits OpenChat from the sidebar.
    holder.fold_input(Input::Intent(Intent::OpenChat(ChatId::fresh())));
    let before = holder.view();
    assert!(before.foreground_chat.is_some());

    let after = holder.dispatch(UiIntent::NewChat);

    assert_ne!(after, before);
    assert_eq!(after.foreground_chat, None);
    // The command returns exactly the slice the state-changed event carries.
    assert_eq!(holder.view(), after);
}

#[test]
fn dismiss_notice_removes_only_that_notice() {
    let holder = AppStateHolder::default();
    // Two deaths, two notices: each unexpected close leaves one notice (S3).
    holder.fold_input(Input::ChatClosed {
        chat: ChatId::of_session("first.json"),
        expected: false,
    });
    holder.fold_input(Input::ChatClosed {
        chat: ChatId::of_session("second.json"),
        expected: false,
    });

    let before = holder.view();
    assert_eq!(before.notices.len(), 2);
    let dismissed = before.notices[0].clone();
    let other = before.notices[1].clone();
    // The death notice is an error, and the level enum matches the app's kinds.
    assert_eq!(dismissed.level, NoticeLevel::Error);

    let after = holder.dispatch(UiIntent::DismissNotice { id: dismissed.id });

    assert_eq!(after.notices, vec![other], "only the dismissed notice goes");
}

#[test]
fn blank_send_prompt_is_rejected_and_leaves_the_view_unchanged() {
    let holder = AppStateHolder::default();
    let before = holder.view();

    let after = holder.dispatch(UiIntent::SendPrompt {
        text: "   ".to_owned(),
    });

    assert_eq!(after, before, "a blank prompt folds to no change");
}

#[test]
fn the_view_state_slice_has_no_invented_fields() {
    // The contract pushes exactly the fields the slice defines; the compiler
    // fixes the shape, so this check pins the default: an empty holder has no
    // foreground chat, no notices and no chat to show activity for.
    let holder = AppStateHolder::default();
    assert_eq!(
        holder.view(),
        ViewState {
            // A run that chose no theme follows the reader's own setting.
            theme: ThemeView::System,
            // No pi found, nothing chosen, so nothing is asked.
            shell: ShellView {
                pi_path: None,
                choice: None,
                ask: false,
                // Nothing looked for the launcher, so none is claimed missing.
                launcher_found: true,
                install: InstallView::Idle,
                restart_needed: false,
            },
            foreground_chat: None,
            // No chat is open, so no header title is drawn either.
            foreground_title: None,
            list_status: ListStatusView::Loading { done: 0, total: 0 },
            sessions_revision: 0,
            total_sessions: 0,
            messages: Vec::new(),
            notices: Vec::new(),
            // No chat is open, so no extension is waiting for an answer.
            dialog: None,
            // No chat is open, so there is no helpers panel and nothing to
            // select in one.
            helpers: None,
            selected_helper: None,
            // No helper is selected, so no thread is open either.
            helper_thread: None,
            activity: Vec::new(),
            // Nothing was dropped, so the chat holds no image and nothing was
            // refused.
            images: Vec::new(),
            images_notice: None,
            config_notice: None,
            // The shell-owned line a refused workspace action sets: nothing was
            // refused here, so it carries nothing.
            workspace_notice: None,
            // The same kind of line for a refused session action (T2's rename):
            // nothing was refused here either.
            sessions_notice: None,
            // The launcher is resolved on the first command, so a run that has
            // asked for nothing yet has no fatal bar.
            fatal: None,
        }
    );
}
