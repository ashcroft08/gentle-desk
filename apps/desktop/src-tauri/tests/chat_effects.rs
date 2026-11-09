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
            ChatCommand::Send {
                chat: chat.clone(),
                rpc: RpcCommand::GetMessages { id: None },
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
