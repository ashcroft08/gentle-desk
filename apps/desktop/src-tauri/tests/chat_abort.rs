//! Windowless tests of what an intent that names no child asks for: the abort,
//! and the id round trip every later effect needs. The reducer is the source of
//! truth here too; nothing re-implements what its `Abort` decides.

mod chat_seam;
use chat_seam::{stored, Answer, Fake, SESSION_PATH};

use gentle_app::{ChatCommand, Input, Intent};
use gentle_core::rpc::RpcCommand;
use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::contract::UiIntent;
use gentle_desk_lib::effects;

#[test]
fn abort_with_no_chat_running_asks_the_executor_for_nothing() {
    let holder = AppStateHolder::default();
    let fake = Fake::answering(Answer::Sent);
    let before = holder.view();

    let (view, commands) = holder.dispatch_commands(UiIntent::Abort);
    for command in commands {
        effects::run_one(command, &fake);
    }

    // No chat is open, so the per-chat path answers no command at all; the
    // single-child Abort command is not a chat command and this slice runs none.
    assert!(fake.asked().is_empty(), "nothing running, nothing asked");
    assert_eq!(
        view, before,
        "an abort with nothing to abort changes nothing"
    );
}

#[test]
fn abort_of_an_open_idle_chat_asks_only_for_the_abort() {
    let holder = AppStateHolder::default();
    let fake = Fake::answering(Answer::Sent);
    let chat = stored();
    // Opened and confirmed: the chat is there, nothing runs in it.
    holder.fold_input(Input::Intent(Intent::OpenChat(chat.clone())));
    holder.fold_input(Input::ChatOpened { chat: chat.clone() });
    assert!(holder.view().activity.is_empty());

    let (_, commands) = holder.dispatch_commands(UiIntent::Abort);
    for command in &commands {
        effects::run_one(command.clone(), &fake);
    }

    // What the reducer answers: one send of the abort to that chat, and never
    // a new child.
    assert_eq!(
        commands,
        vec![ChatCommand::Send {
            chat: chat.clone(),
            rpc: RpcCommand::Abort,
        }]
    );
    assert_eq!(fake.asked(), commands);
}

#[test]
fn an_id_the_state_named_round_trips_back_to_the_same_chat() {
    let holder = AppStateHolder::default();
    let (view, _) = holder.dispatch_commands(UiIntent::OpenNewChat);
    let named = view
        .foreground_chat
        .clone()
        .expect("a new chat is the foreground chat");

    let (_, commands) = holder.dispatch_commands(UiIntent::RestartChat {
        chat_id: named.clone(),
    });

    // A chat of no session file is named `new-N`, which no path can rebuild:
    // the lookup goes through the ids the state itself named.
    assert_eq!(commands.len(), 1);
    assert_eq!(commands[0].chat().as_str(), named);
}

#[test]
fn a_session_path_the_state_has_not_heard_of_becomes_that_stored_chat() {
    let holder = AppStateHolder::default();

    let (view, commands) = holder.dispatch_commands(UiIntent::RestartChat {
        chat_id: SESSION_PATH.to_owned(),
    });

    // The sidebar names a stored chat by the path of its session file: the app
    // builds the chat id from it, and the slice names the chat back the way the
    // view holds it from then on.
    assert_eq!(commands, vec![ChatCommand::OpenProcess { chat: stored() }]);
    assert_eq!(view.foreground_chat.as_deref(), Some(stored().as_str()));
}
