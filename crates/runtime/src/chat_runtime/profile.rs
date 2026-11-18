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
//!
//! The ONE write this module makes is a profile's orchestration mode
//! (profiles-editor T3-mode, S2, S3): a `set_config` of the whole `profiles`
//! document exactly as the chat's own `get_config` sent it, with only
//! `workflow.<profile>` changed, preconditioned on the read's own
//! `writeTargets[global].writeRevision`, and followed by the same three reads. A
//! write that did not happen writes nothing more and is reported as
//! [`ModeWriteResult::Failed`] with the owner's typed reason.

use gentle_app::ChatId;
use gentle_backend::chat_host::HostDesktopChannel;
use gentle_core::rpc::config_store::{
    read_config_verbatim, with_profile_mode, write_config, ConfigProjection, FallbacksDocument,
    GetConfigOutcome, OrchestratorMode, PinsValue, ProfilesDocument, SetConfigOutcome, StoreId,
    WritableStore, WriteTargetName,
};
use gentle_core::rpc::profile_apply::{
    apply_profile, ApplyMode, ApplyProfileOutcome, ConfirmationToken,
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

/// What one attempt to APPLY a profile came to, with the configuration re-read
/// after an attempt that actually ran (profiles-editor T3b, S4, S8, S9).
///
/// The two modes are the terminal's own two outcomes and they are not
/// interchangeable:
///
/// - [`ApplyMode::Session`] binds the profile to THIS chat's child and writes no
///   store at all, so the re-read is the one that shows the binding;
/// - [`ApplyMode::Default`] is the channel's own global apply, which writes the
///   store's `active` and may answer `confirmation_required` first - the
///   screen's dialog, whose text is the child's and travels verbatim (L30 ruling
///   2).
///
/// `read` is `None` for every answer that wrote nothing (a dialog, a refusal, a
/// child that did not answer, a method it does not advertise): the store cannot
/// have moved, so the tab keeps exactly what it drew.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatProfileApply {
    /// The channel's typed answer, in the terms the page draws: applied, the
    /// screen's dialog, or one of the reasons nothing was written.
    pub result: ApplyProfileOutcome,
    /// The three stores read again after an attempt that RAN, exactly as a plain
    /// read reads them.
    pub read: Option<ChatConfigRead>,
}

impl ChatProfileApply {
    /// Nothing was asked, because the chat has no child to ask: no profile was
    /// applied and there is nothing to read.
    pub fn unasked() -> Self {
        Self {
            result: ApplyProfileOutcome::NoAnswer(NoAnswer::NotConnected),
            read: None,
        }
    }
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
        reads.the_three()
    }

    /// Saves one profile's orchestration mode over `chat`'s own child, and then
    /// re-reads that chat's profile configuration exactly as
    /// [`Self::read_chat_profile`] does (profiles-editor T3-mode, S2, S3).
    ///
    /// The write is a `set_config` of the WHOLE `profiles` document, as that
    /// child's own `get_config` sent it ([`with_profile_mode`] changes one
    /// entry of it), preconditioned on the read's `writeTargets[global]`
    /// `writeRevision` - never on the store's `revision`. Every failure - a
    /// refusal the owner words itself, a token that went stale, an apply in
    /// flight, a child that does not answer or does not advertise the method -
    /// writes nothing more and is reported as [`ModeWriteResult::Failed`], so the
    /// page can say the mode could not be saved and draw the re-read truth.
    ///
    /// Blocking, for the same reason and under the same one-deadline bound as
    /// the read: the caller runs it off the window's thread.
    pub fn save_chat_profile_mode(
        &self,
        chat: &ChatId,
        profile: &str,
        mode: OrchestratorMode,
    ) -> ChatModeWrite {
        let Some(client) = self.desktop_client(chat) else {
            return ChatModeWrite::unasked();
        };
        // ONE exchange with this chat at a time, read and write in the same
        // hold: the token this read reports is the one the write sends, and the
        // chat's single request counter numbers both.
        let mut client = super::lock(&client);
        let mut reads = Reads {
            client: &mut client,
            stopped: false,
        };
        let result = reads.write_profile_mode(profile, mode);
        // Whatever came of the attempt, the tab draws the store's truth, and the
        // read is the SAME one a plain read makes (S6: never a broken pane).
        let read = reads.the_three();
        ChatModeWrite { result, read }
    }

    /// Applies `profile` over `chat`'s OWN child (profiles-editor T3b, S4, S8,
    /// S9) and, when the attempt ran, re-reads that chat's profile configuration
    /// exactly as [`Self::read_chat_profile`] does.
    ///
    /// The request is the chat's own: `session` binds this chat and no other, and
    /// `default` is the channel's global apply, which is the only one that can
    /// answer [`ApplyProfileOutcome::ConfirmationRequired`]. That answer is
    /// returned with the child's own title, message and token untouched, so the
    /// caller shows it as it arrived and resends it only if the human approves
    /// (L30 ruling 2).
    ///
    /// Blocking on the channel's deadline under the chat's one client, so the
    /// caller runs it off the window's thread. The re-read is the same three
    /// stores a plain read asks, in the same order, under the same one-deadline
    /// bound.
    pub fn apply_chat_profile(
        &self,
        chat: &ChatId,
        profile: &str,
        mode: ApplyMode,
        confirmation: Option<&ConfirmationToken>,
    ) -> ChatProfileApply {
        let Some(client) = self.desktop_client(chat) else {
            return ChatProfileApply::unasked();
        };
        // ONE exchange with this chat at a time: the apply, and the re-read that
        // follows it, are numbered by the chat's single request counter.
        let mut client = super::lock(&client);
        let result = apply_profile(&mut client, profile, mode, confirmation);
        // Only an attempt that RAN can have moved the store (the session file for
        // `session`, `active` and the routing for `default`); an answer that
        // wrote nothing leaves the tab with the view it drew.
        let read = match &result {
            ApplyProfileOutcome::Applied(_) | ApplyProfileOutcome::NotApplied(_) => {
                let mut reads = Reads {
                    client: &mut client,
                    stopped: false,
                };
                Some(reads.the_three())
            }
            _ => None,
        };
        ChatProfileApply { result, read }
    }
}

/// What saving one profile's orchestration mode came to (profiles-editor
/// T3-mode), with the chat's profile configuration re-read after the attempt.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatModeWrite {
    pub result: ModeWriteResult,
    /// The three stores read again after the attempt, exactly as a plain read
    /// reads them. A failure writes nothing more, and this is still the truth
    /// the tab draws.
    pub read: ChatConfigRead,
}

impl ChatModeWrite {
    /// Nothing was asked, because the chat has no child to ask: every store is
    /// the not-connected reason, the same state [`ChatConfigRead::unasked`]
    /// describes.
    pub fn unasked() -> Self {
        Self {
            result: ModeWriteResult::failed(SetConfigOutcome::NoAnswer(NoAnswer::NotConnected)),
            read: ChatConfigRead::unasked(),
        }
    }
}

/// What one mode save came to, in the terms the page draws: saved, already the
/// stored mode (nothing to say), or one of the reasons nothing was written.
#[derive(Debug, Clone, PartialEq)]
pub enum ModeWriteResult {
    /// The owner accepted the document and stored it (or found it identical).
    Saved,
    /// The store already records this mode, so nothing was written at all.
    Unchanged,
    /// `value.profiles` has no such key: the store defines no profile by that
    /// name, and nothing was written.
    UnknownProfile,
    /// Nothing was written, and this is the typed reason the owner (or the
    /// channel) gave: `revision_conflict`, `invalid_data`,
    /// `profile_apply_busy`, an unsupported method, or no answer at all.
    ///
    /// Boxed: a typed reason is far larger than the other variants, and every
    /// result would pay for it.
    Failed(Box<SetConfigOutcome>),
}

impl ModeWriteResult {
    /// The typed reason nothing was written.
    pub fn failed(outcome: SetConfigOutcome) -> Self {
        Self::Failed(Box::new(outcome))
    }

    /// Whether the tab may treat the chosen mode as the stored one: a save the
    /// owner accepted, or one it did not need to make.
    pub fn saved(&self) -> bool {
        matches!(self, Self::Saved | Self::Unchanged)
    }
}

/// The three reads, one after another, over the chat's one client.
struct Reads<'a> {
    client: &'a mut DesktopClient<HostDesktopChannel>,
    /// Set by the first read that got no answer: from then on nothing is asked.
    stopped: bool,
}

impl Reads<'_> {
    /// The three stores one chat's profile is read from, in the order it was
    /// asked: `profiles`, `profile_fallbacks`, `pins`.
    fn the_three(&mut self) -> ChatConfigRead {
        let profiles = self.read(StoreId::Profiles, |value| match value {
            StoreValue::Profiles(document) => Some(document.clone()),
            _ => None,
        });
        let fallbacks = self.read(StoreId::ProfileFallbacks, |value| match value {
            StoreValue::Fallbacks(document) => Some(document.clone()),
            _ => None,
        });
        let pins = self.read(StoreId::Pins, |value| match value {
            StoreValue::Pins(pins) => Some(pins.clone()),
            _ => None,
        });
        ChatConfigRead {
            profiles,
            fallbacks,
            pins,
        }
    }

    /// Reads one store and types its content with `typed`, which answers `None`
    /// for a value that is not this store's.
    fn read<T>(
        &mut self,
        store: StoreId,
        typed: impl FnOnce(&StoreValue) -> Option<T>,
    ) -> StoreRead<T> {
        match self.ask(store) {
            Asked::Skipped => StoreRead::Skipped,
            Asked::Answered(outcome) => StoreRead::Unread(outcome),
            Asked::Projection { projection, .. } => {
                match projection.value.as_ref().and_then(typed) {
                    Some(value) => StoreRead::Read(value),
                    None => StoreRead::Unread(GetConfigOutcome::UnexpectedAnswer(
                        AnswerError::WrongType { field: "value" },
                    )),
                }
            }
        }
    }

    /// The attempt of one mode save: the `profiles` read that carries the
    /// document and its token, and the `set_config` it needs - or the reason no
    /// write was made.
    fn write_profile_mode(&mut self, profile: &str, mode: OrchestratorMode) -> ModeWriteResult {
        let (projection, verbatim) = match self.ask(StoreId::Profiles) {
            Asked::Projection {
                projection,
                verbatim,
            } => (projection, verbatim),
            Asked::Answered(outcome) => {
                return ModeWriteResult::failed(as_set_config(outcome));
            }
            // The first store of a chat's own read is never skipped: nothing was
            // asked before it.
            Asked::Skipped => {
                return ModeWriteResult::failed(SetConfigOutcome::NoAnswer(NoAnswer::NotConnected));
            }
        };
        let Some(StoreValue::Profiles(document)) = projection.value.as_ref() else {
            return ModeWriteResult::failed(SetConfigOutcome::UnexpectedAnswer(
                AnswerError::WrongType { field: "value" },
            ));
        };
        // A name the store does not define is not something to add: the write
        // sets the mode of a profile that EXISTS.
        if !document.profiles.contains_key(profile) {
            return ModeWriteResult::UnknownProfile;
        }
        // `organic` is absence, so an absent entry is the mode already stored.
        let stored = document
            .workflow
            .get(profile)
            .copied()
            .unwrap_or(OrchestratorMode::Organic);
        if stored == mode {
            return ModeWriteResult::Unchanged;
        }
        // The document the owner sent, never the typed reading of it: the typed
        // half drops every member this contract does not model, and `data` is
        // the COMPLETE content of the store (S2).
        let Some(verbatim) = verbatim.as_ref() else {
            return ModeWriteResult::failed(SetConfigOutcome::UnexpectedAnswer(
                AnswerError::MissingField { field: "value" },
            ));
        };
        let Some(data) = with_profile_mode(verbatim, profile, mode) else {
            return ModeWriteResult::UnknownProfile;
        };
        // S3: the token is the write target's, re-read per store per write, and
        // a read that offers none cannot write this store.
        let Some(expected) = projection.write_revision(WriteTargetName::Global) else {
            return ModeWriteResult::failed(SetConfigOutcome::UnexpectedAnswer(
                AnswerError::MissingField {
                    field: "writeRevision",
                },
            ));
        };
        match write_config(self.client, WritableStore::Profiles, data, expected) {
            SetConfigOutcome::Written(_) => ModeWriteResult::Saved,
            failure => ModeWriteResult::failed(failure),
        }
    }

    /// One `get_config`, typed as the answer its own store carries, or the
    /// reason there is none.
    fn ask(&mut self, store: StoreId) -> Asked {
        if self.stopped {
            return Asked::Skipped;
        }
        let (outcome, verbatim) = read_config_verbatim(self.client, store);
        match outcome {
            GetConfigOutcome::Read(projection) => {
                // The answer names its own store, so a child that answered for
                // another one is not this store's read.
                if projection.store == store {
                    Asked::Projection {
                        projection,
                        verbatim,
                    }
                } else {
                    Asked::Answered(GetConfigOutcome::UnexpectedAnswer(AnswerError::WrongType {
                        field: "value",
                    }))
                }
            }
            outcome => {
                self.stopped = stops_reading(&outcome);
                Asked::Answered(outcome)
            }
        }
    }
}

/// What one store's `get_config` came to.
enum Asked {
    /// This store's answer: typed, and its `value` exactly as the owner sent it.
    Projection {
        projection: ConfigProjection,
        verbatim: Option<serde_json::Value>,
    },
    /// The child answered, and not with this store's content.
    Answered(GetConfigOutcome),
    /// Never asked: an earlier store got no answer at all.
    Skipped,
}

/// A read's failure as the write's, for the results that are reported as one:
/// the write never happened, so the reason is the read's own.
fn as_set_config(outcome: GetConfigOutcome) -> SetConfigOutcome {
    match outcome {
        GetConfigOutcome::Rejected(error) => SetConfigOutcome::Rejected(error),
        GetConfigOutcome::Unsupported => SetConfigOutcome::Unsupported,
        GetConfigOutcome::NoAnswer(reason) => SetConfigOutcome::NoAnswer(reason),
        GetConfigOutcome::InvalidRequest(error) => SetConfigOutcome::InvalidRequest(error),
        GetConfigOutcome::UnexpectedAnswer(error) => SetConfigOutcome::UnexpectedAnswer(error),
        // A store this slice does not type is not a write's own failure: it is
        // the value the write would have edited, and it is missing.
        GetConfigOutcome::Read(_) => {
            SetConfigOutcome::UnexpectedAnswer(AnswerError::WrongType { field: "value" })
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
