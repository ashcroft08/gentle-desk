//! The real install command, with the program swapped for a stand-in through
//! `GENTLE_DESK_NPM`: it must be exactly `install -g gentle-pi` and nothing
//! else, and a program that is not there must say it could not start.
//!
//! One test in this file, because it sets a process environment variable that no
//! other test may see.

use std::fs;

use gentle_desk_lib::shell_install::{CommandInstaller, Installer};

#[test]
fn the_real_install_is_npm_install_g_gentle_pi_and_a_missing_program_is_reported() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("argv.json");
    let script = dir.path().join("fake-npm.mjs");
    fs::write(
        &script,
        format!(
            "import fs from 'node:fs'
fs.writeFileSync({:?}, JSON.stringify(process.argv.slice(2)))
",
            record.to_string_lossy()
        ),
    )
    .unwrap();

    std::env::set_var("GENTLE_DESK_NPM", &script);
    assert_eq!(CommandInstaller::npm().install(), Ok(()));
    let argv: Vec<String> = serde_json::from_slice(&fs::read(&record).unwrap()).unwrap();
    assert_eq!(argv, ["install", "-g", "gentle-pi"]);

    let missing = dir.path().join("no-such-npm.exe");
    std::env::set_var("GENTLE_DESK_NPM", &missing);
    let error = CommandInstaller::npm().install().unwrap_err();
    assert!(error.contains("could not start"), "{error}");
    assert!(error.contains("no-such-npm"), "{error}");

    std::env::remove_var("GENTLE_DESK_NPM");
}
