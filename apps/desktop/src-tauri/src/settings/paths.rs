//! Where the Settings pane reads: gentle-pi's config home (which holds
//! `profiles.json` and its sibling `profile-fallbacks.json`) and the agent
//! directory (which holds `auth.json`), plus the home flags every child and
//! every shell run gets (0.1.12, T2, T3, L13).
//!
//! Both paths follow rules the shell already owns rather than new ones:
//!
//! - the config home is `resolveGentlePiConfigHome` (gentle-shell-bundled.mjs:
//!   227-238): a bundled prefix's folder, then `GENTLE_PI_CONFIG_HOME`, then the
//!   launcher's absolute `GENTLE_SHELL_CONFIG_HOME` marker, then
//!   `<pi home>/gentle-ai`. The rule itself is
//!   [`gentle_runtime::profiles::config_home`]; this module only feeds it the
//!   environment.
//! - the agent directory is [`gentle_backend::home::agent_dir`], read off the
//!   same flags [`with_home_flag`](gentle_backend::home::with_home_flag) gives
//!   the children, so the pane reads the store the chats actually use.

use std::ffi::OsString;
use std::path::{Path, PathBuf};

use gentle_backend::home::{self, HomeChoice, HomeEnv};
use gentle_runtime::profiles::{config_home, ConfigHomeEnv, FALLBACKS_FILE, PROFILES_FILE};

/// The files the Settings pane reads, and the flags to reach the shell that
/// owns them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SettingsPaths {
    /// `profiles.json` in gentle-pi's config home, `None` when no home resolves.
    pub profiles: Option<PathBuf>,
    /// `profile-fallbacks.json` BESIDE it: the FALLBACK row's chains, from the
    /// same config home (L13). `None` whenever `profiles` is.
    pub fallbacks: Option<PathBuf>,
    /// `auth.json` in the agent directory of this run's home, `None` when no
    /// agent directory resolves.
    pub auth: Option<PathBuf>,
    /// The agent directory itself, `None` when no home resolves: the Usage pane
    /// reads the shell's `settings.json` from here and locates the sources'
    /// checkout under it (`<agent>/git/<host>/<owner>/<repo>`).
    pub agent: Option<PathBuf>,
    /// The home flags the children get, reused by every shell run of the pane so
    /// the check reads the same store the chats do.
    pub home_args: Vec<OsString>,
}

/// Resolves the two files from the process environment and this run's home
/// choice, exactly as the children are started.
pub fn resolve(choice: Option<HomeChoice>) -> SettingsPaths {
    let env = HomeEnv {
        choice,
        ..HomeEnv::from_process()
    };
    let home_args = home::with_home_flag(std::env::args_os().skip(1).collect(), &env);
    let agent = home::agent_dir(&home_args, &env);
    // The pi home is the folder of the agent directory (`~/.pi`, `~/.gentle-shell`,
    // or the parent of a `--home` folder), the last layer of the config rule.
    let pi_home = agent
        .as_deref()
        .and_then(Path::parent)
        .map(Path::to_path_buf);
    let pi_config = named(std::env::var_os("GENTLE_PI_CONFIG_HOME"));
    let marker = named(std::env::var_os("GENTLE_SHELL_CONFIG_HOME"));
    let config = config_home(&ConfigHomeEnv {
        // The desk is not bundled into a prefix it could name here, so this
        // layer stays unset: the shell's own readers cover that install shape.
        bundled_prefix: None,
        gentle_pi_config_home: pi_config.as_deref(),
        gentle_shell_config_home: marker.as_deref(),
        pi_home: pi_home.as_deref(),
    });
    SettingsPaths {
        // Both profile files come from the SAME config home: gentle-pi keeps the
        // chains beside the profiles (lib/model-fallback-store.ts:33-35).
        profiles: config.as_ref().map(|dir| dir.join(PROFILES_FILE)),
        fallbacks: config.as_deref().map(|dir| dir.join(FALLBACKS_FILE)),
        auth: agent.as_ref().map(|dir| dir.join("auth.json")),
        agent,
        home_args,
    }
}

/// One environment variable as a path, `None` when it is unset OR empty: empty
/// counts as unset in the shell's own rule too.
fn named(value: Option<OsString>) -> Option<PathBuf> {
    value.filter(|value| !value.is_empty()).map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_variable_names_no_folder() {
        // The shell's rule counts empty as unset, so the desk must too: an empty
        // marker would otherwise become the current directory.
        assert_eq!(named(None), None);
        assert_eq!(named(Some(OsString::new())), None);
        assert_eq!(
            named(Some(OsString::from("C:/config/gentle-ai"))),
            Some(PathBuf::from("C:/config/gentle-ai"))
        );
    }

    #[test]
    fn both_files_hang_off_the_home_the_children_use() {
        // No environment needed: with a linked home the agent directory is pi's
        // own, and both files sit where the shell writes them.
        let env = HomeEnv {
            user_home: Some(PathBuf::from("C:/Users/u")),
            choice: Some(HomeChoice::Link),
            ..HomeEnv::default()
        };
        let args = home::with_home_flag(Vec::new(), &env);
        let agent = home::agent_dir(&args, &env).expect("a linked home resolves");
        assert_eq!(agent, PathBuf::from("C:/Users/u/.pi").join("agent"));
        assert_eq!(
            agent
                .parent()
                .map(|pi| pi.join("gentle-ai").join("profiles.json")),
            Some(
                PathBuf::from("C:/Users/u/.pi")
                    .join("gentle-ai")
                    .join("profiles.json")
            )
        );
    }
}
