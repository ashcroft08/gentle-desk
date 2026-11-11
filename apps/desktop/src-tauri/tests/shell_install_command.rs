//! The real install command, with the program swapped for a stand-in through
//! `GENTLE_DESK_NPM`: it must be exactly `install -g gentle-pi` and nothing
//! else, the lines it prints must reach the sink as they arrive, and a program
//! that is not there must say it could not start.
//!
//! One test in this file, because it sets a process environment variable that no
//! other test may see: a second one spawning a process beside it would read that
//! environment while it changes.

use std::sync::{Arc, Mutex};

use gentle_desk_lib::bridge::AppStateHolder;
use gentle_desk_lib::commands::with_install_line;
use gentle_desk_lib::contract::InstallView;
use gentle_desk_lib::shell_install::{CommandInstaller, Installer};

#[test]
fn the_real_install_is_npm_install_g_gentle_pi_its_lines_reach_the_sink_and_a_missing_program_is_reported(
) {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("argv.json");
    let script = dir.path().join("fake-npm.mjs");
    // What npm does on a machine that is installing: a line per package, and a
    // progress line it redraws with carriage returns.
    std::fs::write(
        &script,
        format!(
            "import fs from 'node:fs'
fs.writeFileSync({:?}, JSON.stringify(process.argv.slice(2)))
console.log('added 1 package')
process.stdout.write('npm WARN 10%\\rnpm WARN 100%\\n')
console.log('changed 1 package')
",
            record.to_string_lossy()
        ),
    )
    .unwrap();

    let lines: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let sink = {
        let lines = Arc::clone(&lines);
        move |line: &str| {
            lines
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .push(line.to_owned())
        }
    };

    std::env::set_var("GENTLE_DESK_NPM", &script);
    assert_eq!(CommandInstaller::npm().install(&sink), Ok(()));
    let argv: Vec<String> = serde_json::from_slice(&std::fs::read(&record).unwrap()).unwrap();
    assert_eq!(argv, ["install", "-g", "gentle-pi"]);

    // The redraw reached the sink as the one line it ended on, in order.
    let seen = lines
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .clone();
    assert_eq!(
        seen,
        ["added 1 package", "npm WARN 100%", "changed 1 package"]
    );

    // And the last of them is what the running screen shows: the holder's own
    // slice, with the line laid over it and nothing else moved.
    let holder = AppStateHolder::default();
    assert!(holder.begin_install(), "the screen shows a running install");
    let view = with_install_line(&holder, seen.last().unwrap());
    assert_eq!(
        view.shell.install,
        InstallView::Running {
            line: Some("changed 1 package".to_owned())
        }
    );

    let missing = dir.path().join("no-such-npm.exe");
    std::env::set_var("GENTLE_DESK_NPM", &missing);
    let error = CommandInstaller::npm().install(&sink).unwrap_err();
    assert!(error.contains("could not start"), "{error}");
    assert!(error.contains("no-such-npm"), "{error}");

    std::env::remove_var("GENTLE_DESK_NPM");
}
