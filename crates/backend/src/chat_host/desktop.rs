//! The `/gentle:desktop` channel over this host: the answers it observes on its
//! children's events, and the channel a caller asks through.
//!
//! gentle-shell's desktop RPC (contract `gentle-shell.desktop-rpc/v1`) turns a
//! request into an ordinary Pi `prompt` and answers through an Extension UI
//! widget (`widgetKey: "gentle-desktop-rpc"`). The widget frames are control
//! traffic: the chat's reducer drops every widget that is not `gentle-agents`,
//! so they never enter the conversation.
//!
//! The host's event stream has exactly one consumer (the app's event pump), so
//! the channel cannot read it. Instead the forwarder that already tags every
//! child event offers it to [`DesktopHub`] first: the hub matches the payload's
//! `requestId` against the requests waiting for that chat, and the frame goes on
//! to the app unchanged. That is the whole of the observation - no second
//! consumer, no filter on the app's stream.
//!
//! [`HostDesktopChannel`] is the real [`DesktopChannel`]: it writes the request
//! through the existing routing (`ChatHost::send`) and waits on the hub, all
//! under one deadline that covers the write as well as the answer. Every way it
//! can fail is a typed [`ChannelReply`], so a
//! child that ignores the command degrades to today's behaviour instead of
//! breaking a chat.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::{Duration, Instant};

use gentle_core::rpc::{
    parse_widget_payload, widget_payload, ChannelReply, DesktopChannel, DesktopRequest, RpcEvent,
    UiRequest, WidgetPayload,
};

use super::{ChatHost, ChatKey};

/// How long one request waits for its widget answer.
///
/// gentle-shell answers a `/gentle:desktop` prompt within the same turn, so a
/// few seconds is generous; the point of the bound is that a child which does
/// not know the command costs the caller a fixed wait instead of forever.
pub const DEFAULT_DESKTOP_DEADLINE: Duration = Duration::from_secs(5);

/// The widget method the answers arrive under.
const SET_WIDGET: &str = "setWidget";

/// The requests waiting for an answer, per chat and request id.
///
/// The hub owns no child and starts nothing: it only holds the waiters the
/// channel registers and hands them the frame that correlates.
#[derive(Default)]
pub struct DesktopHub {
    waits: Mutex<Vec<Pending>>,
    /// Every waiter gets an id no earlier one had, so its removal can never
    /// drop a waiter that took its place.
    handed_out: AtomicU64,
}

/// One registered wait: who asked, with which id, and where the answer goes.
struct Pending {
    id: u64,
    key: ChatKey,
    request_id: String,
    answers: Sender<ChannelReply>,
}

impl DesktopHub {
    /// Registers a wait for `request_id` in `key`. The returned handle answers
    /// once and removes itself when dropped, so a timed-out request leaves
    /// nothing behind.
    pub(super) fn watch(self: &Arc<Self>, key: &ChatKey, request_id: &str) -> DesktopWait {
        let (answers, waiting) = mpsc::channel();
        let id = self.handed_out.fetch_add(1, Ordering::Relaxed);
        lock(&self.waits).push(Pending {
            id,
            key: key.clone(),
            request_id: request_id.to_owned(),
            answers,
        });
        DesktopWait {
            id,
            waiting,
            hub: Arc::clone(self),
        }
    }

    /// Offers one decoded event of `key`'s child. A `setWidget` frame for the
    /// desktop widget whose payload carries a request id someone is waiting for
    /// answers that waiter; every other event changes nothing here.
    pub(super) fn observe(&self, key: &ChatKey, event: &RpcEvent) {
        let RpcEvent::UiRequest(UiRequest::FireAndForget { method, raw, .. }) = event else {
            return;
        };
        if method != SET_WIDGET {
            return;
        }
        let Some(payload) = widget_payload(raw) else {
            return; // a clearing frame carries no answer
        };
        let Some(WidgetPayload::Response(response)) = parse_widget_payload(payload) else {
            return; // an event, or a payload this contract does not model
        };
        let Some(request_id) = response.request_id else {
            return; // `requestId: null` correlates with nothing
        };
        if let Some(pending) = self.take(key, &request_id) {
            let _ = pending
                .answers
                .send(ChannelReply::Widget(payload.to_owned()));
        }
    }

    /// Answers every waiter of `key` with `Closed`: the child is gone, so no
    /// answer can arrive and nobody waits out a deadline for it.
    pub(super) fn close(&self, key: &ChatKey) {
        let closed = {
            let mut waits = lock(&self.waits);
            let (closed, kept): (Vec<Pending>, Vec<Pending>) =
                waits.drain(..).partition(|pending| pending.key == *key);
            *waits = kept;
            closed
        };
        for pending in closed {
            let _ = pending.answers.send(ChannelReply::Closed);
        }
    }

    fn take(&self, key: &ChatKey, request_id: &str) -> Option<Pending> {
        let mut waits = lock(&self.waits);
        let at = waits
            .iter()
            .position(|pending| pending.key == *key && pending.request_id == request_id)?;
        Some(waits.remove(at))
    }

    fn forget(&self, id: u64) {
        lock(&self.waits).retain(|pending| pending.id != id);
    }
}

/// One registered wait, as the channel holds it.
pub struct DesktopWait {
    id: u64,
    waiting: Receiver<ChannelReply>,
    hub: Arc<DesktopHub>,
}

impl DesktopWait {
    /// Waits at most `deadline` for the correlated answer.
    pub fn recv_timeout(&self, deadline: Duration) -> Result<ChannelReply, RecvTimeoutError> {
        self.waiting.recv_timeout(deadline)
    }
}

impl Drop for DesktopWait {
    fn drop(&mut self) {
        self.hub.forget(self.id);
    }
}

/// The real [`DesktopChannel`]: one chat's child, written through the host's
/// existing routing and answered by the frames the host observes.
pub struct HostDesktopChannel {
    host: Arc<ChatHost>,
    key: ChatKey,
    deadline: Duration,
}

impl HostDesktopChannel {
    pub(super) fn new(host: Arc<ChatHost>, key: ChatKey) -> Self {
        Self {
            host,
            key,
            deadline: DEFAULT_DESKTOP_DEADLINE,
        }
    }

    /// Overrides how long one request waits for its answer.
    pub fn with_deadline(mut self, deadline: Duration) -> Self {
        self.deadline = deadline;
        self
    }
}

impl DesktopChannel for HostDesktopChannel {
    fn exchange(&self, request: &DesktopRequest) -> ChannelReply {
        self.exchange_within(request, self.deadline)
    }

    /// The same exchange under a deadline THIS ask names (the plain reads keep
    /// [`DEFAULT_DESKTOP_DEADLINE`]; pi's `refresh` is bounded at 15 s, so the
    /// explicit check names its own bound instead of failing inside this one).
    /// The bound covers the write as well as the answer, exactly as the default
    /// does.
    fn exchange_within(&self, request: &DesktopRequest, deadline: Duration) -> ChannelReply {
        // The channel only exists for an interactive RPC host; everywhere else
        // it is inactive and nothing is written.
        if !self.host.interactive() {
            return ChannelReply::Inactive;
        }
        // Registered before the write, so an answer can never arrive first.
        let wait = self.host.watch_desktop(&self.key, request.request_id());
        let started = Instant::now();
        // One deadline covers the whole exchange, the write included: a child
        // that has stopped reading its stdin parks `ChatHost::send` inside the
        // pipe for as long as it likes, so the write runs on a thread of its
        // own and this call waits for that thread only while the deadline
        // lasts. A write that does not finish in time is abandoned - its
        // thread ends when the pipe drains or the child goes away, and it holds
        // no lock the host needs - and reported as the same timeout the wait
        // returns, so the caller never blocks past the deadline (R3-1).
        let (done, finished) = mpsc::channel();
        let writer = {
            let host = Arc::clone(&self.host);
            let key = self.key.clone();
            let command = request.command();
            thread::spawn(move || {
                let _ = done.send(host.send(&key, &command).is_ok());
            })
        };
        // Detached: the abandoned writer is not joined, so a parked write can
        // never hold the caller.
        drop(writer);
        match finished.recv_timeout(deadline) {
            Ok(true) => {}
            Ok(false) => return ChannelReply::Closed,
            Err(RecvTimeoutError::Timeout) => return ChannelReply::Timeout,
            // The writer never gave a verdict, so nothing more arrives.
            Err(RecvTimeoutError::Disconnected) => return ChannelReply::Closed,
        }
        // Whatever the write left of the deadline is what the answer gets.
        match wait.recv_timeout(deadline.saturating_sub(started.elapsed())) {
            Ok(reply) => reply,
            Err(RecvTimeoutError::Timeout) => ChannelReply::Timeout,
            Err(RecvTimeoutError::Disconnected) => ChannelReply::Closed,
        }
    }
}

impl ChatHost {
    /// Whether this host's children run as interactive RPC hosts, which is the
    /// gate gentle-shell's desktop RPC channel is active behind.
    pub fn interactive(&self) -> bool {
        self.spawner.interactive()
    }

    /// Registers a wait for `request_id` in `key`, answered by the widget frames
    /// this host observes for that chat.
    pub fn watch_desktop(&self, key: &ChatKey, request_id: &str) -> DesktopWait {
        self.desktop.watch(key, request_id)
    }

    /// The channel into `key`'s child, with the default deadline.
    pub fn desktop_channel(self: &Arc<Self>, key: ChatKey) -> HostDesktopChannel {
        HostDesktopChannel::new(Arc::clone(self), key)
    }
}

/// A lock that survives a poisoned mutex: a thread that died holding the waits
/// must not make every later request panic.
fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[cfg(test)]
mod tests;
