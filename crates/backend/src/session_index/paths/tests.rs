use super::*;
use std::path::Path;

#[cfg(windows)]
const SEP: &str = r"\";
#[cfg(not(windows))]
const SEP: &str = "/";

#[cfg(windows)]
const DIR: &str = r"C:\home\sessions";
#[cfg(not(windows))]
const DIR: &str = "/home/sessions";

/// `relative` with `/` separators, joined onto `dir`.
fn under(dir: &str, relative: &str) -> PathBuf {
    Path::new(dir).join(relative.replace('/', SEP))
}

/// The cache key and folded form of `<DIR>/<relative>`.
fn key(relative: &str) -> Option<(PathBuf, String)> {
    session_key(Path::new(DIR), &under(DIR, relative))
}

#[test]
fn a_file_exactly_two_components_below_the_directory_is_a_session_file() {
    let (path, folded) = key("--C--a--/s.jsonl").expect("a session file");
    assert_eq!(path, under(DIR, "--C--a--/s.jsonl"));
    assert!(folded.ends_with("/s.jsonl"), "{folded}");
}

#[test]
fn other_depths_other_extensions_and_directories_are_not_session_files() {
    for ignored in [
        "s.jsonl",            // depth 1: one directory per project holds them
        "--C--a--/b/s.jsonl", // depth 3
        "--C--a--/notes.txt",
        "--C--a--/s.jsonl.bak",
        "--C--a--",   // a project directory
        "--C--a--/b", // a directory below a project
    ] {
        assert!(key(ignored).is_none(), "{ignored} is not a session file");
    }
}

/// `session_files` matches `jsonl` exactly; a second, different rule here would
/// let `refresh` and `refresh_paths` disagree about what a session is.
#[test]
fn an_upper_case_extension_is_not_a_session_file() {
    assert!(key("--C--a--/s.JSONL").is_none());
}

#[test]
fn a_relative_path_is_not_a_session_file() {
    let relative = Path::new("--C--a--").join("s.jsonl");
    assert!(session_key(Path::new(DIR), &relative).is_none());
}

#[test]
fn a_path_outside_the_sessions_directory_is_not_a_session_file() {
    for outside in ["/elsewhere", r"C:\home\sessions2"] {
        let path = under(outside, "--C--a--/s.jsonl");
        assert!(session_key(Path::new(DIR), &path).is_none(), "{outside}");
    }
}

#[test]
fn a_cur_dir_component_does_not_hide_a_session_file() {
    let (path, _) = key("--C--a--/./s.jsonl").expect("a session file");
    assert_eq!(path, under(DIR, "--C--a--/s.jsonl"));
}

#[cfg(windows)]
#[test]
fn windows_accepts_the_verbatim_spelling_a_watcher_reports() {
    let path = under(DIR, "--C--a--/s.jsonl");
    let verbatim = PathBuf::from(format!(r"\\?\{}", path.display()));
    assert_eq!(session_key(Path::new(DIR), &verbatim).unwrap().0, path);
}

#[cfg(windows)]
#[test]
fn windows_ignores_the_case_of_the_sessions_directory() {
    let path = under(DIR, "--C--a--/s.jsonl");
    let upper = DIR.to_uppercase();
    let upper = Path::new(&upper);
    assert_eq!(
        session_key(upper, &path).unwrap().0,
        upper.join("--C--a--").join("s.jsonl")
    );
}

#[cfg(windows)]
#[test]
fn windows_folds_the_verbatim_and_plain_spellings_together() {
    let path = under(DIR, "--C--a--/s.jsonl");
    let verbatim = PathBuf::from(format!(r"\\?\{}", path.display()));
    assert_eq!(
        key("--C--a--/s.jsonl").unwrap().1,
        session_key(Path::new(DIR), &verbatim).unwrap().1
    );
}

#[cfg(windows)]
#[test]
fn windows_accepts_forward_slashes() {
    let path = under(DIR, "--C--a--/s.jsonl");
    let slashes = PathBuf::from(path.display().to_string().replace('\\', "/"));
    assert!(session_key(Path::new(DIR), &slashes).is_some());
}
