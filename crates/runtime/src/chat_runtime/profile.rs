//! Reading one chat's profile configuration over that chat's own child
//! (profiles-editor T2b, S9).
//!
//! The editor lives INSIDE the chat, and the pieces it draws are the chat's own:
//! the profiles and their fallback chains, and the pin, which belongs to the
//! chat's worktree. So the questions go to the chat's own child, through the one
//! [`DesktopClient`] that chat already has - the same client a helper stop uses,
//! which is what gives one chat ONE `requestId` counter (S8): a stop and a
//! profile read of the same chat number from the same sequence instead of each
//! starting again at `desktop-1`.
//!
//! Three stores, three answers. `get_config` names one store per request
//! (`desktop-rpc.md:99-273`), so this asks for `profiles`, `profile_fallbacks`
//! and `pins` and keeps each answer in its own [`StoreRead`]: a store the owner
//! refuses never hides the content of another.
//!
//! The cost is bounded by ONE deadline. The three reads run sequentially under
//! the client's mutex, so the moment a read gets no answer at all - no child, an
//! inactive host, a timeout, a closed child, an unsupported method - the rest are
//! marked [`StoreRead::Skipped`] and never asked: a stalled child holds the mutex
//! for one deadline, not three.

use gentle_app::ChatId;
use gentle_backend::chat_host::HostDesktopChannel;
use gentle_core::rpc::config_store::{
    read_config, FallbacksDocument, GetConfigOutcome, PinsValue, ProfilesDocument, StoreId,
};
use gentle_core::rpc::{AnswerError, DesktopClient, NoAnswer, StoreValue};

use super::ChatRuntime;

/// One chat's profile configuration, as its own child answered it.
///
/// Every store keeps its own answer, so a degradation is never contagious: `pins`
/// can be the only store this chat could not answer, and the editor still draws
/// the profiles it did get.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatConfigRead {
    /// The `profiles` store: the definitions and the global `active` marker.
    pub profiles: StoreRead<ProfilesDocument>,
    /// The `profile_fallbacks` store: the chains per profile and agent.
    pub fallbacks: StoreRead<FallbacksDocument>,
    /// The `pins` store, as this chat's own worktree resolves it.
    pub pins: StoreRead<PinsValue>,
}

impl ChatConfigRead {
    /// Nothing was asked: the chat has no child to ask (never opened, already
    /// closed, or a run with no host at all).
    ///
    /// Every store says the same
    /// [`NoAnswer::NotConnected`](gentle_core::rpc::NoAnswer::NotConnected),
    /// which is exactly the state a client that never resolved its contract is
    /// in: no method is known to exist, so nothing is written.
    pub fn unasked() -> Self {
        let reason = || GetConfigOutcome::NoAnswer(NoAnswer::NotConnected);
        Self {
            profiles: StoreRead::Unread(reason()),
            fallbacks: StoreRead::Unread(reason()),
            pins: StoreRead::Unread(reason()),
        }
    }
}

/// What reading one store came to.
#[derive(Debug, Clone, PartialEq)]
pub enum StoreRead<T> {
    /// The store answered with its typed content.
    Read(T),
    /// The store produced no content, and this is why. A real answer - a
    /// refusal the owner words itself, an answer this contract does not read, an
    /// unusable payload - so the other stores are still asked.
    Unread(GetConfigOutcome),
    /// Never asked: an earlier store got no answer at all, so asking this one
    /// could only wait out another deadline under the same client.
    Skipped,
}

impl ChatRuntime {
    /// Reads `chat`'s profile configuration through that chat's own child.
    ///
    /// Blocking: the reads wait for the child's answers under the channel's own
    /// deadline, so the shell runs it off the window's thread. One deadline in
    /// total - see the module's own note - because the three reads are sequential
    /// and the first answerless one ends the batch.
    pub fn read_chat_profile(&self, chat: &ChatId) -> ChatConfigRead {
        let Some(client) = self.desktop_client(chat) else {
            // No child, so there is nothing to ask and nothing to write.
            return ChatConfigRead::unasked();
        };
        // One exchange per chat at a time, which is what makes its request ids
        // meaningful; another chat's read runs in parallel.
        let mut client = super::lock(&client);
        let mut reads = Reads {
            client: &mut client,
            stopped: false,
        };
        let profiles = reads.read(StoreId::Profiles, |value| match value {
            StoreValue::Profiles(document) => Some(document),
            _ => None,
        });
        let fallbacks = reads.read(StoreId::ProfileFallbacks, |value| match value {
            StoreValue::Fallbacks(document) => Some(document),
            _ => None,
        });
        let pins = reads.read(StoreId::Pins, |value| match value {
            StoreValue::Pins(pins) => Some(pins),
            _ => None,
        });
        ChatConfigRead {
            profiles,
            fallbacks,
            pins,
        }
    }
}

/// The three reads, one after another, over the chat's one client.
struct Reads<'a> {
    client: &'a mut DesktopClient<HostDesktopChannel>,
    /// Set by the first read that got no answer: from then on nothing is asked.
    stopped: bool,
}

impl Reads<'_> {
    /// Reads one store and types its content with `typed`, which answers `None`
    /// for a value that is not this store's.
    fn read<T>(
        &mut self,
        store: StoreId,
        typed: impl FnOnce(StoreValue) -> Option<T>,
    ) -> StoreRead<T> {
        if self.stopped {
            return StoreRead::Skipped;
        }
        match read_config(self.client, store) {
            GetConfigOutcome::Read(projection) => {
                // The answer names its own store, so a child that answered for
                // another one - or with another store's value - is not this
                // store's read.
                let value = match projection.value {
                    Some(value) if projection.store == store => typed(value),
                    _ => None,
                };
                match value {
                    Some(value) => StoreRead::Read(value),
                    None => StoreRead::Unread(GetConfigOutcome::UnexpectedAnswer(
                        AnswerError::WrongType { field: "value" },
                    )),
                }
            }
            outcome => {
                self.stopped = stops_reading(&outcome);
                StoreRead::Unread(outcome)
            }
        }
    }
}

/// Whether this outcome means the child could not serve the request at all, so
/// another store would only wait out another deadline.
///
/// A refusal and an unreadable answer are NOT here: the child answered, and the
/// next store may answer too.
fn stops_reading(outcome: &GetConfigOutcome) -> bool {
    matches!(
        outcome,
        GetConfigOutcome::NoAnswer(_) | GetConfigOutcome::Unsupported
    )
}
