//! Windowless tests of opening a chat: the reducer's `OpenProcess` goes through
//! the recording fake, never through a child process, and what came of it is
//! folded back through the same inputs the iced shell folds.

mod chat_seam;
use chat_seam::{stored, Answer, Fake, SESSION_PATH};

use gentle_app::{ChatCommand, Input};
use gentle_core::rpc::RpcCommand;
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::{ChatActivityMark, ChatActivityView, NoticeLevel, UiIntent};
use gentle_desk_lib::effects;

#[test]
fn open_chat_asks_for_one_open_process_and_the_next_state_shows_it_opening() {
    let holder = AppStateHolder::default();
    let fake = Fake::answering(Answer::Started);
    let chat = stored();

    let (view, commands) = holder.dispatch_commands(UiIntent::OpenChat {
        chat_id: SESSION_PATH.to_owned(),
    });

    // The reducer answered one command for the chat it just created, and the
    // click is seen at once: the chat is opening before the host answers.
    assert_eq!(
        commands,
        vec![ChatCommand::OpenProcess { chat: chat.clone() }]
    );
    assert_eq!(view.foreground_chat.as_deref(), Some(chat.as_str()));
    assert_eq!(
        view.activity,
        vec![ChatActivityView {
            chat_id: chat.as_str().to_owned(),
            activity: ChatActivityMark::Opening,
        }]
    );

    // Running it asks for exactly one open, and then for the history a stored
    // chat needs; nothing else is started and nothing is sent first.
    let ran = effects::run_one(commands[0].clone(), &fake);
    let opens = fake
        .asked()
        .into_iter()
        .filter(|command| matches!(command, ChatCommand::OpenProcess { .. }))
        .count();
    assert_eq!(opens, 1, "one open per click, never two");
    assert_eq!(
        fake.asked(),
        vec![
            ChatCommand::OpenProcess { chat: chat.clone() },
            // The history, then the session's own questions: its state, its
            // context reading (0.1.12) and the chip's two lists (0.1.11).
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetMessages { id: None },
            },
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetState,
            },
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetSessionStats,
            },
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetAvailableModels,
            },
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetAvailableThinkingLevels,
            },
        ]
    );
    // A fresh child only proved its process was created: nothing is confirmed
    // yet, and the safety net is armed for the chat.
    assert!(ran.inputs.is_empty(), "a fresh start confirms no input yet");
    assert_eq!(ran.confirm_after, Some(chat.clone()));

    // The net's confirmation is what ends the opening phase.
    let after = holder.fold_input(Input::ChatOpened { chat });
    assert!(after.activity.is_empty(), "a confirmed chat is idle");
}

#[test]
fn a_new_chat_starts_its_child_on_the_composers_ask_and_only_once() {
    // welcome-screen T4 (R2): the click allocates the chat and starts nothing, so
    // the window carries no process it does not need. The first ask - the box
    // focused, or the first key typed in it - is what asks the host, and every
    // later ask for the same chat is answered with nothing at all.
    let holder = AppStateHolder::default();
    let fake = Fake::answering(Answer::Started);

    let (allocated, commands) = holder.dispatch_commands(UiIntent::OpenNewChat);
    let chat = allocated
        .foreground_chat
        .clone()
        .expect("a new chat is the foreground one");
    assert!(
        commands.is_empty(),
        "the click starts no child, got {commands:?}"
    );
    assert_eq!(
        allocated.activity,
        vec![ChatActivityView {
            chat_id: chat.clone(),
            activity: ChatActivityMark::Opening,
        }],
        "and the chat the composer draws is Opening, with today's reason"
    );

    let (opening, commands) = holder.dispatch_commands(UiIntent::EnsureStarted {
        chat_id: chat.clone(),
    });
    assert_eq!(commands.len(), 1, "the composer's ask is one open");
    assert!(
        matches!(&commands[0], ChatCommand::OpenProcess { chat: opened } if opened.as_str() == chat),
        "for the chat the slice named, got {:?}",
        commands[0]
    );
    assert_eq!(
        opening.activity[0].activity,
        ChatActivityMark::Opening,
        "nothing is usable until the child answers"
    );

    // A second ask while the child comes up, and a third once it is up, both
    // answer nothing: this chat's one start is behind it.
    let (_, again) = holder.dispatch_commands(UiIntent::EnsureStarted {
        chat_id: chat.clone(),
    });
    assert!(again.is_empty(), "no second child, got {again:?}");

    let chat_id = commands[0].chat().clone();
    let ran = effects::run_one(commands[0].clone(), &fake);
    assert_eq!(
        ran.confirm_after,
        Some(chat_id.clone()),
        "a fresh child's phase ends with its own first event, and the net covers one that never speaks"
    );
    let after = holder.fold_input(Input::ChatOpened { chat: chat_id });
    assert!(after.activity.is_empty(), "the confirmed chat is idle");
    let (_, later) = holder.dispatch_commands(UiIntent::EnsureStarted {
        chat_id: chat.clone(),
    });
    assert!(
        later.is_empty(),
        "a chat whose child is up is not started twice, got {later:?}"
    );
}

#[test]
fn an_already_live_chat_is_confirmed_at_once() {
    let holder = AppStateHolder::default();
    let fake = Fake::answering(Answer::Existing);
    let chat = stored();

    let (_, commands) = holder.dispatch_commands(UiIntent::OpenChat {
        chat_id: SESSION_PATH.to_owned(),
    });
    let ran = effects::run_one(commands[0].clone(), &fake);

    assert_eq!(ran.inputs, vec![Input::ChatOpened { chat: chat.clone() }]);
    assert_eq!(ran.confirm_after, None, "nothing waits for that child");
    // The chip was cleared while the chat was away, so the return asks the
    // state, the context reading and both lists again (R3-001, 0.1.12).
    assert_eq!(
        fake.asked()[1..],
        [
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetState,
            },
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetSessionStats,
            },
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetAvailableModels,
            },
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetAvailableThinkingLevels,
            },
        ]
    );
    let after = holder.fold_inputs(ran.inputs);
    assert!(after.activity.is_empty(), "the chat is not left opening");
}

#[test]
fn a_failed_open_ends_opening_and_leaves_the_error_notice_the_reducer_defines() {
    let holder = AppStateHolder::default();
    let fake = Fake::answering(Answer::OpenFailed("cannot spawn"));

    let (view, commands) = holder.dispatch_commands(UiIntent::OpenChat {
        chat_id: SESSION_PATH.to_owned(),
    });
    assert_eq!(view.activity[0].activity, ChatActivityMark::Opening);

    let ran = effects::run_one(commands[0].clone(), &fake);
    let after = holder.fold_inputs(ran.inputs);

    // The reducer's own rule for a failed start: the opening phase is over and
    // exactly one error notice names the chat. It is not a death.
    assert!(
        after.activity.is_empty(),
        "a failed start is not a dead child, so no Ended mark"
    );
    assert_eq!(ran.confirm_after, None);
    assert_eq!(after.notices.len(), 1, "one notice, never one per attempt");
    assert_eq!(after.notices[0].level, NoticeLevel::Error);
    assert_eq!(after.notices[0].text, "a.jsonl: cannot spawn");
}
