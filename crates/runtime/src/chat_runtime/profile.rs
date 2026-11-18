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
//! The THREE writes this module makes go through the same store shape, and two of
//! them are whole documents. One saves a profile's orchestration mode
//! (profiles-editor T3-mode, S2, S3): a `set_config` of the whole `profiles`
//! document exactly as the chat's own `get_config` sent it, with only
//! `workflow.<profile>` changed. The second saves ONE agent's fallback chain, or
//! the profile-wide `*` one (profiles-editor T5, L30-3): a `set_config` of the
//! whole `profile_fallbacks` document, with only `profiles.<profile>.<agent>`
//! replaced. Both are preconditioned on their own store's
//! `writeTargets[global].writeRevision`.
//!
//! The third writes ONE pin layer of this chat's worktree (profiles-editor T6):
//! `pins` is the one store whose payload is a LAYER - exactly `{source, profile}`,
//! with `null` clearing it - so there is no document to rebuild and the other
//! layer's file is untouched by construction. Its precondition is THAT layer's own
//! `writeTargets` entry, because `pins` has one file per layer and the other
//! layer's token would conflict for ever (`desktop-rpc.md:246-273`).
//!
//! All three are followed by the same three reads, and all three write nothing
//! when the owner refuses or the answer never comes - reported with the owner's
//! typed reason so the page can say the change was not saved.

use gentle_app::ChatId;
use gentle_backend::chat_host::HostDesktopChannel;
use gentle_core::rpc::config_store::{
    pin_data, read_config_verbatim, validate_chain, validate_pin, with_profile_chain,
    with_profile_mode, write_config, ChainInvalid, ConfigProjection, FallbackStep,
    FallbacksDocument, GetConfigOutcome, OrchestratorMode, PinInvalid, PinSource, PinsValue,
    ProfilesDocument, SetConfigOutcome, StoreId, WritableStore, WriteTargetName,
};
use gentle_core::rpc::profile_apply::{
    apply_profile, ApplyMode, ApplyProfileOutcome, ConfirmationToken,
};
use gentle_core::rpc::{
    AnswerError, DesktopClient, LayerStatus, NoAnswer, StoreValue, WriteRevision,
};

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
    /// The `pins` store: what the layers amount to, which pin files this chat's
    /// worktree has, and which of them a write may replace.
    pub pins: StoreRead<PinsRead>,
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

/// What the `pins` store answered for THIS chat's worktree (profiles-editor T6):
/// what the layers amount to (`value`), which pin FILES the worktree has
/// (`layers[]`, one per file the store read), and which of them a write may
/// replace (that layer's `writeTargets[]` entry).
///
/// The two lists are the projection's own and are not part of `value`, because a
/// write needs them and the value does not carry them: `pins` reports one
/// `writeRevision` PER LAYER (`desktop-rpc.md:246-273`), and a layer outside a
/// resolvable Git worktree offers neither a file nor a token.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PinsRead {
    /// What the pin layers amount to, as the owner evaluates them.
    pub value: PinsValue,
    /// One row per pin file, in the store's own order: `local`, then `repo`.
    pub layers: Vec<PinLayerState>,
    /// The layers a write may replace, from `writeTargets[].target`.
    pub targets: Vec<WriteTargetName>,
}

impl PinsRead {
    /// The read `projection` reports: its `value`, the pin files it found
    /// (`layers[]`) and the layers a write may replace (`writeTargets[]`).
    ///
    /// `value` is the projection's own typed value, passed in because the
    /// projection keeps it typed by store: a caller that already has the
    /// [`PinsValue`] does not parse it twice.
    pub fn of(value: PinsValue, projection: &ConfigProjection) -> Self {
        let layers = projection
            .layers
            .iter()
            .filter_map(|layer| {
                let source = match layer.source {
                    gentle_core::rpc::LayerSource::Local => PinSource::Local,
                    gentle_core::rpc::LayerSource::Repo => PinSource::Repo,
                    // `pins` never has a global or project layer; an answer that
                    // named one is not a pin file and is not drawn as one.
                    _ => return None,
                };
                Some(PinLayerState {
                    source,
                    status: layer.status,
                })
            })
            .collect();
        let targets = projection
            .write_targets
            .iter()
            .map(|target| target.target)
            .collect();
        Self {
            value,
            layers,
            targets,
        }
    }

    /// What `source` holds, as far as the owner's `value` reports it.
    ///
    /// The value names the layer that WINS, the layers that name a profile the
    /// profiles store dropped, and the ones that are not a pin at all. A VALID
    /// layer another one outranks is not named anywhere
    /// (`desktop-rpc.md:211-234`), which is [`PinSlot::Unreported`] and never a
    /// guess.
    pub fn slot(&self, source: PinSource) -> PinSlot {
        if let Some(winner) = &self.value.winner {
            if winner.source == source {
                return match &winner.profile {
                    Some(profile) => PinSlot::Pinned(profile.clone()),
                    None => PinSlot::Invalid,
                };
            }
        }
        for stale in &self.value.stale {
            if stale.source == source {
                return match &stale.profile {
                    Some(profile) => PinSlot::Stale(profile.clone()),
                    None => PinSlot::Invalid,
                };
            }
        }
        if self
            .value
            .invalid
            .iter()
            .any(|layer| layer.source == source)
        {
            return PinSlot::Invalid;
        }
        match self.layer(source).map(|layer| layer.status) {
            // A file that is not there holds no pin, and so does one whose bytes
            // are not a pin at all: nothing takes effect from it.
            Some(LayerStatus::Missing) | Some(LayerStatus::Invalid) | None => PinSlot::Empty,
            // The file reads `ok` and the value does not name it: a valid pin
            // another layer outranks, whose content this client is not told.
            Some(LayerStatus::Ok) => PinSlot::Unreported,
        }
    }

    /// The row the store reported for `source`, when it reported one.
    pub fn layer(&self, source: PinSource) -> Option<&PinLayerState> {
        self.layers.iter().find(|layer| layer.source == source)
    }

    /// Whether a write may replace `source`'s file: its `writeTargets` entry is
    /// present, which outside a resolvable Git worktree it is not
    /// (`desktop-rpc.md:273`).
    pub fn writable(&self, source: PinSource) -> bool {
        self.targets.contains(&source.write_target())
    }
}

/// One pin FILE this chat's worktree has, and the owner reported (`layers[]`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinLayerState {
    pub source: PinSource,
    pub status: LayerStatus,
}

/// What one pin layer holds, as far as the owner's `value` says.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PinSlot {
    /// No pin takes effect from this layer: its file is missing, or it is not a
    /// pin at all.
    Empty,
    /// This layer holds `profile`, and it is the layer that decides here.
    Pinned(String),
    /// This layer names a profile the profiles store no longer defines.
    Stale(String),
    /// This layer's file is not a pin at all.
    Invalid,
    /// This layer's file reads `ok` and the `value` does not name it: a valid
    /// pin another layer outranks, whose content is not reported.
    Unreported,
}

impl PinSlot {
    /// Whether the layer already holds exactly `profile` (`None` clearing it), so
    /// a write would change nothing.
    ///
    /// Only what the owner's own report PROVES counts: the layer that wins, and
    /// the file that is not there. A layer whose content is not reported, or a
    /// broken one, is written to rather than guessed at - an explicit write
    /// replaces a broken pin, which the channel documents as a valid operation
    /// (`desktop-rpc.md:372`).
    fn holds(&self, profile: Option<&str>) -> bool {
        match (self, profile) {
            (Self::Empty, None) => true,
            (Self::Pinned(held), Some(profile)) => held == profile,
            _ => false,
        }
    }
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

    /// Saves ONE chain of one profile - an agent's, or the profile-wide `*` -
    /// over `chat`'s own child, and then re-reads that chat's profile
    /// configuration exactly as [`Self::read_chat_profile`] does (profiles-editor
    /// T5, S2, S3, L30-3).
    ///
    /// The write is a `set_config` of the WHOLE `profile_fallbacks` document, as
    /// that child's own `get_config` sent it
    /// ([`with_profile_chain`](gentle_core::rpc::config_store::with_profile_chain)
    /// replaces one chain of it), preconditioned on THAT store's
    /// `writeTargets[global]` `writeRevision` - never on the store's `revision`,
    /// and never on another store's token.
    ///
    /// The chain is checked before anything is sent
    /// ([`validate_chain`](gentle_core::rpc::config_store::validate_chain)): at
    /// most eight steps, a model per step, no model twice, no empty chain - the
    /// rules the owner's own normalizer enforces, so an `invalid_data` is never
    /// this app's fault. A profile the store records no chain for, an agent key it
    /// does not have, and a chain it already holds write nothing.
    ///
    /// Blocking, for the same reason and under the same one-deadline bound as
    /// the read: the caller runs it off the window's thread.
    pub fn save_chat_profile_chain(
        &self,
        chat: &ChatId,
        profile: &str,
        agent: &str,
        steps: &[FallbackStep],
    ) -> ChatChainWrite {
        let Some(client) = self.desktop_client(chat) else {
            return ChatChainWrite::unasked();
        };
        // ONE exchange with this chat at a time, read and write in the same
        // hold: the token this read reports is the one the write sends, and the
        // chat's single request counter numbers both.
        let mut client = super::lock(&client);
        let mut reads = Reads {
            client: &mut client,
            stopped: false,
        };
        let result = reads.write_profile_chain(profile, agent, steps);
        // Whatever came of the attempt, the tab draws the store's truth, and the
        // read is the SAME one a plain read makes (S6: never a broken pane).
        let read = reads.the_three();
        ChatChainWrite { result, read }
    }

    /// Saves ONE pin layer of this chat's worktree over `chat`'s own child, and
    /// then re-reads that chat's profile configuration exactly as
    /// [`Self::read_chat_profile`] does (profiles-editor T6, S2, S3).
    ///
    /// The write is a `set_config` of the `pins` store naming ONE layer and what
    /// it should hold - `{source, profile}`, with `null` clearing that layer - and
    /// it is preconditioned on THAT layer's own `writeTargets[].writeRevision`.
    /// `pins` has two targets, one per file, so the other layer's token would
    /// answer `revision_conflict` for ever on a file nobody touched; the layer and
    /// its target are one thing here ([`PinSource::write_target`]).
    ///
    /// Unlike the two document stores there is nothing to take verbatim: the
    /// payload IS one layer, so the other layer's file is untouched by
    /// construction (`desktop-rpc.md:322-345`).
    ///
    /// Three things are refused HERE, before a request exists: a blank profile
    /// ([`validate_pin`]), a layer this worktree offers no target for (no
    /// resolvable Git worktree, `config_unavailable` at the owner), and a pin the
    /// layer already holds. Every failure the owner words itself - a stale token,
    /// a name the profiles store dropped, an apply in flight, a child that does not
    /// answer or does not advertise the method - writes nothing more and is
    /// reported as [`PinWriteResult::Failed`].
    ///
    /// Blocking, for the same reason and under the same one-deadline bound as
    /// the read: the caller runs it off the window's thread.
    pub fn save_chat_profile_pin(
        &self,
        chat: &ChatId,
        source: PinSource,
        profile: Option<&str>,
    ) -> ChatPinWrite {
        let Some(client) = self.desktop_client(chat) else {
            return ChatPinWrite::unasked();
        };
        // ONE exchange with this chat at a time, read and write in the same
        // hold: the token this read reports is the one the write sends, and the
        // chat's single request counter numbers both.
        let mut client = super::lock(&client);
        let mut reads = Reads {
            client: &mut client,
            stopped: false,
        };
        let result = reads.write_profile_pin(source, profile);
        // Whatever came of the attempt, the tab draws the store's truth, and the
        // read is the SAME one a plain read makes (S6: never a broken pane).
        let read = reads.the_three();
        ChatPinWrite { result, read }
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

/// What saving one profile's fallback chain came to (profiles-editor T5), with
/// the chat's profile configuration re-read after the attempt.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatChainWrite {
    pub result: ChainWriteResult,
    /// The three stores read again after the attempt, exactly as a plain read
    /// reads them. A failure writes nothing more, and this is still the truth
    /// the tab draws.
    pub read: ChatConfigRead,
}

impl ChatChainWrite {
    /// Nothing was asked, because the chat has no child to ask: every store is
    /// the not-connected reason, the same state [`ChatConfigRead::unasked`]
    /// describes.
    pub fn unasked() -> Self {
        Self {
            result: ChainWriteResult::failed(SetConfigOutcome::NoAnswer(NoAnswer::NotConnected)),
            read: ChatConfigRead::unasked(),
        }
    }
}

/// What one chain save came to, in the terms the page draws: saved, already the
/// stored chain, a chain or profile the store does not record, a chain the
/// owner's own rules refuse (so nothing was even sent), or the typed reason the
/// write did not happen.
#[derive(Debug, Clone, PartialEq)]
pub enum ChainWriteResult {
    /// The owner accepted the document and stored it (or found it identical).
    Saved,
    /// The store already records exactly this chain, so nothing was written.
    Unchanged,
    /// `value.profiles` has no key for that profile: the fallbacks store records
    /// no chain of its, and nothing was written.
    UnknownProfile,
    /// The profile exists and has no chain under that agent key (nor under `*`
    /// when that is the key): nothing was written, and no chain was invented.
    UnknownChain,
    /// A chain the owner's normalizer would refuse, caught before the request
    /// existed: nothing was sent at all.
    Invalid(ChainInvalid),
    /// Nothing was written, and this is the typed reason the owner (or the
    /// channel) gave: `revision_conflict`, `invalid_data`, `profile_apply_busy`,
    /// an unsupported method, or no answer at all.
    ///
    /// Boxed: a typed reason is far larger than the other variants, and every
    /// result would pay for it.
    Failed(Box<SetConfigOutcome>),
}

impl ChainWriteResult {
    /// The typed reason nothing was written.
    pub fn failed(outcome: SetConfigOutcome) -> Self {
        Self::Failed(Box::new(outcome))
    }

    /// Whether the tab may treat the drawn chain as the stored one: a save the
    /// owner accepted, or one it did not need to make.
    pub fn saved(&self) -> bool {
        matches!(self, Self::Saved | Self::Unchanged)
    }
}

/// What saving ONE pin layer came to (profiles-editor T6), with the chat's
/// profile configuration re-read after the attempt.
#[derive(Debug, Clone, PartialEq)]
pub struct ChatPinWrite {
    pub result: PinWriteResult,
    /// The three stores read again after the attempt, exactly as a plain read
    /// reads them. A failure writes nothing more, and this is still the truth
    /// the tab draws.
    pub read: ChatConfigRead,
}

impl ChatPinWrite {
    /// Nothing was asked, because the chat has no child to ask: every store is
    /// the not-connected reason, the same state [`ChatConfigRead::unasked`]
    /// describes.
    pub fn unasked() -> Self {
        Self {
            result: PinWriteResult::failed(SetConfigOutcome::NoAnswer(NoAnswer::NotConnected)),
            read: ChatConfigRead::unasked(),
        }
    }
}

/// What one pin save came to, in the terms the page draws: saved, already the
/// layer's pin (nothing to say), a name or a layer a request may never carry, or
/// the typed reason the write did not happen.
#[derive(Debug, Clone, PartialEq)]
pub enum PinWriteResult {
    /// The owner accepted the layer and stored it (or found it identical).
    Saved,
    /// The layer already holds exactly this pin, so nothing was written at all.
    Unchanged,
    /// A pin the owner's own rules would refuse, caught before the request
    /// existed: a blank profile. Nothing was sent.
    Invalid(PinInvalid),
    /// This chat's worktree offers no file for that layer (no resolvable Git
    /// worktree), so no pin write is accepted: nothing was sent.
    Unavailable,
    /// Nothing was written, and this is the typed reason the owner (or the
    /// channel) gave: `revision_conflict`, `invalid_data`, `config_unavailable`,
    /// `profile_apply_busy`, an unsupported method, or no answer at all.
    ///
    /// Boxed: a typed reason is far larger than the other variants, and every
    /// result would pay for it.
    Failed(Box<SetConfigOutcome>),
}

impl PinWriteResult {
    /// The typed reason nothing was written.
    pub fn failed(outcome: SetConfigOutcome) -> Self {
        Self::Failed(Box::new(outcome))
    }

    /// Whether the tab may treat the drawn pin as the stored one: a save the
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
        let pins = self.pins();
        ChatConfigRead {
            profiles,
            fallbacks,
            pins,
        }
    }

    /// The `pins` store, read with the two lists a pin WRITE needs and the value
    /// alone does not carry: the pin files this worktree has and the layers a
    /// write may replace ([`PinsRead`]).
    fn pins(&mut self) -> StoreRead<PinsRead> {
        match self.ask(StoreId::Pins) {
            Asked::Skipped => StoreRead::Skipped,
            Asked::Answered(outcome) => StoreRead::Unread(outcome),
            Asked::Projection { projection, .. } => {
                match projection.value.as_ref() {
                    Some(StoreValue::Pins(value)) => {
                        StoreRead::Read(PinsRead::of(value.clone(), &projection))
                    }
                    _ => StoreRead::Unread(GetConfigOutcome::UnexpectedAnswer(
                        AnswerError::WrongType { field: "value" },
                    )),
                }
            }
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
        let expected = match expected_revision(&projection) {
            Ok(expected) => expected,
            Err(failure) => return ModeWriteResult::failed(*failure),
        };
        match write_config(self.client, WritableStore::Profiles, data, &expected) {
            SetConfigOutcome::Written(_) => ModeWriteResult::Saved,
            failure => ModeWriteResult::failed(failure),
        }
    }

    /// The attempt of one chain save (profiles-editor T5): the `profile_fallbacks`
    /// read that carries the document and ITS token, the owner's own rules
    /// checked locally, and the `set_config` that follows - or the typed reason no
    /// write was made.
    fn write_profile_chain(
        &mut self,
        profile: &str,
        agent: &str,
        steps: &[FallbackStep],
    ) -> ChainWriteResult {
        let (projection, verbatim) = match self.ask(StoreId::ProfileFallbacks) {
            Asked::Projection {
                projection,
                verbatim,
            } => (projection, verbatim),
            Asked::Answered(outcome) => return ChainWriteResult::failed(as_set_config(outcome)),
            // The first store of a chat's own chain save is never skipped: nothing
            // was asked before it.
            Asked::Skipped => {
                return ChainWriteResult::failed(SetConfigOutcome::NoAnswer(
                    NoAnswer::NotConnected,
                ));
            }
        };
        let Some(StoreValue::Fallbacks(document)) = projection.value.as_ref() else {
            return ChainWriteResult::failed(SetConfigOutcome::UnexpectedAnswer(
                AnswerError::WrongType { field: "value" },
            ));
        };
        // A profile or an agent key the store does not record is not something to
        // add here: the write replaces a chain that EXISTS (the view only offers
        // those, and the terminal is where a new one is created).
        let Some(chains) = document.profiles.get(profile) else {
            return ChainWriteResult::UnknownProfile;
        };
        let Some(stored) = chains.get(agent) else {
            return ChainWriteResult::UnknownChain;
        };
        // The owner's own rules, BEFORE a request exists: a chain it would refuse
        // is never sent, so an `invalid_data` is never this app's fault.
        if let Err(refused) = validate_chain(steps) {
            return ChainWriteResult::Invalid(refused);
        }
        // The chain the store already holds is not written again (the answer
        // would be a no-op, and the page has nothing to say).
        if stored.as_slice() == steps {
            return ChainWriteResult::Unchanged;
        }
        // The document the owner sent, never the typed reading of it: the typed
        // half drops every member this contract does not model, and `data` is the
        // COMPLETE content of the store (S2).
        let Some(verbatim) = verbatim.as_ref() else {
            return ChainWriteResult::failed(SetConfigOutcome::UnexpectedAnswer(
                AnswerError::MissingField { field: "value" },
            ));
        };
        let Some(data) = with_profile_chain(verbatim, profile, agent, steps) else {
            return ChainWriteResult::UnknownChain;
        };
        let expected = match expected_revision(&projection) {
            Ok(expected) => expected,
            Err(failure) => return ChainWriteResult::failed(*failure),
        };
        match write_config(
            self.client,
            WritableStore::ProfileFallbacks,
            data,
            &expected,
        ) {
            SetConfigOutcome::Written(_) => ChainWriteResult::Saved,
            failure => ChainWriteResult::failed(failure),
        }
    }

    /// The attempt of one pin save (profiles-editor T6): the `pins` read that
    /// carries this worktree's write target and ITS token, the owner's own rules
    /// checked locally, and the `set_config` that follows - or the typed reason no
    /// write was made.
    fn write_profile_pin(&mut self, source: PinSource, profile: Option<&str>) -> PinWriteResult {
        // The owner's own rule, BEFORE a request exists: a blank profile is
        // `invalid_data`, so it never leaves the app. Whether the name is one the
        // profiles store DEFINES stays the owner's check - it holds that store,
        // and this read does not.
        if let Err(refused) = validate_pin(profile) {
            return PinWriteResult::Invalid(refused);
        }
        let (projection, pins) = match self.ask(StoreId::Pins) {
            Asked::Projection { projection, .. } => {
                let Some(StoreValue::Pins(value)) = projection.value.as_ref() else {
                    return PinWriteResult::failed(SetConfigOutcome::UnexpectedAnswer(
                        AnswerError::WrongType { field: "value" },
                    ));
                };
                let pins = PinsRead::of(value.clone(), &projection);
                (projection, pins)
            }
            Asked::Answered(outcome) => return PinWriteResult::failed(as_set_config(outcome)),
            // The first store of a chat's own pin save is never skipped: nothing
            // was asked before it.
            Asked::Skipped => {
                return PinWriteResult::failed(SetConfigOutcome::NoAnswer(NoAnswer::NotConnected));
            }
        };
        // No working directory in a resolvable worktree means no truthful pin
        // file (`desktop-rpc.md:273`): a request would only be answered
        // `config_unavailable`, so it is not made.
        if !pins.writable(source) {
            return PinWriteResult::Unavailable;
        }
        // The pin the layer already holds is not written again (the answer would
        // be a no-op, and the page has nothing to say).
        if pins.slot(source).holds(profile) {
            return PinWriteResult::Unchanged;
        }
        let data = match pin_data(source, profile) {
            Ok(data) => data,
            Err(refused) => return PinWriteResult::Invalid(refused),
        };
        // S3: the token is THIS layer's write target's, re-read per write, and a
        // read that offers none cannot write that layer.
        let Some(expected) = projection.write_revision(source.write_target()) else {
            return PinWriteResult::Unavailable;
        };
        match write_config(self.client, WritableStore::Pins, data, expected) {
            SetConfigOutcome::Written(_) => PinWriteResult::Saved,
            failure => PinWriteResult::failed(failure),
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

/// The token a write to the store `projection` names must carry as
/// `expectedRevision`: that store's OWN `writeTargets[global]`, re-read per store
/// per write (S3). A read that offers none is a store this client cannot write.
///
/// The refusal is boxed because a typed `SetConfigOutcome` is far larger than the
/// token, and this result travels on the same stack as the read it was taken from.
fn expected_revision(
    projection: &ConfigProjection,
) -> Result<WriteRevision, Box<SetConfigOutcome>> {
    projection
        .write_revision(WriteTargetName::Global)
        .cloned()
        .ok_or_else(|| {
            Box::new(SetConfigOutcome::UnexpectedAnswer(
                AnswerError::MissingField {
                    field: "writeRevision",
                },
            ))
        })
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
