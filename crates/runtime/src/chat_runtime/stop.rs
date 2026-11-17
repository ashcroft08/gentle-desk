//! Stopping one delegated helper over the chat's own child (helper-stop T3).
//!
//! The request is the desktop channel's `cancel_subagent` (S3), and it goes to
//! the child of the chat that named the helper and no other (S6): the channel
//! resolves the chat's own [`ChatKey`](gentle_backend::chat_host::ChatKey) each
//! time it writes, so a reopened chat reaches its new child.
//!
//! This module also owns the per-chat desktop client registry, because the stop
//! is what needed it first and the chat's profile read (profiles-editor T2b)
//! shares it. One client per chat is the point of the cache: a
//! [`DesktopClient`] numbers its requests and caches the child's capabilities, so
//! keeping one per chat is what gives each chat ONE growing `requestId` counter
//! (S8) instead of every request starting again at `desktop-1`, which two answers
//! arriving together could no longer be told apart by. A chat's client is dropped
//! when the chat is (re)opened, because a fresh child must be asked for its own
//! capabilities before anything is written to it.
//!
//! Silent degradation is the whole failure story (S7): no child, an inactive
//! host, a timeout, a closed child or a refusal answers "not stopped" and stops
//! there - no notice, no mark, nothing the user sees. The reason is logged, never
//! shown.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use gentle_app::ChatId;
use gentle_backend::chat_host::HostDesktopChannel;
use gentle_core::rpc::helper_stop::{stop_helper, StopOutcome};
use gentle_core::rpc::DesktopClient;

use super::{lock, ChatRuntime};

/// One chat's desktop client: its own requestId counter and the capabilities of
/// the child it asked. Every desktop request of that chat goes through it.
pub(super) type ChatDesktopClient = Mutex<DesktopClient<HostDesktopChannel>>;

impl ChatRuntime {
    /// Asks `chat`'s own child to cancel one delegated helper, and answers whether
    /// the child accepted it (helper-stop S2, S3).
    ///
    /// Blocking: the exchange waits for the child's answer under the channel's own
    /// deadline (5 s), so the shell runs it off the window's thread. A chat the
    /// host does not know has no child and answers `false` without writing
    /// anything.
    pub fn stop_helper(&self, chat: &ChatId, helper_id: &str) -> bool {
        let Some(client) = self.desktop_client(chat) else {
            return false;
        };
        // One exchange per chat at a time, which is what makes its request ids
        // meaningful; another chat's stop runs in parallel.
        let mut client = lock(&client);
        match stop_helper(&mut client, helper_id) {
            StopOutcome::Stopped => true,
            other => {
                // Silent for the user, logged for whoever reads the bridge's
                // output: the stop simply did not happen (S7).
                eprintln!(
                    "no stop of helper {helper_id} in {}: {other:?}",
                    chat.label()
                );
                false
            }
        }
    }

    /// The chat's desktop client, created on its first desktop request. `None`
    /// for a chat the host has no key for (a fresh chat that was never opened).
    ///
    /// `pub(super)` so the runtime's own tests can pin S8's one client per chat
    /// without a child: the client is what carries the counter.
    pub(super) fn desktop_client(&self, chat: &ChatId) -> Option<Arc<ChatDesktopClient>> {
        // The registry lock ends before any exchange: it is taken only to read
        // the key, never across a call that blocks on a child.
        let key = self.keys().key_of(chat)?;
        let mut clients = lock(&self.clients);
        Some(Arc::clone(clients.entry(chat.clone()).or_insert_with(
            || {
                Arc::new(Mutex::new(DesktopClient::new(
                    self.host.desktop_channel(key),
                )))
            },
        )))
    }

    /// Forgets the chat's desktop client, so the next request asks the child that
    /// is up then for its capabilities. Called when the chat is (re)opened:
    /// capabilities belong to the child that reported them.
    pub(super) fn forget_desktop_client(&self, chat: &ChatId) {
        lock(&self.clients).remove(chat);
    }
}

/// The type of the registry the runtime holds: one desktop client per chat.
pub(super) type ChatDesktopClients = Mutex<BTreeMap<ChatId, Arc<ChatDesktopClient>>>;
