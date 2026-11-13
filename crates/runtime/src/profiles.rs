//! The agent profiles gentle-pi keeps in `profiles.json` (0.1.12, S1, T2): the
//! read-only view the Settings dialog draws, the folder the file lives in, and
//! the fallback chains of its sibling `profile-fallbacks.json` (L13, option a).
//!
//! Nothing here writes: gentle-desk never edits a gentle-pi file, so this is a
//! parser and a path rule and nothing else. Both are pure, so the shape of the
//! envelope is pinned by tests without a file or a shell.
//!
//! The envelope is strict on purpose (`wrong kind or version shows nothing
//! rather than a wrong view`): a document another tool owns, or a version this
//! reader does not know, is a refusal, never a partial read. The chains follow
//! the same rule, and the same one gentle-pi applies to them: everything the
//! shell would refuse is refused whole here too (lib/model-fallback-store.ts:
//! 52-96), so the pane can only ever draw a chain gentle-pi would run.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use gentle_backend::chat_host::OrchestratorMode;
use serde::Deserialize;

/// The `kind` every agent-model-profiles document carries.
pub const KIND: &str = "gentle-pi.agent_model_profiles";
/// The only envelope version this reader understands.
pub const VERSION: u64 = 1;
/// The reserved role names the orchestrator route.
pub const ORCHESTRATOR_ROLE: &str = "orchestrator";
/// The `kind` every profile-fallback-chains document carries
/// (lib/model-fallback-store.ts:9).
pub const FALLBACKS_KIND: &str = "gentle-pi.profile_model_fallbacks";
/// The only chain-envelope version gentle-pi writes (lib/model-fallback-store.ts:10).
pub const FALLBACKS_VERSION: u64 = 1;
/// The agent key whose chain every agent of a profile without one takes
/// (lib/model-fallback-store.ts:12).
pub const DEFAULT_CHAIN_KEY: &str = "*";
/// The longest chain gentle-pi accepts (lib/model-fallback-store.ts:13).
pub const MAX_CHAIN_LENGTH: usize = 8;
/// The effort levels a chain step may name (lib/model-routing-authority.ts:4-12).
pub const THINKING_LEVELS: [&str; 7] = ["off", "minimal", "low", "medium", "high", "xhigh", "max"];
/// The name of the profiles file inside the config home.
pub const PROFILES_FILE: &str = "profiles.json";
/// The name of the chain file, which lives beside the profiles file: gentle-pi
/// keeps them apart so an older reader of `profiles.json` still reads it
/// unchanged (lib/model-fallback-store.ts:6-8,33-35).
pub const FALLBACKS_FILE: &str = "profile-fallbacks.json";

/// One profile as the view needs it: its roles and, when the file records one,
/// the workflow the profile implies.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Profile {
    /// role -> its model and effort, in the file's own order (role names sort).
    pub roles: BTreeMap<String, RoleSetting>,
    /// The orchestrator mode this profile prefers, from the sibling `workflow`
    /// map of the envelope. `None` for a profile that names none.
    pub workflow: Option<OrchestratorMode>,
    /// agent name (or `*`) -> the ordered fallbacks it may move to, from the
    /// sibling `profile-fallbacks.json`. Empty for a profile that file does not
    /// name, or when there is no chain file at all.
    pub fallbacks: BTreeMap<String, Vec<FallbackStep>>,
}

impl Profile {
    /// The reserved orchestrator route, the one role the summary line draws.
    pub fn orchestrator(&self) -> Option<&RoleSetting> {
        self.roles.get(ORCHESTRATOR_ROLE)
    }

    /// The chain for `agent`: its own, else the profile's `*`, else none - the
    /// rule gentle-pi falls back by (lib/model-fallback-store.ts:102-107).
    pub fn chain_for(&self, agent: &str) -> &[FallbackStep] {
        self.fallbacks
            .get(agent)
            .or_else(|| self.fallbacks.get(DEFAULT_CHAIN_KEY))
            .map_or(&[], Vec::as_slice)
    }

    /// The chain the profile's summary row draws: the orchestrator's own, or the
    /// profile's default one. Empty when the profile records no chain.
    pub fn fallback(&self) -> &[FallbackStep] {
        self.chain_for(ORCHESTRATOR_ROLE)
    }
}

/// One step of a chain: the model, and the effort gentle-pi switches to with it
/// when the step names one (lib/model-fallback-store.ts:15-18).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackStep {
    pub model: String,
    pub thinking: Option<String>,
}

/// The chains of one `profile-fallbacks.json`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Fallbacks {
    /// profile name -> agent name (or `*`) -> its ordered chain, sorted by name.
    pub profiles: BTreeMap<String, BTreeMap<String, Vec<FallbackStep>>>,
}

impl Fallbacks {
    /// The chain for `agent` under `profile`, resolved with the same rule
    /// [`Profile::chain_for`] applies: its own, else the profile's `*`, else none.
    pub fn chain_for(&self, profile: &str, agent: &str) -> &[FallbackStep] {
        self.profiles
            .get(profile)
            .and_then(|chains| chains.get(agent).or_else(|| chains.get(DEFAULT_CHAIN_KEY)))
            .map_or(&[], Vec::as_slice)
    }
}

/// Reads one `profile-fallbacks.json`, refusing everything gentle-pi refuses
/// (lib/model-fallback-store.ts:52-80): another kind or version, a missing
/// `profiles` map, an unknown key, a profile or agent name that is not a safe
/// key, a chain of more than [`MAX_CHAIN_LENGTH`] steps, a model twice in one
/// chain, a model outside the safe pattern, or an effort gentle-pi does not know.
/// A chain with no steps names no fallback at all and is dropped, as gentle-pi
/// drops it.
pub fn parse_fallbacks(text: &str) -> Result<Fallbacks, String> {
    let envelope: FallbacksEnvelope = serde_json::from_str(text)
        .map_err(|error| format!("{FALLBACKS_FILE} is not readable: {error}"))?;
    if envelope.kind != FALLBACKS_KIND {
        return Err(format!(
            "{FALLBACKS_FILE} is not a {FALLBACKS_KIND} document, it says {:?}",
            envelope.kind
        ));
    }
    if envelope.version != FALLBACKS_VERSION {
        return Err(format!(
            "{FALLBACKS_FILE} is version {}, this view reads version {FALLBACKS_VERSION}",
            envelope.version
        ));
    }
    let mut profiles: BTreeMap<String, BTreeMap<String, Vec<FallbackStep>>> = BTreeMap::new();
    for (name, agents) in envelope.profiles {
        if !is_valid_profile_name(&name) {
            return Err(format!(
                "{FALLBACKS_FILE} names a profile no reader accepts: {name:?}"
            ));
        }
        let mut chains: BTreeMap<String, Vec<FallbackStep>> = BTreeMap::new();
        for (agent, steps) in agents {
            if agent != DEFAULT_CHAIN_KEY && !is_safe_agent_name(&agent) {
                return Err(format!(
                    r#"{FALLBACKS_FILE}: {name:?} names the agent {agent:?}"#
                ));
            }
            let chain = chain_of(&steps)
                .ok_or_else(|| format!(r#"{FALLBACKS_FILE}: {name:?}/{agent:?} is not a chain"#))?;
            if !chain.is_empty() {
                chains.insert(agent, chain);
            }
        }
        if !chains.is_empty() {
            profiles.insert(name, chains);
        }
    }
    Ok(Fallbacks { profiles })
}

/// One chain, or `None` when gentle-pi would refuse it.
fn chain_of(steps: &[StepEntry]) -> Option<Vec<FallbackStep>> {
    if steps.len() > MAX_CHAIN_LENGTH {
        return None;
    }
    let mut chain = Vec::with_capacity(steps.len());
    for step in steps {
        if step.model.is_empty() || !is_safe_model_id(&step.model) {
            return None;
        }
        if let Some(thinking) = &step.thinking {
            if !is_thinking_level(thinking) {
                return None;
            }
        }
        chain.push(FallbackStep {
            model: step.model.clone(),
            thinking: step.thinking.clone(),
        });
    }
    // The same model twice in one chain is not a fallback.
    let models: std::collections::BTreeSet<&str> =
        chain.iter().map(|step| step.model.as_str()).collect();
    (models.len() == chain.len()).then_some(chain)
}

/// Attaches the chains of a parsed chain file to the profiles it names. A
/// profile the file does not name keeps no chains, which is the truth: gentle-pi
/// falls back to nothing for it either.
pub fn attach_fallbacks(profiles: &mut Profiles, fallbacks: &Fallbacks) {
    for (name, profile) in profiles.profiles.iter_mut() {
        profile.fallbacks = fallbacks.profiles.get(name).cloned().unwrap_or_default();
    }
}

/// The profile-name policy of gentle-pi's own reader (lib/agent-profiles.ts:72-75,
/// 111-116): a slug of up to 64 characters, minus the three names every record
/// already has.
fn is_valid_profile_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    let head = bytes
        .first()
        .is_some_and(|first| first.is_ascii_alphanumeric());
    let tail = bytes
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'));
    head && tail && name.len() <= 64 && !matches!(name, "__proto__" | "constructor" | "prototype")
}

/// The agent-name policy (lib/model-routing-authority.ts:36-42).
fn is_safe_agent_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"._:@/+%-".contains(&byte))
}

/// The model-id policy (lib/model-routing-authority.ts:35): the characters a
/// `provider/model` reference may carry.
fn is_safe_model_id(model: &str) -> bool {
    model
        .bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || b"._~:@/+%-".contains(&byte))
}

/// The effort levels gentle-pi writes (lib/model-routing-authority.ts:4-14).
fn is_thinking_level(level: &str) -> bool {
    THINKING_LEVELS.contains(&level)
}

/// One role's settings: what it runs and how hard it thinks. Every field is
/// optional because the file allows a partial row.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RoleSetting {
    pub model: Option<String>,
    pub thinking: Option<String>,
    pub tier: Option<String>,
}

/// The whole document as the view reads it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Profiles {
    /// The active profile's name, when the file names one.
    pub active: Option<String>,
    /// name -> profile, sorted by name.
    pub profiles: BTreeMap<String, Profile>,
}

/// Reads one `profiles.json`. A document of another kind or version is a
/// refusal with the reason, so the view can say it cannot read the file
/// instead of drawing a wrong one.
pub fn parse(text: &str) -> Result<Profiles, String> {
    let envelope: Envelope = serde_json::from_str(text)
        .map_err(|error| format!("profiles.json is not readable: {error}"))?;
    if envelope.kind != KIND {
        return Err(format!(
            "profiles.json is not a {KIND} document, it says {:?}",
            envelope.kind
        ));
    }
    if envelope.version != VERSION {
        return Err(format!(
            "profiles.json is version {}, this view reads version {VERSION}",
            envelope.version
        ));
    }
    let profiles = envelope
        .profiles
        .into_iter()
        .map(|(name, roles)| {
            let workflow = envelope
                .workflow
                .get(&name)
                .and_then(|workflow| workflow.orchestrator_mode.as_deref())
                .and_then(OrchestratorMode::parse);
            let roles = roles
                .into_iter()
                .map(|(role, entry)| {
                    (
                        role,
                        RoleSetting {
                            model: entry.model,
                            thinking: entry.thinking,
                            tier: entry.tier,
                        },
                    )
                })
                .collect();
            (
                name,
                Profile {
                    roles,
                    workflow,
                    fallbacks: BTreeMap::new(),
                },
            )
        })
        .collect();
    Ok(Profiles {
        active: envelope.active,
        profiles,
    })
}

/// What the config-home rule reads. `None` for a variable that is unset or
/// empty: both mean "not named".
#[derive(Debug, Clone, Default)]
pub struct ConfigHomeEnv<'a> {
    /// The folder of a bundled gentle-pi install (`<prefix>/config/gentle-ai`
    /// is its config home). The desk resolves no bundled prefix of its own, so
    /// this stays `None` unless a caller knows one.
    pub bundled_prefix: Option<&'a Path>,
    /// `GENTLE_PI_CONFIG_HOME`, which names the config home itself.
    pub gentle_pi_config_home: Option<&'a Path>,
    /// `GENTLE_SHELL_CONFIG_HOME`, the launcher's marker: a config home only
    /// when it is an absolute path (a relative one is not a location).
    pub gentle_shell_config_home: Option<&'a Path>,
    /// The pi home of the agent directory (`~/.pi`, `~/.gentle-shell`, or the
    /// parent of a `--home` folder), the last layer's base.
    pub pi_home: Option<&'a Path>,
}

/// gentle-pi's config home, the rule every one of its readers shares: a bundled
/// prefix's folder, then an explicit `GENTLE_PI_CONFIG_HOME`, then the
/// launcher's marker when it is absolute, then `<pi home>/gentle-ai`.
pub fn config_home(env: &ConfigHomeEnv<'_>) -> Option<PathBuf> {
    if let Some(prefix) = env.bundled_prefix {
        return Some(prefix.join("config").join("gentle-ai"));
    }
    if let Some(dir) = env.gentle_pi_config_home {
        return Some(dir.to_path_buf());
    }
    if let Some(marker) = env.gentle_shell_config_home {
        if marker.is_absolute() {
            return Some(marker.to_path_buf());
        }
    }
    Some(env.pi_home?.join("gentle-ai"))
}

/// The file this view reads: `<config home>/profiles.json`.
pub fn profiles_path(env: &ConfigHomeEnv<'_>) -> Option<PathBuf> {
    config_home(env).map(|dir| dir.join(PROFILES_FILE))
}

/// The chain file, which is the profiles file's SIBLING: gentle-pi resolves both
/// from the same config home (lib/agent-profiles.ts:685-687 and
/// lib/model-fallback-store.ts:33-35), so the desk must too.
pub fn fallbacks_path(env: &ConfigHomeEnv<'_>) -> Option<PathBuf> {
    config_home(env).map(|dir| dir.join(FALLBACKS_FILE))
}

/// The envelope as it is written, with everything the view ignores left out.
#[derive(Debug, Deserialize)]
struct Envelope {
    kind: String,
    version: u64,
    #[serde(default)]
    active: Option<String>,
    #[serde(default)]
    profiles: BTreeMap<String, BTreeMap<String, RoleEntry>>,
    #[serde(default)]
    workflow: BTreeMap<String, WorkflowEntry>,
}

#[derive(Debug, Deserialize)]
struct RoleEntry {
    #[serde(default)]
    model: Option<String>,
    #[serde(default)]
    thinking: Option<String>,
    #[serde(default)]
    tier: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorkflowEntry {
    #[serde(rename = "orchestratorMode", default)]
    orchestrator_mode: Option<String>,
}

/// The chain envelope as it is written. Unknown keys are refused, because
/// gentle-pi's own normalizer refuses them (lib/model-fallback-store.ts:58-60).
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FallbacksEnvelope {
    kind: String,
    version: u64,
    profiles: BTreeMap<String, BTreeMap<String, Vec<StepEntry>>>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StepEntry {
    model: String,
    #[serde(default)]
    thinking: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A document with the real envelope: two roles and a workflow.
    fn document() -> String {
        r#"{
            "kind": "gentle-pi.agent_model_profiles",
            "version": 1,
            "active": "Claude-CommandCode",
            "profiles": {
                "Claude-CommandCode": {
                    "orchestrator": {"model": "claude-bridge/claude-sonnet-5-5", "thinking": "medium"},
                    "review-refuter": {"model": "commandcode/z-ai/glm-5.3-flash", "thinking": "high", "tier": "high"}
                },
                "Plain": {
                    "orchestrator": {"model": "commandcode/deepseek/deepseek-v4.1-flash"}
                }
            },
            "workflow": {
                "Claude-CommandCode": {"orchestratorMode": "full-delegation"}
            }
        }"#
        .to_owned()
    }

    #[test]
    fn the_active_profile_and_its_orchestrator_route_are_read() {
        let profiles = parse(&document()).expect("the real envelope reads");
        assert_eq!(profiles.active.as_deref(), Some("Claude-CommandCode"));

        let active = &profiles.profiles["Claude-CommandCode"];
        let orchestrator = active.orchestrator().expect("the reserved role is there");
        assert_eq!(
            orchestrator.model.as_deref(),
            Some("claude-bridge/claude-sonnet-5-5")
        );
        assert_eq!(orchestrator.thinking.as_deref(), Some("medium"));
        assert_eq!(orchestrator.tier, None);
        assert_eq!(active.roles["review-refuter"].tier.as_deref(), Some("high"));
        assert_eq!(active.workflow, Some(OrchestratorMode::FullDelegation));
    }

    #[test]
    fn a_profile_with_no_workflow_names_none() {
        let profiles = parse(&document()).expect("the document reads");
        let plain = &profiles.profiles["Plain"];
        assert_eq!(plain.workflow, None);
        assert_eq!(
            plain
                .orchestrator()
                .and_then(|role| role.thinking.as_deref()),
            None,
            "a partial row stays partial"
        );
    }

    #[test]
    fn a_document_of_another_kind_is_refused() {
        let wrong = document().replace(KIND, "someone.else.profiles");
        let error = parse(&wrong).expect_err("another kind is not this view's document");
        assert!(
            error.contains("not a gentle-pi.agent_model_profiles document"),
            "{error}"
        );
        assert!(error.contains("someone.else.profiles"), "{error}");
    }

    #[test]
    fn a_document_of_another_version_is_refused() {
        let wrong = document().replace("\"version\": 1", "\"version\": 2");
        let error = parse(&wrong).expect_err("a version this reader does not know is refused");
        assert!(error.contains("version 2"), "{error}");
    }

    #[test]
    fn a_document_with_no_envelope_or_no_json_is_refused() {
        assert!(parse("").is_err());
        assert!(parse("null").is_err());
        assert!(parse(r#"{"kind":"gentle-pi.agent_model_profiles"}"#).is_err());
        // A version that is text, not a number, is not this reader's either.
        assert!(parse(r#"{"kind":"gentle-pi.agent_model_profiles","version":"1"}"#).is_err());
    }

    #[test]
    fn an_account_with_no_profiles_is_an_empty_document_not_an_error() {
        let profiles = parse(r#"{"kind":"gentle-pi.agent_model_profiles","version":1}"#)
            .expect("the envelope is enough");
        assert_eq!(profiles.active, None);
        assert!(profiles.profiles.is_empty());
    }

    fn env<'a>(
        prefix: Option<&'a Path>,
        pi_home: Option<&'a Path>,
        pi_config: Option<&'a Path>,
        marker: Option<&'a Path>,
    ) -> ConfigHomeEnv<'a> {
        ConfigHomeEnv {
            bundled_prefix: prefix,
            gentle_pi_config_home: pi_config,
            gentle_shell_config_home: marker,
            pi_home,
        }
    }

    #[test]
    fn the_config_home_layers_in_the_shells_own_order() {
        let prefix = Path::new("C:/npm/node_modules/gentle-pi");
        let pi_home = Path::new("C:/Users/u/.pi");
        let pi_config = Path::new("C:/custom");
        let marker = Path::new("C:/Users/u/.gentle-shell/gentle-ai");

        // A bundled prefix wins over everything.
        assert_eq!(
            config_home(&env(
                Some(prefix),
                Some(pi_home),
                Some(pi_config),
                Some(marker)
            )),
            Some(prefix.join("config").join("gentle-ai"))
        );
        // Then the explicit variable, which names the config home itself.
        assert_eq!(
            config_home(&env(None, Some(pi_home), Some(pi_config), Some(marker))),
            Some(pi_config.to_path_buf())
        );
        // Then the launcher's marker.
        assert_eq!(
            config_home(&env(None, Some(pi_home), None, Some(marker))),
            Some(marker.to_path_buf())
        );
        // Then the pi home of the agent directory.
        assert_eq!(
            config_home(&env(None, Some(pi_home), None, None)),
            Some(pi_home.join("gentle-ai"))
        );
        // Nothing to go on at all: no home, no path.
        assert_eq!(config_home(&env(None, None, None, None)), None);
    }

    #[test]
    fn a_relative_marker_is_not_a_location() {
        let pi_home = Path::new("C:/Users/u/.pi");
        let relative = Path::new("gentle-ai");
        assert_eq!(
            config_home(&env(None, Some(pi_home), None, Some(relative))),
            Some(pi_home.join("gentle-ai")),
            "a relative marker falls through to the pi home"
        );
    }

    #[test]
    fn the_profile_file_is_the_config_home_plus_its_name() {
        let pi_home = Path::new("C:/Users/u/.pi");
        assert_eq!(
            profiles_path(&env(None, Some(pi_home), None, None)),
            Some(pi_home.join("gentle-ai").join("profiles.json"))
        );
    }

    /// The document gentle-pi writes, quoted from its own tests
    /// (tests/gentle-agents.test.ts:8274): the profile's default chain.
    fn fallbacks_document() -> String {
        r#"{
            "kind": "gentle-pi.profile_model_fallbacks",
            "version": 1,
            "profiles": {
                "home": {
                    "*": [{"model": "fby/second-model", "thinking": "low"}]
                }
            }
        }"#
        .to_owned()
    }

    /// The same document with one agent chain beside the default one, as the
    /// orchestrator's own chain is written (tests/gentle-agents.test.ts:8303).
    fn fallbacks_with_agent_chain() -> String {
        fallbacks_document().replace(
            r#""*": [{"model": "fby/second-model", "thinking": "low"}]"#,
            r#""*": [{"model": "fby/second-model", "thinking": "low"}],
                    "orchestrator": [{"model": "orx/backup", "thinking": "high"}]"#,
        )
    }

    #[test]
    fn the_real_fallbacks_document_reads_and_names_a_chain() {
        let fallbacks = parse_fallbacks(&fallbacks_document()).expect("the real document reads");
        let chain = fallbacks.chain_for("home", "worker");
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].model, "fby/second-model");
        assert_eq!(chain[0].thinking.as_deref(), Some("low"));
        assert!(
            fallbacks.chain_for("another-profile", "worker").is_empty(),
            "a profile the file does not name has no chain"
        );
    }

    #[test]
    fn an_agents_own_chain_wins_over_the_profiles_default() {
        let fallbacks = parse_fallbacks(&fallbacks_with_agent_chain()).expect("the document reads");
        assert_eq!(
            fallbacks.chain_for("home", "orchestrator")[0].model,
            "orx/backup",
            "the agent's own chain is the first one"
        );
        assert_eq!(
            fallbacks.chain_for("home", "worker")[0].model,
            "fby/second-model",
            "every other agent takes the profile's default chain"
        );
    }

    #[test]
    fn a_fallbacks_document_of_another_kind_or_version_is_refused() {
        let wrong_kind = fallbacks_document().replace(FALLBACKS_KIND, "someone.else.fallbacks");
        let error = parse_fallbacks(&wrong_kind).expect_err("another kind is not this document");
        assert!(
            error.contains("not a gentle-pi.profile_model_fallbacks document"),
            "{error}"
        );
        assert!(error.contains("someone.else.fallbacks"), "{error}");

        let wrong_version = fallbacks_document().replace("\"version\": 1", "\"version\": 2");
        let error =
            parse_fallbacks(&wrong_version).expect_err("a version this reader does not know");
        assert!(error.contains("version 2"), "{error}");
    }

    #[test]
    fn a_fallbacks_document_gentle_pi_would_refuse_is_refused_whole() {
        // The rules are gentle-pi's own (lib/model-fallback-store.ts:52-80): the
        // envelope has exactly three keys, a chain is at most eight steps of
        // one model each, and every model and effort is one it recognises.
        let too_long: Vec<String> = (0..=MAX_CHAIN_LENGTH)
            .map(|index| format!(r#"{{"model":"prov/m{index}"}}"#))
            .collect();
        let chain = |body: &str| {
            format!(
                r#"{{"kind":"{FALLBACKS_KIND}","version":1,"profiles":{{"home":{{"*":[{body}]}}}}}}"#
            )
        };
        let refused = [
            // A key gentle-pi does not write.
            format!(r#"{{"kind":"{FALLBACKS_KIND}","version":1,"profiles":{{}},"extra":true}}"#),
            // No profiles map at all.
            format!(r#"{{"kind":"{FALLBACKS_KIND}","version":1}}"#),
            // More steps than a chain may hold.
            chain(&too_long.join(",")),
            // The same model twice in one chain.
            chain(r#"{"model":"prov/one"},{"model":"prov/one"}"#),
            // An effort gentle-pi does not know.
            chain(r#"{"model":"prov/one","thinking":"turbo"}"#),
            // A step with a key gentle-pi does not write.
            chain(r#"{"model":"prov/one","tier":"high"}"#),
            // A model id outside the safe pattern, and the empty one.
            chain(r#"{"model":"prov/one two"}"#),
            chain(r#"{"model":""}"#),
            // A profile name and an agent name that are not safe keys.
            format!(
                r#"{{"kind":"{FALLBACKS_KIND}","version":1,"profiles":{{"bad name":{{"*":[{{"model":"prov/one"}}]}}}}}}"#
            ),
            format!(
                r#"{{"kind":"{FALLBACKS_KIND}","version":1,"profiles":{{"home":{{"bad agent":[{{"model":"prov/one"}}]}}}}}}"#
            ),
            // Not JSON, and not a document at all.
            "not json".to_owned(),
            String::new(),
        ];
        for document in refused {
            assert!(
                parse_fallbacks(&document).is_err(),
                "gentle-pi refuses this, so the desk must too: {document}"
            );
        }
    }

    #[test]
    fn an_empty_chain_names_no_fallback_at_all() {
        let document = format!(
            r#"{{"kind":"{FALLBACKS_KIND}","version":1,"profiles":{{"home":{{"*":[]}}}}}}"#
        );
        let fallbacks = parse_fallbacks(&document).expect("an empty chain is a valid document");
        assert!(
            fallbacks.profiles.is_empty(),
            "a profile with no step left is dropped, exactly as gentle-pi drops it"
        );
        assert!(fallbacks.chain_for("home", "worker").is_empty());
    }

    #[test]
    fn the_fallbacks_file_is_a_sibling_of_the_profile_file() {
        for env in [
            env(None, Some(Path::new("C:/Users/u/.pi")), None, None),
            env(
                None,
                Some(Path::new("C:/Users/u/.pi")),
                Some(Path::new("C:/custom")),
                None,
            ),
            env(
                None,
                Some(Path::new("C:/Users/u/.pi")),
                None,
                Some(Path::new("C:/Users/u/.gentle-shell/gentle-ai")),
            ),
        ] {
            let profiles = profiles_path(&env).expect("a config home resolves");
            let fallbacks = fallbacks_path(&env).expect("a config home resolves");
            assert_eq!(
                fallbacks.parent(),
                profiles.parent(),
                "both files come from the same config home"
            );
            assert_eq!(
                fallbacks.file_name().and_then(|name| name.to_str()),
                Some("profile-fallbacks.json")
            );
        }
        assert_eq!(fallbacks_path(&env(None, None, None, None)), None);
    }

    #[test]
    fn the_chains_attach_to_the_profile_that_is_named_and_no_other() {
        let mut profiles = parse(&document()).expect("the profiles document reads");
        let text = fallbacks_with_agent_chain().replace("\"home\"", "\"Plain\"");
        let fallbacks = parse_fallbacks(&text).expect("the fallbacks document reads");

        attach_fallbacks(&mut profiles, &fallbacks);

        assert_eq!(
            profiles.profiles["Plain"].fallback()[0].model,
            "orx/backup",
            "the summary row is the orchestrator's chain"
        );
        assert_eq!(profiles.profiles["Plain"].fallbacks.len(), 2);
        assert!(
            profiles.profiles["Claude-CommandCode"]
                .fallback()
                .is_empty(),
            "a profile the fallbacks file does not name keeps none"
        );
    }
}
