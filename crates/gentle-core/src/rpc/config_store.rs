//! gentle-pi's three authoritative config stores over the `/gentle:desktop`
//! channel (profiles-editor T2a: S2, S3, S9).
//!
//! The channel names a store, never a path (`docs/desktop-rpc.md:99-273` of
//! gentle-pi): `get_config` takes exactly `{ "store": "<id>" }` and answers the
//! store's projection, `set_config` takes exactly `{ store, data,
//! expectedRevision }` where `data` is the **complete** new content and
//! `expectedRevision` is the `writeTargets[].writeRevision` of the file the write
//! replaces - "never against `revision`" - and a stale token answers
//! `revision_conflict` having written nothing.
//!
//! This module is the pure half of that contract as gentle-desk needs it:
//!
//! - [`get_config_params`] / [`set_config_params`], the two request shapes;
//! - [`ConfigProjection`] and its typed store values, hand-parsed from a
//!   correlated `ok: true` answer so an unknown extra member is ignored (the
//!   channel may grow) while a missing required member is a typed
//!   [`AnswerError`], never a panic;
//! - [`read_config`] / [`write_config`], one round trip each over the fakeable
//!   [`DesktopChannel`] seam, with helper-stop's degradation ladder: the method
//!   is only ever written when the child's own `capabilities` advertised it, and
//!   an inactive host, a timeout, a closed child, an unparsable payload or a
//!   refusal is a typed outcome rather than an error the user sees.
//!
//! Two limits are the contract's and are honored here rather than checked
//! twice: the request is capped at 64 KiB by [`DesktopRequest`], and a read-only
//! store cannot be written because the only way to name a writable store is
//! [`WritableStore`], which [`WritableStore::try_from_store`] refuses for
//! `models`, `subagents` and `managed_assets`.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde_json::{json, Value};

use super::answer::{
    as_object, optional_array, optional_bool, optional_object, optional_str, required_bool,
    required_object, required_str, required_u64, AnswerError, LayerStatus, StoreRevision,
    WriteRevision,
};
use super::desktop::{
    DesktopChannel, DesktopClient, DesktopError, DesktopOutcome, NoAnswer, RequestError,
};

/// The method that reads one store.
pub const GET_CONFIG: &str = "get_config";
/// The method that writes one store.
pub const SET_CONFIG: &str = "set_config";
/// The `kind` every agent-model-profiles document carries.
pub const PROFILES_KIND: &str = "gentle-pi.agent_model_profiles";
/// The only profiles envelope version this reader knows.
pub const PROFILES_VERSION: u64 = 1;
/// The `kind` every profile-fallback-chains document carries.
pub const FALLBACKS_KIND: &str = "gentle-pi.profile_model_fallbacks";
/// The only fallbacks envelope version this reader knows.
pub const FALLBACKS_VERSION: u64 = 1;
/// The agent key whose chain every agent of a profile without one takes.
pub const DEFAULT_CHAIN_KEY: &str = "*";
/// The most steps one fallback chain may hold (`desktop-rpc.md:345`).
pub const MAX_CHAIN_STEPS: usize = 8;
/// The `value.reason` a `pins` read reports outside a resolvable Git worktree
/// (`desktop-rpc.md:234`).
pub const WORKTREE_UNRESOLVED: &str = "worktree_unresolved";

/// One of the six store identifiers the channel names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreId {
    Profiles,
    Models,
    ProfileFallbacks,
    Subagents,
    Pins,
    ManagedAssets,
}

impl StoreId {
    /// The identifier, or `None` for a string the channel would refuse as
    /// `unknown_store`.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "profiles" => Some(Self::Profiles),
            "models" => Some(Self::Models),
            "profile_fallbacks" => Some(Self::ProfileFallbacks),
            "subagents" => Some(Self::Subagents),
            "pins" => Some(Self::Pins),
            "managed_assets" => Some(Self::ManagedAssets),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Profiles => "profiles",
            Self::Models => "models",
            Self::ProfileFallbacks => "profile_fallbacks",
            Self::Subagents => "subagents",
            Self::Pins => "pins",
            Self::ManagedAssets => "managed_assets",
        }
    }
}

/// One of the three stores `set_config` may write.
///
/// The type is the gate: `models`, `subagents` and `managed_assets` answer
/// `read_only` and are rejected before their `data` is inspected, so this module
/// simply never builds such a request. The channel's own `read_only` answer is
/// still parsed and reported, for a child that answers it anyway.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WritableStore {
    Profiles,
    ProfileFallbacks,
    Pins,
}

impl WritableStore {
    /// The writable store `store` names, or the read-only refusal naming it.
    pub fn try_from_store(store: StoreId) -> Result<Self, ReadOnlyStore> {
        match store {
            StoreId::Profiles => Ok(Self::Profiles),
            StoreId::ProfileFallbacks => Ok(Self::ProfileFallbacks),
            StoreId::Pins => Ok(Self::Pins),
            read_only => Err(ReadOnlyStore(read_only)),
        }
    }

    pub fn store(&self) -> StoreId {
        match self {
            Self::Profiles => StoreId::Profiles,
            Self::ProfileFallbacks => StoreId::ProfileFallbacks,
            Self::Pins => StoreId::Pins,
        }
    }

    pub fn as_str(&self) -> &'static str {
        self.store().as_str()
    }

    /// The files one write replaces: one for the two global stores, and the two
    /// pin layers for `pins` (`desktop-rpc.md:246-262`).
    pub fn write_targets(&self) -> &'static [WriteTargetName] {
        match self {
            Self::Profiles | Self::ProfileFallbacks => &[WriteTargetName::Global],
            Self::Pins => &[WriteTargetName::Local, WriteTargetName::Repo],
        }
    }
}

/// A store `set_config` never writes, and the reason a caller learns instead of
/// a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadOnlyStore(pub StoreId);

impl fmt::Display for ReadOnlyStore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "the {} store is read-only: set_config writes profiles, profile_fallbacks and pins",
            self.0.as_str()
        )
    }
}

impl std::error::Error for ReadOnlyStore {}

/// One file a write replaces: `writeTargets[].target`, and the `writeTarget` a
/// successful write answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteTargetName {
    Global,
    Local,
    Repo,
}

impl WriteTargetName {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "global" => Some(Self::Global),
            "local" => Some(Self::Local),
            "repo" => Some(Self::Repo),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Local => "local",
            Self::Repo => "repo",
        }
    }
}

/// Which owner layers a store is assembled from (`scope`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreScope {
    Global,
    GlobalAndProject,
    LocalAndRepo,
}

impl StoreScope {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "global" => Some(Self::Global),
            "global+project" => Some(Self::GlobalAndProject),
            "local+repo" => Some(Self::LocalAndRepo),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::GlobalAndProject => "global+project",
            Self::LocalAndRepo => "local+repo",
        }
    }
}

/// One layer of a store (`layers[].source`): `global` then `project`, or `local`
/// then `repo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerSource {
    Global,
    Project,
    Local,
    Repo,
}

impl LayerSource {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "global" => Some(Self::Global),
            "project" => Some(Self::Project),
            "local" => Some(Self::Local),
            "repo" => Some(Self::Repo),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "global",
            Self::Project => "project",
            Self::Local => "local",
            Self::Repo => "repo",
        }
    }
}

/// The pin layer a resolved, invalid or stale pin names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinSource {
    Local,
    Repo,
}

impl PinSource {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "local" => Some(Self::Local),
            "repo" => Some(Self::Repo),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Local => "local",
            Self::Repo => "repo",
        }
    }
}

/// One file layer of a store, as `layers[]` reports it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigLayer {
    pub source: LayerSource,
    pub status: LayerStatus,
}

/// One file a write may replace, and its own token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteTarget {
    pub target: WriteTargetName,
    pub status: LayerStatus,
    /// The token `set_config` requires as `expectedRevision`.
    pub write_revision: WriteRevision,
}

/// The orchestration mode a profile's `workflow` entry names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrchestratorMode {
    Organic,
    FullDelegation,
}

impl OrchestratorMode {
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "organic" => Some(Self::Organic),
            "full-delegation" => Some(Self::FullDelegation),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Organic => "organic",
            Self::FullDelegation => "full-delegation",
        }
    }
}

/// One agent's routing row: every member is optional because the file allows a
/// partial row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentRoute {
    pub model: Option<String>,
    pub thinking: Option<String>,
    pub tier: Option<String>,
}

/// The normalized `profiles.json` content (`AgentProfilesFile`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProfilesDocument {
    /// Name of the globally active profile. `None` for a cleared marker.
    ///
    /// It is only the global layer: a valid pin outranks it (L8).
    pub active: Option<String>,
    /// profile name -> agent name -> its routing row.
    pub profiles: BTreeMap<String, BTreeMap<String, AgentRoute>>,
    /// profile name -> the orchestration mode it prefers. Non-default modes only,
    /// which is what the owner persists.
    pub workflow: BTreeMap<String, OrchestratorMode>,
}

/// One step of a fallback chain: the model, and the effort gentle-pi switches to
/// with it when the step names one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackStep {
    pub model: String,
    pub thinking: Option<String>,
}

/// The normalized `profile-fallbacks.json` content (`FallbacksFile`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FallbacksDocument {
    /// profile name -> agent name (or `*`) -> its ordered chain.
    ///
    /// The chains are reported as the file holds them: the owner's own rules
    /// (at most eight steps, one model per chain) are its writer's job, and a
    /// read that hid a chain it disliked would report a store that does not
    /// exist.
    pub profiles: BTreeMap<String, BTreeMap<String, Vec<FallbackStep>>>,
}

/// One pin layer: which file said which profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinLayer {
    pub source: PinSource,
    /// The profile name the layer holds. `None` when the layer is named as
    /// invalid without one.
    pub profile: Option<String>,
}

/// What the pin layers amount to (`pins`' `value`).
///
/// A broken or stale pin is exactly the state an operator needs to see, so this
/// value is reported even when the store's `status` is not `ok`
/// (`desktop-rpc.md:211-234`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PinsValue {
    /// Whether a layer named a profile the profiles store still defines.
    pub resolved: bool,
    /// The winning layer and profile.
    pub winner: Option<PinLayer>,
    /// Layers that are not a pin at all.
    pub invalid: Vec<PinLayer>,
    /// Layers that name a profile the profiles store dropped.
    pub stale: Vec<PinLayer>,
    /// Why nothing resolved, when the store only tells it here: today the one
    /// value is [`WORKTREE_UNRESOLVED`], outside a resolvable Git worktree.
    pub reason: Option<String>,
}

impl PinsValue {
    /// Whether this answer is the "no resolvable worktree" one.
    pub fn worktree_unresolved(&self) -> bool {
        self.reason.as_deref() == Some(WORKTREE_UNRESOLVED)
    }
}

/// The normalized content of one store, typed where this slice reads it.
#[derive(Debug, Clone, PartialEq)]
pub enum StoreValue {
    Profiles(ProfilesDocument),
    Fallbacks(FallbacksDocument),
    Pins(PinsValue),
    /// A store this slice does not type (`models`, `subagents`,
    /// `managed_assets`): kept exactly as the owner reported it.
    Other(Value),
}

/// One `get_config` answer, typed.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigProjection {
    pub store: StoreId,
    pub scope: StoreScope,
    pub read_only: bool,
    pub status: LayerStatus,
    /// The store's read-time digest. Never a write token - see [`WriteRevision`].
    pub revision: StoreRevision,
    /// The normalized content, present when the store produced one.
    pub value: Option<StoreValue>,
    pub layers: Vec<ConfigLayer>,
    /// Present only for the three writable stores (`desktop-rpc.md:246-262`).
    pub write_targets: Vec<WriteTarget>,
}

impl ConfigProjection {
    /// The token a write to `target` requires, or `None` when the answer offers
    /// no such target (a read-only store, or a pin read outside a worktree).
    pub fn write_revision(&self, target: WriteTargetName) -> Option<&WriteRevision> {
        self.write_targets
            .iter()
            .find(|entry| entry.target == target)
            .map(|entry| &entry.write_revision)
    }
}

/// A `set_config` answer, typed: the store re-read after the attempt, the target
/// this request replaced, that target's new token, and whether anything changed.
#[derive(Debug, Clone, PartialEq)]
pub struct SetConfigWritten {
    /// The projection of the store **after** the write, never an echo of `data`.
    pub projection: ConfigProjection,
    pub write_target: WriteTargetName,
    /// The token the NEXT write needs as `expectedRevision`.
    pub write_revision: WriteRevision,
    /// `false` for a no-op: nothing was written and no event was emitted.
    pub changed: bool,
}

/// The `get_config` params: exactly the store identifier.
///
/// Any other key - `scope`, `path`, `file`, `physical_path` - is
/// `invalid_request`, so this writes nothing else.
pub fn get_config_params(store: StoreId) -> Value {
    json!({ "store": store.as_str() })
}

/// The `set_config` params: exactly `store`, the **complete** `data` and the
/// `expectedRevision` the caller read from `get_config`'s `writeTargets`.
///
/// The store is a [`WritableStore`], so a read-only store cannot reach this
/// builder, and the token is a [`WriteRevision`], so the store's own `revision`
/// cannot be passed instead.
pub fn set_config_params(store: WritableStore, data: Value, expected: &WriteRevision) -> Value {
    json!({
        "store": store.as_str(),
        "data": data,
        "expectedRevision": expected.as_str(),
    })
}

/// One `get_config` answer: the typed projection, or why there is none.
#[derive(Debug, Clone, PartialEq)]
pub enum GetConfigOutcome {
    /// A correlated `ok` answer this contract reads.
    Read(ConfigProjection),
    /// The child refused: `unknown_store`, `config_unavailable`,
    /// `profiles_unavailable`, `invalid_request`, `internal_error`.
    Rejected(DesktopError),
    /// The child's `capabilities` does not list `get_config`.
    Unsupported,
    /// No usable answer: an inactive host, a timeout, a closed child, an
    /// unparsable payload, or a client that never resolved its contract.
    NoAnswer(NoAnswer),
    /// The request broke a contract limit and never left.
    InvalidRequest(RequestError),
    /// The child answered, but not this contract's answer.
    UnexpectedAnswer(AnswerError),
}

/// One `set_config` answer: the write, or the typed reason there is none.
#[derive(Debug, Clone, PartialEq)]
pub enum SetConfigOutcome {
    /// The owner wrote (or found identical content) and re-read the store.
    Written(SetConfigWritten),
    /// `expectedRevision` was no longer current: nothing was written.
    RevisionConflict(DesktopError),
    /// The owner would reject or silently reshape `data`: nothing was written.
    InvalidData(DesktopError),
    /// The store is not writable (`models`, `subagents`, `managed_assets`).
    ReadOnly(DesktopError),
    /// No session context, no working directory, or a pin write outside a
    /// resolvable Git worktree.
    ConfigUnavailable(DesktopError),
    /// The owner's atomic write threw; the store is as its cleanup left it.
    WriteFailed(DesktopError),
    /// An apply is still running and the store must not be edited mid-apply.
    ApplyBusy(DesktopError),
    /// Any other refusal this slice does not model (`internal_error`, ...).
    Rejected(DesktopError),
    /// The child's `capabilities` does not list `set_config`.
    Unsupported,
    /// No usable answer, for one of the reasons [`NoAnswer`] names.
    NoAnswer(NoAnswer),
    /// The request broke a contract limit - including the 64 KiB envelope cap -
    /// and never left.
    InvalidRequest(RequestError),
    /// The child answered `ok`, but not this contract's answer.
    UnexpectedAnswer(AnswerError),
}

impl SetConfigOutcome {
    /// This outcome's own refusal, mapped from the channel's error code
    /// (`desktop-rpc.md:934-956`).
    fn from_refusal(error: DesktopError) -> Self {
        match error.code.as_str() {
            "revision_conflict" => Self::RevisionConflict(error),
            "invalid_data" => Self::InvalidData(error),
            "read_only" => Self::ReadOnly(error),
            "config_unavailable" => Self::ConfigUnavailable(error),
            "write_failed" => Self::WriteFailed(error),
            "profile_apply_busy" => Self::ApplyBusy(error),
            _ => Self::Rejected(error),
        }
    }
}

/// Reads one store through a client that keeps this chat's own request ids and
/// cached contract.
///
/// The client's first call of a session is `capabilities`; this asks it when the
/// client has not resolved one yet, so a failed connection is retried on the
/// next read instead of poisoning the chat. A client that still has no contract
/// answers [`NoAnswer::NotConnected`] and writes nothing: support is never
/// inferred (`desktop-rpc.md:58-67`).
pub fn read_config<C: DesktopChannel>(
    client: &mut DesktopClient<C>,
    store: StoreId,
) -> GetConfigOutcome {
    ask_config(client, store).0
}

/// [`read_config`] for a caller that must send the store's `value` BACK: the
/// typed outcome, and the store's `value` exactly as the owner sent it.
///
/// This is the one read a whole-document write makes (`set_config`'s `data` is
/// the complete content of the store, so the document must be the owner's own,
/// unknown members and all); the typed projection alone cannot be sent back
/// without losing what this contract does not model.
pub fn read_config_verbatim<C: DesktopChannel>(
    client: &mut DesktopClient<C>,
    store: StoreId,
) -> (GetConfigOutcome, Option<Value>) {
    ask_config(client, store)
}

/// One `get_config` exchange: the typed outcome, and the store's raw `value`.
fn ask_config<C: DesktopChannel>(
    client: &mut DesktopClient<C>,
    store: StoreId,
) -> (GetConfigOutcome, Option<Value>) {
    if client.capabilities().is_none() {
        client.connect();
    }
    match client.ask(GET_CONFIG, get_config_params(store)) {
        DesktopOutcome::Data { data, .. } => match parse_config_verbatim(&data) {
            Ok(answer) => (GetConfigOutcome::Read(answer.projection), answer.verbatim),
            Err(error) => (GetConfigOutcome::UnexpectedAnswer(error), None),
        },
        DesktopOutcome::Rejected { error, .. } => (GetConfigOutcome::Rejected(error), None),
        DesktopOutcome::Unsupported { .. } => (GetConfigOutcome::Unsupported, None),
        DesktopOutcome::NoAnswer(reason) => (GetConfigOutcome::NoAnswer(reason), None),
        DesktopOutcome::Invalid(error) => (GetConfigOutcome::InvalidRequest(error), None),
    }
}

/// Writes one store through a client that keeps this chat's own request ids and
/// cached contract.
///
/// `data` is the complete new content of the store, never a patch: the owner
/// applies it whole or not at all, so a partial document can only be refused
/// (`desktop-rpc.md:275-302`, `:376-383`). `expected` is the token the caller
/// read from this store's `writeTargets`; it comes from a [`WriteRevision`], so
/// the store's own `revision` cannot be passed by mistake.
pub fn write_config<C: DesktopChannel>(
    client: &mut DesktopClient<C>,
    store: WritableStore,
    data: Value,
    expected: &WriteRevision,
) -> SetConfigOutcome {
    if client.capabilities().is_none() {
        client.connect();
    }
    match client.ask(SET_CONFIG, set_config_params(store, data, expected)) {
        DesktopOutcome::Data { data, .. } => match parse_set_config_written(&data) {
            Ok(written) => SetConfigOutcome::Written(written),
            Err(error) => SetConfigOutcome::UnexpectedAnswer(error),
        },
        DesktopOutcome::Rejected { error, .. } => SetConfigOutcome::from_refusal(error),
        DesktopOutcome::Unsupported { .. } => SetConfigOutcome::Unsupported,
        DesktopOutcome::NoAnswer(reason) => SetConfigOutcome::NoAnswer(reason),
        DesktopOutcome::Invalid(error) => SetConfigOutcome::InvalidRequest(error),
    }
}

/// Reads one `get_config` answer (`data` of the response envelope) into its
/// typed projection.
///
/// Unknown extra members are ignored, because the channel may grow; a required
/// member that is absent, of another JSON type, or one this contract does not
/// define is a typed [`AnswerError`].
pub fn parse_config_projection(data: &Value) -> Result<ConfigProjection, AnswerError> {
    let object = as_object(data, "data")?;
    let store = parse_store(&required_str(object, "store")?)?;
    let scope = parse_scope(&required_str(object, "scope")?)?;
    let read_only = required_bool(object, "readOnly")?;
    let status = parse_status(&required_str(object, "status")?)?;
    let revision = parse_store_revision(&required_str(object, "revision")?)?;
    let value = match optional_object(object, "value")? {
        Some(value) => Some(parse_value_object(store, value)?),
        None => None,
    };
    let mut layers = Vec::new();
    for layer in optional_array(object, "layers")? {
        let layer = as_object(layer, "layers")?;
        layers.push(ConfigLayer {
            source: parse_layer_source(&required_str(layer, "source")?)?,
            status: parse_status(&required_str(layer, "status")?)?,
        });
    }
    let mut write_targets = Vec::new();
    for target in optional_array(object, "writeTargets")? {
        let target = as_object(target, "writeTargets")?;
        write_targets.push(WriteTarget {
            target: parse_write_target(&required_str(target, "target")?)?,
            status: parse_status(&required_str(target, "status")?)?,
            write_revision: parse_write_revision(&required_str(target, "writeRevision")?)?,
        });
    }
    Ok(ConfigProjection {
        store,
        scope,
        read_only,
        status,
        revision,
        value,
        layers,
        write_targets,
    })
}

/// One `get_config` answer, with its store's `value` EXACTLY as the owner sent
/// it: the store's projection, and the document a whole-document write must send
/// back (profiles-editor T3-mode, S2).
///
/// [`ConfigProjection::value`] is the TYPED reading, and typing discards every
/// member this contract does not model. A write must not: `set_config`'s `data`
/// is the complete content of the store, so a document rebuilt from the typed
/// half would silently drop whatever the channel grew. `verbatim` is `None` for
/// an answer that carried no `value`, and for one this contract could not read.
pub struct VerbatimAnswer {
    pub projection: ConfigProjection,
    pub verbatim: Option<Value>,
}
/// [`parse_config_projection`], keeping the store's `value` as the owner sent it.
pub fn parse_config_verbatim(data: &Value) -> Result<VerbatimAnswer, AnswerError> {
    let projection = parse_config_projection(data)?;
    let verbatim = as_object(data, "data")?
        .get("value")
        .filter(|value| value.is_object())
        .cloned();
    Ok(VerbatimAnswer {
        projection,
        verbatim,
    })
}

/// The `profiles` document `verbatim` with ONLY `workflow.<profile>` set to
/// `mode`, or `None` when there is nothing this rule may write.
///
/// The document is the store's own `value`, taken exactly as the owner sent it
/// ([`VerbatimAnswer::verbatim`](VerbatimAnswer)), and every member of it
/// survives the edit: `data` is the COMPLETE content of the store, so this
/// changes one entry and touches nothing else (`desktop-rpc.md:275-302`).
///
/// The two modes are the owner's own spelling of them (`desktop-rpc.md:319`,
/// `:329`), and the probe of L24 confirmed both halves on a real child:
///
/// - `full-delegation` SETS the entry; the `workflow` map is created when the
///   document has none;
/// - `organic` REMOVES it, because absence is the default, and a `workflow` the
///   removal empties is removed entirely.
///
/// `None` - nothing is written - for a name the document's `profiles` does not
/// define, for a `value` that is not an object, and for a `workflow` that is not
/// one: a shape this rule does not model is never guessed at.
pub fn with_profile_mode(verbatim: &Value, profile: &str, mode: OrchestratorMode) -> Option<Value> {
    let mut document = verbatim.as_object()?.clone();
    let profiles = document.get("profiles")?.as_object()?;
    if !profiles.contains_key(profile) {
        return None;
    }
    match mode {
        OrchestratorMode::FullDelegation => {
            let workflow = document
                .entry("workflow")
                .or_insert_with(|| Value::Object(serde_json::Map::new()))
                .as_object_mut()?;
            let mut entry = serde_json::Map::new();
            entry.insert(
                "orchestratorMode".to_owned(),
                Value::String(mode.as_str().to_owned()),
            );
            workflow.insert(profile.to_owned(), Value::Object(entry));
        }
        OrchestratorMode::Organic => {
            let emptied = match document.get_mut("workflow") {
                Some(Value::Object(workflow)) => {
                    workflow.remove(profile);
                    workflow.is_empty()
                }
                // No workflow at all is the same answer as an absent entry.
                None => false,
                // A `workflow` this rule cannot read is not a document to write.
                Some(_) => return None,
            };
            if emptied {
                document.remove("workflow");
            }
        }
    }
    Some(Value::Object(document))
}

/// Why one fallback chain is not a chain the owner would keep.
///
/// The owner's own normalizer decides these (`normalizeFallbacksFile`): a chain
/// of at most [`MAX_CHAIN_STEPS`] steps, each step `{model, thinking?}`, a model
/// at most once per chain, and no empty chain (`desktop-rpc.md:345`, `:382`).
/// Checking them here is what keeps an `invalid_data` from ever being this
/// app's fault: a chain this type refuses never leaves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainInvalid {
    /// No steps at all: the owner does not persist an empty chain.
    Empty,
    /// More than [`MAX_CHAIN_STEPS`] steps.
    TooLong { steps: usize },
    /// The same model twice in one chain.
    RepeatedModel { model: String },
    /// A step whose model is absent or blank.
    MissingModel { index: usize },
}

impl fmt::Display for ChainInvalid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "a fallback chain needs at least one step"),
            Self::TooLong { steps } => write!(
                f,
                "a fallback chain holds at most {MAX_CHAIN_STEPS} steps, not {steps}"
            ),
            Self::RepeatedModel { model } => {
                write!(f, "the model {model} appears twice in one chain")
            }
            Self::MissingModel { index } => {
                write!(f, "step {} names no model", index + 1)
            }
        }
    }
}

impl std::error::Error for ChainInvalid {}

/// Whether `steps` is a chain the owner would keep, or the first rule it breaks.
///
/// The rules are the owner's, in the owner's own order of cheapness: the count,
/// then every step, then the repetition. `thinking` is not validated at all - a
/// value the owner refuses is its own `invalid_data`, which the page shows.
pub fn validate_chain(steps: &[FallbackStep]) -> Result<(), ChainInvalid> {
    if steps.is_empty() {
        return Err(ChainInvalid::Empty);
    }
    if steps.len() > MAX_CHAIN_STEPS {
        return Err(ChainInvalid::TooLong { steps: steps.len() });
    }
    let mut seen = BTreeSet::new();
    for (index, step) in steps.iter().enumerate() {
        if step.model.trim().is_empty() {
            return Err(ChainInvalid::MissingModel { index });
        }
        if !seen.insert(step.model.as_str()) {
            return Err(ChainInvalid::RepeatedModel {
                model: step.model.clone(),
            });
        }
    }
    Ok(())
}

/// The `profile_fallbacks` document `verbatim` with ONLY
/// `profiles.<profile>.<agent>` replaced by `steps`, or `None` when there is
/// nothing this rule may write.
///
/// The document is the store's own `value`, taken exactly as the owner sent it
/// ([`VerbatimAnswer::verbatim`](VerbatimAnswer)), so every member of it - the
/// envelope's unknown ones, the other profiles' chains, the chains of this
/// profile the user did not touch, and even an unknown member inside a step of
/// another chain - survives the edit: `data` is the COMPLETE content of the
/// store (`desktop-rpc.md:275-302`).
///
/// The edited chain itself is written as the user built it (`{model, thinking?}`
/// per step), which is the shape the owner documents; a step's members this app
/// does not model are not carried over into the chain being replaced, because
/// the chain being replaced is exactly what the user replaced.
///
/// `None` - nothing is written - for a `value` that is not an object, a
/// `profiles` that is not one, and a profile the document does not record: a
/// chain is edited under a profile that EXISTS, never invented with one.
/// [`validate_chain`] is the caller's step: this only edits.
pub fn with_profile_chain(
    verbatim: &Value,
    profile: &str,
    agent: &str,
    steps: &[FallbackStep],
) -> Option<Value> {
    let mut document = verbatim.as_object()?.clone();
    let chains = document.get_mut("profiles")?.as_object_mut()?;
    let chains = chains.get_mut(profile)?.as_object_mut()?;
    chains.insert(
        agent.to_owned(),
        Value::Array(steps.iter().map(fallback_step_json).collect()),
    );
    Some(Value::Object(document))
}

/// One step as the owner's own shape: `{model, thinking?}`.
fn fallback_step_json(step: &FallbackStep) -> Value {
    let mut object = serde_json::Map::new();
    object.insert("model".to_owned(), Value::String(step.model.clone()));
    if let Some(thinking) = &step.thinking {
        object.insert("thinking".to_owned(), Value::String(thinking.clone()));
    }
    Value::Object(object)
}

/// Reads one store's `value` into the type that store carries.
///
/// The three writable stores are typed; the three read-only ones are kept as
/// the owner reported them, because this slice does not draw them.
pub fn parse_store_value(store: StoreId, value: &Value) -> Result<StoreValue, AnswerError> {
    parse_value_object(store, as_object(value, "value")?)
}

fn parse_value_object(
    store: StoreId,
    value: &serde_json::Map<String, Value>,
) -> Result<StoreValue, AnswerError> {
    match store {
        StoreId::Profiles => parse_profiles(value).map(StoreValue::Profiles),
        StoreId::ProfileFallbacks => parse_fallbacks(value).map(StoreValue::Fallbacks),
        StoreId::Pins => parse_pins(value).map(StoreValue::Pins),
        StoreId::Models | StoreId::Subagents | StoreId::ManagedAssets => {
            Ok(StoreValue::Other(Value::Object(value.clone())))
        }
    }
}

/// Reads one `set_config` answer (`data` of the response envelope): the store's
/// projection after the write, the target it replaced, that target's new token
/// and whether this request changed anything.
pub fn parse_set_config_written(data: &Value) -> Result<SetConfigWritten, AnswerError> {
    let projection = parse_config_projection(data)?;
    let object = as_object(data, "data")?;
    Ok(SetConfigWritten {
        projection,
        write_target: parse_write_target(&required_str(object, "writeTarget")?)?,
        write_revision: parse_write_revision(&required_str(object, "writeRevision")?)?,
        changed: required_bool(object, "changed")?,
    })
}

fn parse_profiles(
    object: &serde_json::Map<String, Value>,
) -> Result<ProfilesDocument, AnswerError> {
    let kind = required_str(object, "kind")?;
    if kind != PROFILES_KIND {
        return Err(AnswerError::UnknownValue {
            field: "kind",
            value: kind,
        });
    }
    let version = required_u64(object, "version")?;
    if version != PROFILES_VERSION {
        return Err(AnswerError::UnknownValue {
            field: "version",
            value: version.to_string(),
        });
    }
    let active = optional_str(object, "active")?;
    let mut profiles = BTreeMap::new();
    for (name, agents) in required_object(object, "profiles")? {
        let agents = as_object(agents, "profiles")?;
        let mut routes = BTreeMap::new();
        for (agent, route) in agents {
            let route = as_object(route, "profiles")?;
            routes.insert(
                agent.clone(),
                AgentRoute {
                    model: optional_str(route, "model")?,
                    thinking: optional_str(route, "thinking")?,
                    tier: optional_str(route, "tier")?,
                },
            );
        }
        profiles.insert(name.clone(), routes);
    }
    let mut workflow = BTreeMap::new();
    if let Some(entries) = optional_object(object, "workflow")? {
        for (name, entry) in entries {
            let entry = as_object(entry, "workflow")?;
            let mode = required_str(entry, "orchestratorMode")?;
            workflow.insert(name.clone(), parse_orchestrator_mode(&mode)?);
        }
    }
    Ok(ProfilesDocument {
        active,
        profiles,
        workflow,
    })
}

fn parse_fallbacks(
    object: &serde_json::Map<String, Value>,
) -> Result<FallbacksDocument, AnswerError> {
    let kind = required_str(object, "kind")?;
    if kind != FALLBACKS_KIND {
        return Err(AnswerError::UnknownValue {
            field: "kind",
            value: kind,
        });
    }
    let version = required_u64(object, "version")?;
    if version != FALLBACKS_VERSION {
        return Err(AnswerError::UnknownValue {
            field: "version",
            value: version.to_string(),
        });
    }
    let mut profiles = BTreeMap::new();
    for (name, agents) in required_object(object, "profiles")? {
        let agents = as_object(agents, "profiles")?;
        let mut chains = BTreeMap::new();
        for (agent, chain) in agents {
            let chain = match chain {
                Value::Array(steps) => steps,
                _ => return Err(AnswerError::WrongType { field: "profiles" }),
            };
            let mut parsed = Vec::with_capacity(chain.len());
            for step in chain {
                let step = as_object(step, "profiles")?;
                parsed.push(FallbackStep {
                    model: required_str(step, "model")?,
                    thinking: optional_str(step, "thinking")?,
                });
            }
            chains.insert(agent.clone(), parsed);
        }
        profiles.insert(name.clone(), chains);
    }
    Ok(FallbacksDocument { profiles })
}

fn parse_pins(object: &serde_json::Map<String, Value>) -> Result<PinsValue, AnswerError> {
    let mut invalid = Vec::new();
    for layer in optional_array(object, "invalid")? {
        invalid.push(parse_pin_layer(as_object(layer, "invalid")?)?);
    }
    let mut stale = Vec::new();
    for layer in optional_array(object, "stale")? {
        stale.push(parse_pin_layer(as_object(layer, "stale")?)?);
    }
    Ok(PinsValue {
        resolved: optional_bool(object, "resolved")?.unwrap_or(false),
        winner: match optional_object(object, "winner")? {
            Some(winner) => Some(parse_pin_layer(winner)?),
            None => None,
        },
        invalid,
        stale,
        reason: optional_str(object, "reason")?,
    })
}

fn parse_pin_layer(object: &serde_json::Map<String, Value>) -> Result<PinLayer, AnswerError> {
    Ok(PinLayer {
        source: parse_pin_source(&required_str(object, "source")?)?,
        profile: optional_str(object, "profile")?,
    })
}

fn parse_store(value: &str) -> Result<StoreId, AnswerError> {
    StoreId::parse(value).ok_or_else(|| AnswerError::UnknownValue {
        field: "store",
        value: value.to_owned(),
    })
}

fn parse_scope(value: &str) -> Result<StoreScope, AnswerError> {
    StoreScope::parse(value).ok_or_else(|| AnswerError::UnknownValue {
        field: "scope",
        value: value.to_owned(),
    })
}

fn parse_store_revision(value: &str) -> Result<StoreRevision, AnswerError> {
    StoreRevision::parse(value).ok_or_else(|| AnswerError::UnknownValue {
        field: "revision",
        value: value.to_owned(),
    })
}

fn parse_write_target(value: &str) -> Result<WriteTargetName, AnswerError> {
    WriteTargetName::parse(value).ok_or_else(|| AnswerError::UnknownValue {
        field: "target",
        value: value.to_owned(),
    })
}

fn parse_write_revision(value: &str) -> Result<WriteRevision, AnswerError> {
    WriteRevision::parse(value).ok_or_else(|| AnswerError::UnknownValue {
        field: "writeRevision",
        value: value.to_owned(),
    })
}

fn parse_status(value: &str) -> Result<LayerStatus, AnswerError> {
    LayerStatus::parse(value).ok_or_else(|| AnswerError::UnknownValue {
        field: "status",
        value: value.to_owned(),
    })
}

fn parse_layer_source(value: &str) -> Result<LayerSource, AnswerError> {
    LayerSource::parse(value).ok_or_else(|| AnswerError::UnknownValue {
        field: "source",
        value: value.to_owned(),
    })
}

fn parse_pin_source(value: &str) -> Result<PinSource, AnswerError> {
    PinSource::parse(value).ok_or_else(|| AnswerError::UnknownValue {
        field: "source",
        value: value.to_owned(),
    })
}

fn parse_orchestrator_mode(value: &str) -> Result<OrchestratorMode, AnswerError> {
    OrchestratorMode::parse(value).ok_or_else(|| AnswerError::UnknownValue {
        field: "orchestratorMode",
        value: value.to_owned(),
    })
}
