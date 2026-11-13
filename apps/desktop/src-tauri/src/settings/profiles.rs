//! The Profiles pane's slice: `profiles.json` read through the seam into the
//! contract's view (0.1.12, T2), and the fallback chains of its sibling
//! `profile-fallbacks.json` (L13, option a).
//!
//! The files' rules (the strict envelope, the active profile, the reserved
//! orchestrator role, the workflow map, the chains) live in
//! [`gentle_runtime::profiles`]; this is the mapping onto what the pane draws,
//! plus the one thing the file cannot say: the orchestrator mode the desk's next
//! chats start with, which is the session's switch and never a stored value.
//!
//! The FALLBACK pair has no field of its own: the pane draws it from a role named
//! `fallback` when the active profile has one, which is the shape the pane
//! already handles, so the contract does not change. The row is the desk's
//! reading of the sibling file, never a role gentle-pi wrote in `profiles.json`:
//! a real role of that name is replaced by the chain when the chain file records
//! one, so the pane's keyed list always holds exactly one such row.

use std::path::Path;

use gentle_runtime::profiles::ORCHESTRATOR_ROLE;
use gentle_runtime::profiles::{self as rule, FallbackStep, Profile};

use super::FileRead;
use crate::contract::{
    OrchestratorModeView, ProfileRoleView, ProfileView, ProfilesStatusView, ProfilesView,
};

/// The role name the pane draws the profile's fallback chain under. It is the
/// desk's own row, and the pane's copy (`profiles-copy.ts`) labels it.
const FALLBACK_ROLE: &str = "fallback";

/// Reads `path` through `read`, its sibling `fallbacks` through the same seam,
/// and answers the pane's slice.
///
/// A file that is not there is a ready pane with no profiles (nothing was ever
/// created); a file that cannot be read, or is not this reader's document, is
/// `Failed` with the reason, so the pane says it cannot read instead of drawing
/// a wrong view.
///
/// The chain file is never a reason to fail the pane: it is the sibling's, so a
/// missing, unreadable or refused document simply leaves the FALLBACK row
/// undrawn - which is what gentle-pi does with it too.
pub fn view(
    read: &FileRead,
    path: Option<&Path>,
    fallbacks: Option<&Path>,
    mode: OrchestratorModeView,
) -> ProfilesView {
    let mut view = ProfilesView {
        status: ProfilesStatusView::Unknown,
        active: None,
        profiles: Vec::new(),
        mode,
    };
    let Some(path) = path else {
        view.status = ProfilesStatusView::Failed(
            "gentle-pi's config folder could not be found on this machine".to_owned(),
        );
        return view;
    };
    let text = match read(path) {
        Ok(Some(text)) => text,
        Ok(None) => {
            view.status = ProfilesStatusView::Ready;
            return view;
        }
        Err(reason) => {
            view.status = ProfilesStatusView::Failed(reason);
            return view;
        }
    };
    match rule::parse(&text) {
        Ok(mut profiles) => {
            rule::attach_fallbacks(&mut profiles, &chains(read, fallbacks));
            view_of(&profiles, mode)
        }
        Err(reason) => ProfilesView {
            status: ProfilesStatusView::Failed(reason),
            mode,
            ..ProfilesView::default()
        },
    }
}

/// One read of the sibling chain file. Everything that is not a valid chain
/// document is an empty set of chains: no file, a file that cannot be read, and
/// a document of another kind, version or step shape. The desk never invents a
/// fallback to fill the row.
fn chains(read: &FileRead, path: Option<&Path>) -> rule::Fallbacks {
    let Some(path) = path else {
        return rule::Fallbacks::default();
    };
    match read(path) {
        Ok(Some(text)) => rule::parse_fallbacks(&text).unwrap_or_default(),
        _ => rule::Fallbacks::default(),
    }
}

/// The pane's slice of one parsed document.
fn view_of(profiles: &rule::Profiles, mode: OrchestratorModeView) -> ProfilesView {
    let active = profiles.active.clone();
    ProfilesView {
        status: ProfilesStatusView::Ready,
        active: active.clone(),
        profiles: profiles
            .profiles
            .iter()
            .map(|(name, profile)| profile_view(name, profile, active.as_deref()))
            .collect(),
        mode,
    }
}

/// One profile: its name, whether it is the active one, the reserved
/// orchestrator role as the summary line, and every other role in the file's
/// order - plus the fallback chain the sibling file records for it, as one more
/// row when there is a chain.
fn profile_view(name: &str, profile: &Profile, active: Option<&str>) -> ProfileView {
    let mut roles: Vec<ProfileRoleView> = profile
        .roles
        .iter()
        .filter(|(role, _)| role.as_str() != ORCHESTRATOR_ROLE)
        .map(|(role, setting)| role_view(role, setting))
        .collect();
    let chain = profile.fallback();
    if !chain.is_empty() {
        let row = fallback_view(chain);
        match roles
            .iter_mut()
            .find(|existing| existing.role == FALLBACK_ROLE)
        {
            Some(existing) => *existing = row,
            None => roles.push(row),
        }
    }
    ProfileView {
        name: name.to_owned(),
        active: active == Some(name),
        orchestrator: profile
            .orchestrator()
            .map(|role| role_view(ORCHESTRATOR_ROLE, role)),
        roles,
        mode: profile.workflow.map(mode_view),
    }
}

/// The FALLBACK pair of a chain: the models in the order gentle-pi would move
/// through them, and the effort only when every step asks for the same one (the
/// pair has a single effort column, and a chain that changes effort per step
/// cannot say that in one cell). A step carries no tier, so the row never draws
/// one.
fn fallback_view(chain: &[FallbackStep]) -> ProfileRoleView {
    let model = chain
        .iter()
        .map(|step| step.model.as_str())
        .collect::<Vec<_>>()
        .join(" \u{2192} ");
    let thinking = chain
        .first()
        .and_then(|step| step.thinking.clone())
        .filter(|level| {
            chain
                .iter()
                .all(|step| step.thinking.as_deref() == Some(level.as_str()))
        });
    ProfileRoleView {
        role: FALLBACK_ROLE.to_owned(),
        model: Some(model),
        thinking,
        tier: None,
    }
}

fn role_view(role: &str, setting: &rule::RoleSetting) -> ProfileRoleView {
    ProfileRoleView {
        role: role.to_owned(),
        model: setting.model.clone(),
        thinking: setting.thinking.clone(),
        tier: setting.tier.clone(),
    }
}

/// The file's own name for a mode, as the contract names it.
pub fn mode_view(mode: gentle_backend::chat_host::OrchestratorMode) -> OrchestratorModeView {
    match mode {
        gentle_backend::chat_host::OrchestratorMode::Organic => OrchestratorModeView::Organic,
        gentle_backend::chat_host::OrchestratorMode::FullDelegation => {
            OrchestratorModeView::FullDelegation
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::sync::Arc;

    use super::*;

    const DOCUMENT: &str = r#"{
        "kind": "gentle-pi.agent_model_profiles",
        "version": 1,
        "active": "Mine",
        "profiles": {
            "Mine": {
                "orchestrator": {"model": "prov/big", "thinking": "medium"},
                "worker": {"model": "prov/small", "thinking": "high", "tier": "high"}
            }
        },
        "workflow": {"Mine": {"orchestratorMode": "organic"}}
    }"#;

    /// The sibling chain file gentle-pi writes, in its real envelope
    /// (lib/model-fallback-store.ts:20-31): two steps that share one effort.
    const FALLBACKS: &str = r#"{
        "kind": "gentle-pi.profile_model_fallbacks",
        "version": 1,
        "profiles": {
            "Mine": {
                "*": [
                    {"model": "prov/fb-one", "thinking": "low"},
                    {"model": "prov/fb-two", "thinking": "low"}
                ]
            }
        }
    }"#;

    /// The two files of one config home, as `SettingsPaths` resolves them.
    fn paths() -> (PathBuf, PathBuf) {
        let dir = PathBuf::from("C:/pi/gentle-ai");
        (
            dir.join("profiles.json"),
            dir.join("profile-fallbacks.json"),
        )
    }

    /// A reader of one config home: `profiles.json`, and its sibling
    /// `profile-fallbacks.json` when the machine has one.
    fn files(profiles: &str, fallbacks: Option<&str>) -> FileRead {
        let profiles = profiles.to_owned();
        let fallbacks = fallbacks.map(str::to_owned);
        Arc::new(move |path: &Path| {
            if path.file_name().and_then(|name| name.to_str()) == Some("profile-fallbacks.json") {
                return Ok(fallbacks.clone());
            }
            Ok(Some(profiles.clone()))
        })
    }

    /// The reader of the profiles file alone.
    fn reader(text: &str) -> FileRead {
        files(text, None)
    }

    /// A reader of a machine where neither file was ever written.
    fn no_file() -> FileRead {
        Arc::new(|_path: &Path| Ok(None))
    }

    #[test]
    fn the_active_profile_and_its_roles_reach_the_view() {
        let path = PathBuf::from("C:/pi/gentle-ai/profiles.json");

        let view = view(
            &reader(DOCUMENT),
            Some(&path),
            None,
            OrchestratorModeView::FullDelegation,
        );

        assert_eq!(view.status, ProfilesStatusView::Ready);
        assert_eq!(view.active.as_deref(), Some("Mine"));
        assert_eq!(
            view.mode,
            OrchestratorModeView::FullDelegation,
            "the pane's mode is the session's switch"
        );
        let profile = &view.profiles[0];
        assert!(profile.active);
        assert_eq!(
            profile
                .orchestrator
                .as_ref()
                .and_then(|role| role.model.as_deref()),
            Some("prov/big")
        );
        assert_eq!(
            profile.roles.len(),
            1,
            "the orchestrator is the summary line"
        );
        assert_eq!(profile.roles[0].role, "worker");
        assert_eq!(profile.roles[0].tier.as_deref(), Some("high"));
        assert_eq!(profile.mode, Some(OrchestratorModeView::Organic));
    }

    #[test]
    fn a_missing_file_is_a_ready_pane_with_no_profiles() {
        let view = view(
            &no_file(),
            Some(&PathBuf::from("C:/pi/gentle-ai/profiles.json")),
            Some(&PathBuf::from("C:/pi/gentle-ai/profile-fallbacks.json")),
            OrchestratorModeView::Organic,
        );
        assert_eq!(view.status, ProfilesStatusView::Ready);
        assert!(view.profiles.is_empty());
        assert_eq!(view.active, None);
    }

    #[test]
    fn a_document_of_another_kind_says_why_and_draws_nothing() {
        let wrong = DOCUMENT.replace("gentle-pi.agent_model_profiles", "other");
        let view = view(
            &reader(&wrong),
            Some(&PathBuf::from("p.json")),
            None,
            OrchestratorModeView::Organic,
        );
        match view.status {
            ProfilesStatusView::Failed(reason) => assert!(reason.contains("not a"), "{reason}"),
            other => panic!("a wrong document must say why, got {other:?}"),
        }
        assert!(view.profiles.is_empty(), "a wrong document shows nothing");
    }

    #[test]
    fn a_read_that_failed_says_why() {
        let failing: FileRead = Arc::new(|_path: &Path| Err("permission denied".to_owned()));
        let view = view(
            &failing,
            Some(&PathBuf::from("p.json")),
            None,
            OrchestratorModeView::Organic,
        );
        assert_eq!(
            view.status,
            ProfilesStatusView::Failed("permission denied".to_owned())
        );
    }

    #[test]
    fn no_config_folder_is_said_plainly() {
        let view = view(&reader(DOCUMENT), None, None, OrchestratorModeView::Organic);
        match view.status {
            ProfilesStatusView::Failed(reason) => {
                assert!(reason.contains("config folder"), "{reason}")
            }
            other => panic!("no folder must say so, got {other:?}"),
        }
    }

    /// The fallback row of a drawn view, when it has one.
    fn fallback(view: &ProfilesView) -> Option<&ProfileRoleView> {
        view.profiles[0]
            .roles
            .iter()
            .find(|role| role.role == "fallback")
    }

    #[test]
    fn the_fallback_row_is_the_chain_the_sibling_file_records() {
        let (profiles_path, fallbacks_path) = paths();

        let view = view(
            &files(DOCUMENT, Some(FALLBACKS)),
            Some(&profiles_path),
            Some(&fallbacks_path),
            OrchestratorModeView::Organic,
        );

        assert_eq!(view.status, ProfilesStatusView::Ready);
        let fallback = fallback(&view).expect("the chain is a row the pane draws");
        assert_eq!(
            fallback.model.as_deref(),
            Some("prov/fb-one \u{2192} prov/fb-two"),
            "the row names the chain in its own order"
        );
        assert_eq!(
            fallback.thinking.as_deref(),
            Some("low"),
            "the effort is drawn when every step asks for the same one"
        );
        assert_eq!(fallback.tier, None, "a fallback step carries no tier");
        assert_eq!(
            view.profiles[0].roles.len(),
            2,
            "the chain is one more row beside the profile's own roles"
        );
    }

    #[test]
    fn no_fallbacks_file_draws_no_fallback_row() {
        let (profiles_path, fallbacks_path) = paths();

        let view = view(
            &files(DOCUMENT, None),
            Some(&profiles_path),
            Some(&fallbacks_path),
            OrchestratorModeView::Organic,
        );

        assert_eq!(view.status, ProfilesStatusView::Ready);
        assert!(
            fallback(&view).is_none(),
            "a machine with no chain file gets no fallback row, never an invented one"
        );
    }

    #[test]
    fn a_fallbacks_document_of_another_kind_or_version_draws_no_row() {
        let (profiles_path, fallbacks_path) = paths();
        let refused = [
            FALLBACKS.replace(
                "gentle-pi.profile_model_fallbacks",
                "someone.else.fallbacks",
            ),
            FALLBACKS.replace("\"version\": 1", "\"version\": 2"),
            "{ not json".to_owned(),
        ];

        for document in refused {
            let view = view(
                &files(DOCUMENT, Some(&document)),
                Some(&profiles_path),
                Some(&fallbacks_path),
                OrchestratorModeView::Organic,
            );
            assert_eq!(
                view.status,
                ProfilesStatusView::Ready,
                "the chain file is the sibling's business, not a reason to fail the pane"
            );
            assert!(fallback(&view).is_none(), "no row for a bad chain file");
        }
    }

    #[test]
    fn the_summary_row_is_the_chain_of_the_orchestrator_agent() {
        let (profiles_path, fallbacks_path) = paths();
        let chains = r#"{
            "kind": "gentle-pi.profile_model_fallbacks",
            "version": 1,
            "profiles": {
                "Mine": {
                    "*": [{"model": "prov/fb-one"}],
                    "orchestrator": [{"model": "orx/backup", "thinking": "high"}]
                }
            }
        }"#;

        let view = view(
            &files(DOCUMENT, Some(chains)),
            Some(&profiles_path),
            Some(&fallbacks_path),
            OrchestratorModeView::Organic,
        );

        let fallback = fallback(&view).expect("the orchestrator's chain is the row");
        assert_eq!(fallback.model.as_deref(), Some("orx/backup"));
        assert_eq!(fallback.thinking.as_deref(), Some("high"));
    }

    #[test]
    fn a_profile_the_fallbacks_file_does_not_name_gets_no_row() {
        let (profiles_path, fallbacks_path) = paths();
        let other = FALLBACKS.replace("\"Mine\"", "\"Another\"");

        let view = view(
            &files(DOCUMENT, Some(&other)),
            Some(&profiles_path),
            Some(&fallbacks_path),
            OrchestratorModeView::Organic,
        );

        assert_eq!(view.status, ProfilesStatusView::Ready);
        assert!(
            fallback(&view).is_none(),
            "a chain recorded for another profile is not this profile's"
        );
    }

    #[test]
    fn a_chain_whose_steps_ask_for_different_efforts_draws_the_models_alone() {
        let (profiles_path, fallbacks_path) = paths();
        let mixed = FALLBACKS.replace(
            r#"{"model": "prov/fb-two", "thinking": "low"}"#,
            r#"{"model": "prov/fb-two", "thinking": "high"}"#,
        );

        let view = view(
            &files(DOCUMENT, Some(&mixed)),
            Some(&profiles_path),
            Some(&fallbacks_path),
            OrchestratorModeView::Organic,
        );

        let fallback = fallback(&view).expect("the chain is drawn");
        assert_eq!(
            fallback.model.as_deref(),
            Some("prov/fb-one \u{2192} prov/fb-two")
        );
        assert_eq!(
            fallback.thinking, None,
            "one effort column cannot speak for two different efforts"
        );
    }

    #[test]
    fn a_profile_that_already_has_a_fallback_role_keeps_exactly_one_row() {
        let (profiles_path, fallbacks_path) = paths();
        let with_role = DOCUMENT.replace(
            r#""worker": {"model": "prov/small", "thinking": "high", "tier": "high"}"#,
            r#""worker": {"model": "prov/small", "thinking": "high", "tier": "high"},
                "fallback": {"model": "prov/stale"}"#,
        );

        let view = view(
            &files(&with_role, Some(FALLBACKS)),
            Some(&profiles_path),
            Some(&fallbacks_path),
            OrchestratorModeView::Organic,
        );

        let rows: Vec<&ProfileRoleView> = view.profiles[0]
            .roles
            .iter()
            .filter(|role| role.role == "fallback")
            .collect();
        assert_eq!(
            rows.len(),
            1,
            "one row, so the pane's keyed list stays keyed"
        );
        assert_eq!(
            rows[0].model.as_deref(),
            Some("prov/fb-one \u{2192} prov/fb-two"),
            "the chain the sibling file records replaces a stale row of its own name"
        );
    }
}
