use super::*;
use std::fs;

const NPM_SHIM: &str = "@ECHO off\r\nGOTO start\r\n:start\r\nSETLOCAL\r\nIF EXIST \"%dp0%\\node.exe\" (\r\n  SET \"_prog=%dp0%\\node.exe\"\r\n) ELSE (\r\n  SET \"_prog=node\"\r\n)\r\nendLocal & goto #_undefined_# 2>NUL || title %COMSPEC% & \"%_prog%\"  \"%dp0%\\node_modules\\gentle-pi\\bin\\gentle-shell.mjs\" %*\r\n";

const ENTRY: &str = "node_modules\\gentle-pi\\bin\\gentle-shell.mjs";

/// A fake npm global dir: the shim, plus the entry it points at when asked.
fn npm_dir(with_entry: bool) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("gentle-shell.cmd"), NPM_SHIM).unwrap();
    if with_entry {
        let bin = dir.path().join("node_modules/gentle-pi/bin");
        fs::create_dir_all(&bin).unwrap();
        fs::write(bin.join("gentle-shell.mjs"), "").unwrap();
    }
    dir
}

fn on_path(dirs: &[&Path]) -> LauncherEnv {
    LauncherEnv {
        gentle_shell_bin: None,
        path: Some(std::env::join_paths(dirs).unwrap()),
    }
}

fn with_bin(bin: &Path) -> LauncherEnv {
    LauncherEnv {
        gentle_shell_bin: Some(bin.into()),
        path: None,
    }
}

#[test]
fn shim_entry_is_extracted() {
    assert_eq!(shim_js_entry(NPM_SHIM).as_deref(), Some(ENTRY));
    assert_eq!(shim_js_entry("@echo off\r\nfoo.exe %*"), None);
}

#[test]
fn cmd_shim_on_path_resolves_to_node_and_the_real_entry() {
    let npm = npm_dir(true);
    let launcher = resolve_for(&on_path(&[npm.path()]), true).unwrap();
    assert_eq!(launcher.program, PathBuf::from("node"));
    assert_eq!(launcher.args, vec![OsString::from(npm.path().join(ENTRY))]);
}

#[test]
fn node_exe_beside_the_shim_is_preferred() {
    let npm = npm_dir(true);
    fs::write(npm.path().join("node.exe"), "").unwrap();
    let launcher = resolve_for(&on_path(&[npm.path()]), true).unwrap();
    assert_eq!(launcher.program, npm.path().join("node.exe"));
}

#[test]
fn unreadable_shim_falls_back_to_cmd_c() {
    let npm = npm_dir(false); // the shim points at an entry that is not there
    let launcher = resolve_for(&on_path(&[npm.path()]), true).unwrap();
    assert_eq!(launcher.program, PathBuf::from("cmd.exe"));
    assert_eq!(
        launcher.args,
        vec![
            OsString::from("/c"),
            OsString::from(npm.path().join("gentle-shell.cmd"))
        ]
    );
}

#[test]
fn env_override_with_js_entry_runs_under_node() {
    let dir = tempfile::tempdir().unwrap();
    let entry = dir.path().join("gentle-shell.mjs");
    fs::write(&entry, "").unwrap();
    let launcher = resolve_for(&with_bin(&entry), false).unwrap();
    assert_eq!(launcher.program, PathBuf::from("node"));
    assert_eq!(launcher.args, vec![OsString::from(entry)]);
}

#[test]
fn env_override_with_cmd_shim_is_resolved_too() {
    let npm = npm_dir(true);
    let launcher = resolve_for(&with_bin(&npm.path().join("gentle-shell.cmd")), true).unwrap();
    assert_eq!(launcher.program, PathBuf::from("node"));
    assert_eq!(launcher.args, vec![OsString::from(npm.path().join(ENTRY))]);
}

#[test]
fn env_override_with_plain_binary_runs_directly() {
    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("gentle-shell");
    fs::write(&bin, "").unwrap();
    assert_eq!(
        resolve_for(&with_bin(&bin), false).unwrap(),
        Launcher {
            program: bin,
            args: vec![]
        }
    );
}

#[test]
fn missing_override_is_an_error_naming_the_path() {
    let err = resolve_for(&with_bin(Path::new("nope/gentle-shell.mjs")), false).unwrap_err();
    assert!(matches!(err, LauncherError::BinMissing(_)));
    assert!(err.to_string().contains("nope"));
}

#[test]
fn nothing_found_explains_both_options() {
    let empty = tempfile::tempdir().unwrap();
    let err = resolve_for(&on_path(&[empty.path()]), true).unwrap_err();
    assert_eq!(err, LauncherError::NotFound);
    let text = err.to_string();
    assert!(text.contains("npm install -g gentle-pi") && text.contains("GENTLE_SHELL_BIN"));
}

#[test]
fn unix_path_lookup_uses_the_plain_name() {
    let dir = tempfile::tempdir().unwrap();
    let bin = dir.path().join("gentle-shell");
    fs::write(&bin, "").unwrap();
    assert_eq!(
        resolve_for(&on_path(&[dir.path()]), false).unwrap().program,
        bin
    );
}
