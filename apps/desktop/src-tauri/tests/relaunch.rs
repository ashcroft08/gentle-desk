//! The last step of the orderly close decides between leaving and starting the
//! app again (shell-choice: a home chosen at the welcome question only applies
//! to a new run, so the app relaunches itself after the config is saved). The
//! decision is one flag read at the very end, so it is checked here with fakes
//! for the two ways out; the real relaunch is checked in a window.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

use gentle_desk_lib::close::exit_step;

fn counter() -> Arc<AtomicUsize> {
    Arc::new(AtomicUsize::new(0))
}

fn bump(counter: &Arc<AtomicUsize>) -> impl Fn() + Send + Sync + 'static {
    let counter = Arc::clone(counter);
    move || {
        counter.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn a_plain_close_exits_and_does_not_restart() {
    let (restarts, exits) = (counter(), counter());
    let step = exit_step(
        Arc::new(AtomicBool::new(false)),
        bump(&restarts),
        bump(&exits),
    );

    step();

    assert_eq!(exits.load(Ordering::SeqCst), 1);
    assert_eq!(restarts.load(Ordering::SeqCst), 0);
}

#[test]
fn a_close_that_asked_for_a_relaunch_restarts_and_does_not_exit() {
    let (restarts, exits) = (counter(), counter());
    let step = exit_step(
        Arc::new(AtomicBool::new(true)),
        bump(&restarts),
        bump(&exits),
    );

    step();

    assert_eq!(restarts.load(Ordering::SeqCst), 1);
    assert_eq!(exits.load(Ordering::SeqCst), 0);
}

#[test]
fn the_flag_is_read_when_the_step_runs_not_when_it_is_built() {
    let flag = Arc::new(AtomicBool::new(false));
    let (restarts, exits) = (counter(), counter());
    let step = exit_step(Arc::clone(&flag), bump(&restarts), bump(&exits));

    flag.store(true, Ordering::SeqCst);
    step();

    assert_eq!(restarts.load(Ordering::SeqCst), 1);
    assert_eq!(exits.load(Ordering::SeqCst), 0);
}
