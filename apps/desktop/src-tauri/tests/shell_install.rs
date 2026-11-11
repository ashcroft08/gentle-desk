//! Windowless tests of the missing-gentle-shell path (shell-choice S4): the view
//! knows whether the launcher resolves, the install is a small state machine
//! that never starts twice, "Check again" re-reads the machine, a successful
//! install re-reads it too (a failed one never claims anything), and the real
//! installer runs a command and says what went wrong in a bounded message.
//! Nothing here installs anything: the command under test is `node`.

use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::commands::after_install;
use gentle_desk_lib::contract::InstallView;
use gentle_desk_lib::shell_install::{CommandInstaller, Installer};

#[test]
fn a_run_assumes_the_launcher_is_there_until_it_looks() {
    let shell = AppStateHolder::default().view().shell;

    assert!(shell.launcher_found);
    assert_eq!(shell.install, InstallView::Idle);
}

#[test]
fn a_launcher_that_is_not_found_reaches_the_view() {
    let holder = AppStateHolder::default();
    holder.set_launcher_found(false);

    assert!(!holder.view().shell.launcher_found);
}

#[test]
fn an_install_starts_once_and_says_how_it_ended() {
    let holder = AppStateHolder::default();

    assert!(holder.begin_install(), "the first request starts it");
    assert_eq!(
        holder.view().shell.install,
        InstallView::Running { line: None }
    );
    assert!(
        !holder.begin_install(),
        "a second one while it runs does not"
    );

    holder.finish_install(Err("npm is not installed".to_owned()));
    assert_eq!(
        holder.view().shell.install,
        InstallView::Failed {
            message: "npm is not installed".to_owned()
        }
    );

    assert!(
        holder.begin_install(),
        "after a failure it can be tried again"
    );
    holder.finish_install(Ok(()));
    assert_eq!(holder.view().shell.install, InstallView::Done);
    assert!(
        !holder.begin_install(),
        "a finished install is not repeated"
    );
}

#[test]
fn checking_again_reads_the_launcher_and_keeps_the_install_outcome() {
    let holder = AppStateHolder::default();
    holder.set_launcher_found(false);
    assert!(holder.begin_install());
    holder.finish_install(Ok(()));

    let view = holder.recheck_launcher(&|| true);

    assert!(view.shell.launcher_found);
    assert_eq!(view.shell.install, InstallView::Done);

    let still_missing = holder.recheck_launcher(&|| false);
    assert!(!still_missing.shell.launcher_found);
}

/// A successful install leaves the user on a screen that says it worked: the
/// launcher is there now, so the run looks for it once more instead of asking
/// for another "Check again", and the home question follows (S2, S4).
#[test]
fn a_successful_install_re_probes_the_launcher_and_unlocks_the_home_question() {
    const PI: &str = "C:/Users/someone/.pi/agent";
    let holder = AppStateHolder::default();
    holder.set_launcher_found(false);
    holder.set_pi_path(Some(PI.to_owned()));
    assert!(
        !holder.view().shell.ask,
        "the install screen asks nothing of someone who cannot run yet"
    );
    assert!(holder.begin_install());

    let view = after_install(&holder, Ok(()), &|| true);

    assert!(
        view.shell.launcher_found,
        "the install put the launcher there"
    );
    assert_eq!(view.shell.install, InstallView::Done);
    assert!(
        view.shell.ask,
        "pi is there, nothing is chosen and gentle-shell can run"
    );
}

/// The other half of the same rule: a failed install says nothing about the
/// machine, so it never probes and never claims the launcher is there.
#[test]
fn a_failed_install_never_claims_the_launcher_is_there() {
    const PI: &str = "C:/Users/someone/.pi/agent";
    let holder = AppStateHolder::default();
    holder.set_launcher_found(false);
    holder.set_pi_path(Some(PI.to_owned()));
    assert!(holder.begin_install());

    let view = after_install(&holder, Err("npm is not installed".to_owned()), &|| {
        panic!("a failed install is no reason to look for the launcher")
    });

    assert!(!view.shell.launcher_found);
    assert_eq!(
        view.shell.install,
        InstallView::Failed {
            message: "npm is not installed".to_owned()
        }
    );
    assert!(!view.shell.ask);
}

#[test]
fn the_installer_reports_success_of_the_command() {
    let installer = CommandInstaller::new("node", ["-e", "process.exit(0)"]);

    assert_eq!(installer.install(&|_| {}), Ok(()));
}

#[test]
fn the_installer_reports_the_last_lines_of_a_failed_command() {
    let installer = CommandInstaller::new(
        "node",
        [
            "-e",
            "console.error('first'); console.error('npm ERR! code E404'); process.exit(3)",
        ],
    );

    let error = installer.install(&|_| {}).unwrap_err();

    assert!(error.contains("npm ERR! code E404"), "{error}");
    assert!(error.contains('3'), "the exit code is named: {error}");
}

#[test]
fn a_command_that_cannot_start_says_so() {
    let installer = CommandInstaller::new("definitely-not-a-real-program-xyz", ["install"]);

    let error = installer.install(&|_| {}).unwrap_err();

    assert!(error.contains("could not start"), "{error}");
}

#[test]
fn a_failure_message_stays_small_whatever_the_command_prints() {
    let installer = CommandInstaller::new(
        "node",
        ["-e", "for (let i = 0; i < 5000; i++) console.error('line ' + i + ' ' + 'x'.repeat(80)); process.exit(1)"],
    );

    let error = installer.install(&|_| {}).unwrap_err();

    assert!(error.len() <= 2_100, "{} bytes", error.len());
    assert!(error.contains("line 4999"), "the end of the output is kept");
}
