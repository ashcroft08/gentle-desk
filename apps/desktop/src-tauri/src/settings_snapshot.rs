//! The Settings dialog's ONE read: everything its pages draw, gathered once when
//! the dialog opens, off the window's thread, as one plain object.
//!
//! The 0.1.12 panes each asked for their own slice when they appeared (Account
//! ran one `auth check` per provider, Usage ran the sources' script), so a run
//! that walked the rail ran the same subprocesses again on every visit, and the
//! pane someone was reading stayed blank until its own read answered. Here the
//! dialog asks ONCE, on the open, and every page draws from the answer: a visit
//! to a section costs nothing, and the object arrives whole.
//!
//! What it carries is the plain data of the five sections that show the machine:
//!
//! - the part of the slice [`ViewState`](crate::contract::ViewState) already
//!   holds (theme, language, the notifications mode, the skipped update version,
//!   home, running version, the open-chat gate and what became of the config
//!   file), which costs no read at all;
//! - the Account pane's auth view, the Usage pane's usage view and the Updates
//!   pane's update view, each of which IS a read;
//! - the General pane's environment check (0.9.0 general T2, S2-S6): one row per
//!   tool the app and the shell depend on. The shell's and pi's rows are the
//!   versions read the Updates slice already made - the same one
//!   `gentle-shell --version` run, never a second one - and node and git are the
//!   load's only two extra bounded probes.
//!
//! What it deliberately does not carry: a chat's profiles, session mode or
//! images. Those are per chat, they are read through that chat's own child, and
//! Settings names none of them since profiles-editor T2d.
//!
//! Every field is the contract's own view, so nothing here can add a shape the
//! push does not already have, and every read's failure is that section's status:
//! a store that cannot be read, a run that fails or a probe that does not answer
//! says so in its own section and leaves the other two - and the dialog's own
//! half - untouched. No secret travels: the auth view is names, states and
//! reasons, exactly as `settings::auth_view` builds it.

use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;

use gentle_core::i18n::Language;
use gentle_runtime::chat_runtime::PackagesRead;

use crate::contract::{
    AuthView, GeneralView, LanguageView, NotificationsMode, ShellView, ThemeView, UpdatesView,
    UsageView,
};
use crate::settings::{self, FileRead, NodeProbe, ProviderProbe, ToolProbe, VersionProbe, WebGet};

/// The half of the snapshot this run already holds in memory: the things the
/// dialog draws that cost no read, taken from the slice the last push carried.
#[derive(Debug, Clone, PartialEq)]
pub struct SettingsConfig {
    /// The appearance of the whole window, as the app config remembers it.
    pub theme: ThemeView,
    /// The language the app's own words are written in, `None` while no
    /// preference is stored.
    pub language: Option<LanguageView>,
    /// When the app notifies the user about a chat that needs them, as the app
    /// config remembers it.
    pub notifications: NotificationsMode,
    /// The version the user skipped, as the app config remembers it (0.8.0
    /// settings-polish S1, S2), `None` while nothing is skipped. Live, like the
    /// theme: the pane changes it from inside this card, and the dialog's load
    /// carries it so a skip is visible without waiting for the next check. It is
    /// carried and never applied - the load reports the real state, which is what
    /// offers to stop skipping.
    pub skipped_update: Option<String>,
    /// The shell and the home this run sees, which the Home section draws.
    pub shell: ShellView,
    /// The version this run RUNS as, which the About section draws.
    pub app_version: Option<String>,
    /// How many chats are open: the Updates pane's gate.
    pub open_chats: u32,
    /// What became of the app config file, when it was set aside or a save
    /// failed. The chat's own notices are not part of Settings and stay out.
    pub config_notice: Option<String>,
}

/// The three reads of one load, cloned out of the bridge before the load starts
/// so the gathering holds no lock while a file, a process or a socket is read.
/// Every field is a seam: a test replaces one and the machine stays untouched.
#[derive(Clone)]
pub struct Reads {
    /// The file reader the three reads share (the auth store, the shell's
    /// settings and the sources' script).
    pub files: FileRead,
    /// `auth.json` in the agent directory, `None` when no agent directory
    /// resolves.
    pub auth: Option<PathBuf>,
    /// The agent directory itself, where the Usage read looks for the shell's
    /// settings and the sources' checkout.
    pub agent: Option<PathBuf>,
    /// One `auth check --no-refresh` per provider.
    pub probe: ProviderProbe,
    /// `gentle-shell --version`, the one process that names both versions.
    pub versions: VersionProbe,
    /// The release manifest and pi's registry, one GET each.
    pub web: WebGet,
    /// One run of the usage sources' own script.
    pub node: NodeProbe,
    /// One bounded `node --version` / `git --version` run each: the General
    /// page's environment check (0.9.0 general T2, S3). One ask per tool per
    /// load, never on the window's thread, because the whole gather is on a
    /// blocking worker. The shell's and pi's rows are NOT asked here: they are the
    /// versions read the Updates rows already made from the same load.
    pub tools: ToolProbe,
    /// The extensions read over ONE open chat's child (packages-status T1, S1-S5):
    /// `Unasked` when no chat is open, so a load with nothing to ask touches no
    /// host and starts no child. Called exactly once per load, which is the one
    /// read the dialog open (and its "Check again") makes (S2).
    pub packages: Arc<dyn Fn() -> PackagesRead + Send + Sync>,
    /// The version this run runs as, which the desk's row is compared against.
    pub desk_version: Option<String>,
    /// The desk's release manifest, named once at startup by the update check.
    pub desk_manifest: Option<String>,
    /// The language every read words its own refusals in.
    pub language: Language,
}

/// One Settings dialog's whole content (settings-snapshot): the plain data
/// every page draws, gathered once per open and never on the window's thread.
///
/// The dialog asks for it when it opens and draws what it already had until the
/// answer lands, so a section is never blank and no page reads anything on its
/// own. Every section is the contract's own view and every one of them carries
/// its own status: what failed says so there.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SettingsSnapshot {
    /// The appearance of the whole window: the theme cards, and the
    /// `data-theme` the page paints. The dialog keeps drawing the live slice for
    /// this one, because it is what the user CHANGES from inside the card; the
    /// field says what the load saw.
    pub theme: ThemeView,
    /// The language the app's own words are written in, `None` while no
    /// preference is stored. Live, like the theme, and carried for the same
    /// reason.
    pub language: Option<LanguageView>,
    /// When the app notifies the user, live like the theme and carried for the
    /// same reason: the Settings pane draws the mode this run holds.
    pub notifications: NotificationsMode,
    /// The shell, its home and where installing it stands: what the Home section
    /// draws. Live, like the theme.
    pub shell: ShellView,
    /// The version this run runs as, which the About section draws, or `None`
    /// for a run that never named one.
    pub app_version: Option<String>,
    /// How many chats are open, which gates the Updates pane's Update button.
    pub open_chats: u32,
    /// What became of the app config file, when it was set aside or a save
    /// failed. `None` when it was read, or was not there, and every save worked.
    pub config_notice: Option<String>,
    /// Where the shell's authentication stands: one row per provider the store
    /// lists, with its own check's answer. Names only, never a key or a token.
    pub auth: AuthView,
    /// What the machine runs and what is published for it: the desk, pi and
    /// gentle-shell, each row with its own answer and its own reason for having
    /// none.
    pub updates: UpdatesView,
    /// The subscription usage the shell's own sources report, or the reason they
    /// could not be read. Nothing here touches a chat.
    pub usage: UsageView,
    /// The General page's environment check (0.9.0 general T2, S2-S6): one row per
    /// tool the app and the shell depend on, state and version only, and the
    /// instant this load read them.
    ///
    /// Additive and optional on the wire, like the Extensions block: a payload
    /// written before this page existed reads as [`GeneralView::default`] - no
    /// rows and no instant, "nothing was read yet" - which is never "nothing is
    /// installed". This build always carries it as a VALUE.
    #[serde(default)]
    pub general: GeneralView,
}

/// Gathers one snapshot: the three reads, in the order the panes used to ask for
/// them, over the seams the caller resolved.
///
/// The caller runs this off the window's thread (the command uses a blocking
/// worker), because all three reads touch the machine: a file, a process, a
/// script and two sockets. Nothing here holds a lock, and nothing here can
/// decide the whole load failed: each read answers with its own status, so a
/// failing source leaves its section saying why and the rest of the snapshot
/// whole.
pub fn gather(config: SettingsConfig, reads: &Reads) -> SettingsSnapshot {
    let skipped = config.skipped_update;
    // The ONE instant this load is worded from (S4): the Updates rows' last-checked
    // line and the environment check's own are the same reading of the clock, so
    // the two pages can never disagree about when they last read the machine.
    let now_ms = gentle_runtime::clock::now_ms();
    // Whether this machine has the shell at all, which the environment check takes
    // from the shell view instead of resolving the launcher a second time (S3).
    let launcher_found = config.shell.launcher_found;
    // The one chat's extensions, read ONCE here (S2): the ask blocks on the
    // channel's own deadline, which is why the caller gathers off the window's
    // thread, and `Unasked` costs nothing at all.
    let packages = (reads.packages)();
    // The Updates rows first: their two version rows ARE the environment check's
    // shell and pi, from the one `gentle-shell --version` run of this load.
    let updates = settings::updates_view(
        reads.desk_version.as_deref(),
        reads.desk_manifest.as_deref(),
        &reads.web,
        &reads.versions,
        reads.language,
        skipped.as_deref(),
        // The dialog's own load IS a check, so its rows were read just now.
        now_ms,
        &packages,
    );
    // The environment check, from the versions read just above plus one bounded
    // probe per tool that prints its own (S3). git's row exists whatever its
    // probe answers (S6).
    let general = settings::general_view(
        &updates,
        launcher_found,
        &reads.tools,
        reads.language,
        now_ms,
    );
    SettingsSnapshot {
        theme: config.theme,
        language: config.language,
        notifications: config.notifications,
        shell: config.shell,
        app_version: config.app_version,
        open_chats: config.open_chats,
        config_notice: config.config_notice,
        auth: settings::auth_view(
            &reads.files,
            reads.auth.as_deref(),
            &reads.probe,
            reads.language,
        ),
        updates,
        usage: settings::usage_view(
            &reads.files,
            reads.agent.as_deref(),
            &reads.node,
            now_ms,
            reads.language,
        ),
        general,
    }
}
