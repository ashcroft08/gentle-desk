//! One chat's child must never stall the host. A write to a child that cannot
//! keep up with its pipe parks inside that write; every other operation of the
//! host has to come back anyway, or a single slow chat freezes the window.

use std::sync::mpsc;
use std::sync::Arc;
use std::thread;

use gentle_core::rpc::RpcCommand;

use super::*;
use fake::{FakeSpawner, Gate, WAIT};

/// Runs `operation` on its own thread and fails instead of hanging: with the
/// state lock held across a parked write the call would never come back, and a
/// test that hangs proves nothing.
fn promptly<T: Send + 'static>(what: &str, operation: impl FnOnce() -> T + Send + 'static) -> T {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let _ = tx.send(operation());
    });
    rx.recv_timeout(WAIT)
        .unwrap_or_else(|_| panic!("{what} was stalled by another chat's blocked write"))
}

/// Releases a gate when it leaves scope, so an assertion that fails cannot
/// leave a parked writer behind.
struct Releases(Arc<Gate>);

impl Drop for Releases {
    fn drop(&mut self) {
        self.0.release();
    }
}

#[test]
fn a_blocked_write_of_one_chat_stalls_no_other_chat() {
    let (spawner, writes) = FakeSpawner::with_blocked_writes(saved("a"));
    let spawner = Arc::new(spawner);
    let host = Arc::new(plain(&spawner));
    host.open(saved("a")).unwrap();
    host.open(saved("b")).unwrap();

    let prompt = RpcCommand::Prompt {
        message: "slow".into(),
    };
    let parked = {
        let host = Arc::clone(&host);
        let prompt = prompt.clone();
        thread::spawn(move || host.send(&saved("a"), &prompt))
    };
    writes.wait_until_parked(1);
    let _released = Releases(Arc::clone(&writes));

    let opened = promptly("open", {
        let host = Arc::clone(&host);
        move || host.open(saved("c"))
    });
    assert_eq!(opened.unwrap(), Opened::Started);
    let keys = promptly("live_keys", {
        let host = Arc::clone(&host);
        move || host.live_keys()
    });
    assert_eq!(keys, vec![saved("a"), saved("b"), saved("c")]);
    let moved = promptly("set_working", {
        let host = Arc::clone(&host);
        move || host.set_working(&saved("b"), true)
    });
    assert!(moved);
    let working = promptly("working_count", {
        let host = Arc::clone(&host);
        move || host.working_count()
    });
    assert_eq!(working, 1);
    let to_another = promptly("send to another chat", {
        let host = Arc::clone(&host);
        move || host.send(&saved("b"), &RpcCommand::GetState)
    });
    assert!(to_another.is_ok());
    let foreground = promptly("foreground", {
        let host = Arc::clone(&host);
        move || host.foreground()
    });
    assert_eq!(foreground, Some(saved("c")));
    let closed = promptly("close", {
        let host = Arc::clone(&host);
        move || host.close(&saved("c"))
    });
    assert!(closed);

    drop(_released);
    assert!(parked.join().unwrap().is_ok());
    // The blocked command was written late, and only to its own child.
    assert_eq!(spawner.child(&saved("a")).commands(), vec![prompt]);
    assert_eq!(
        spawner.child(&saved("b")).commands(),
        vec![RpcCommand::GetState]
    );
}
