//! Windowless tests of the commands a CHILD EVENT's fold asks for
//! (chat-event-commands, S5): the pump hands them to the armed sink, so a
//! settled turn's context reading is asked for without the chat being reopened.
//!
//! The sink records here instead of starting a `chat-command` worker: no child,
//! no window and no pipe is involved, and what is asserted is exactly what the
//! pump delivered.

mod pump_seam;
use pump_seam::{open_stored, settled, settled_input, Pump};

use gentle_app::ChatCommand;
use gentle_core::rpc::RpcCommand;

/// The user's report: the context chip only moved after closing and reopening
/// the chat. Folding the settled turn through the pump must hand that chat's own
/// `get_session_stats` to the sink, which is what runs it.
#[test]
fn a_settled_child_event_asks_its_own_child_for_the_context_reading() {
    let (holder, chat) = open_stored();
    let pump = Pump::start(&holder);

    pump.send(settled(&chat));
    let (pushed, delivered) = pump.finish_recording();

    assert_eq!(
        pushed.len(),
        1,
        "the pump pushed the slice of the one event"
    );
    // A reverted pump folds with `fold_input` and drops the answer: `delivered`
    // is then empty and the chip keeps the stale number until a reopen.
    assert_eq!(
        delivered,
        vec![ChatCommand::Send {
            chat: chat.clone(),
            rpc: RpcCommand::GetSessionStats,
        }],
        "the settled turn's reading is run, with no close and reopen"
    );
}

/// The counter-case that keeps the intent path's ownership: the same event
/// folded through the seed entry point delivers nothing, even with the sink
/// armed. The delivery belongs to the pump's fold-and-hand-over pair, not to the
/// command list (S4: it never runs twice through both paths either).
#[test]
fn folding_the_same_event_through_the_seed_entry_point_delivers_nothing() {
    let (holder, chat) = open_stored();
    // The pump arms the recording sink the real run arms with its executor.
    let pump = Pump::start(&holder);

    holder.fold_input(settled_input(&chat));
    // The fold itself does ask, and the pump's own entry point still answers
    // the same command on a second fold of the same event: the list is the
    // reducer's, and folding alone runs nothing.
    let (_, commands) = holder.fold_event(settled_input(&chat));
    assert_eq!(
        commands,
        vec![ChatCommand::Send {
            chat: chat.clone(),
            rpc: RpcCommand::GetSessionStats,
        }],
        "the settled turn's fold asks for the reading"
    );

    let (pushed, delivered) = pump.finish_recording();

    assert!(pushed.is_empty(), "nothing was fed to the pump itself");
    assert!(
        delivered.is_empty(),
        "a fold through fold_input delivers nothing, so no command runs twice"
    );
}
