//! What the Updates pane shows (0.1.12, S2, S3, S7): which of the three pieces
//! this machine runs have a newer version published, decided by the shell's own
//! rule.
//!
//! Pure: no file, no network, no process. The versions arrive as data, so every
//! branch is testable without reaching anything.
//!
//! The comparison is `gentle-shell upgrade`'s own (main-channel.mjs:291-303):
//! strict numeric `x.y.z`, and a version that is not that shape - a prerelease,
//! a build suffix, a word - is NOT older, so it never claims an update. That is
//! the safe direction: a false "you are up to date" is a line, a false "update
//! available" is a wrong instruction to the user.
//!
//! What is compared, and what deliberately is not:
//!
//! - pi: `pi --version` (read from `gentle-shell --version`, which prints both)
//!   against `registry.npmjs.org/@earendil-works/pi-coding-agent/latest`.
//! - gentle-shell: its channel decides. A main-channel shell compares GitHub
//!   commit SHAs and is never compared against npm, and the desk cannot tell the
//!   channel cheaply, so the shell's `latest` stays unknown here: the pane shows
//!   its version and says nothing about an update it cannot prove.

use std::cmp::Ordering;

/// One row of the pane: what is installed, what is published, and whether the
/// second is newer. `update_available: None` is "cannot be told", which is a
/// different answer from "no".
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct VersionRow {
    pub current: Option<String>,
    pub latest: Option<String>,
    pub update_available: Option<bool>,
}

/// A strict `x.y.z`, or `None` for anything else.
pub fn parse(text: &str) -> Option<(u64, u64, u64)> {
    let text = text.trim();
    let mut parts = text.split('.');
    let major = number(parts.next()?)?;
    let minor = number(parts.next()?)?;
    let patch = number(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

/// One component of a version: ASCII digits only, so `+1` and `-1` are no
/// versions. A component too large for a `u64` is no version either.
fn number(part: &str) -> Option<u64> {
    if part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    part.parse().ok()
}

/// Whether `latest` is a newer version than `current`. A version that is not a
/// strict `x.y.z` is never older: prereleases, suffixes and garbage compare as
/// "no update".
pub fn is_older(current: &str, latest: &str) -> bool {
    let (Some(current), Some(latest)) = (parse(current), parse(latest)) else {
        return false;
    };
    latest.cmp(&current) == Ordering::Greater
}

/// One row from the two versions. Either one can be unknown; the comparison is
/// only made when both are there, so a probe that failed never looks like an
/// update, and a registry that did not answer never looks like "up to date".
pub fn row(current: Option<&str>, latest: Option<&str>) -> VersionRow {
    let clean = |version: Option<&str>| {
        version
            .map(str::trim)
            .filter(|version| !version.is_empty())
            .map(str::to_owned)
    };
    let current = clean(current);
    let latest = clean(latest);
    let update_available = match (current.as_deref(), latest.as_deref()) {
        (Some(current), Some(latest)) => Some(is_older(current, latest)),
        _ => None,
    };
    VersionRow {
        current,
        latest,
        update_available,
    }
}

/// What `gentle-shell --version` printed, split into the two versions it names.
/// The tool prints a line per fact (`gentle-shell 4.7.0`, `pi 1.5.2`, and the
/// home), so each line is read by its first word.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ShellVersions {
    pub shell: Option<String>,
    pub pi: Option<String>,
}

pub fn parse_version_output(text: &str) -> ShellVersions {
    let mut versions = ShellVersions::default();
    for line in text.lines() {
        let mut words = line.split_whitespace();
        let (Some(tool), Some(version)) = (words.next(), words.next()) else {
            continue;
        };
        let slot = match tool {
            "gentle-shell" => &mut versions.shell,
            "pi" => &mut versions.pi,
            _ => continue,
        };
        // The first answer wins: a later repetition is not a second version.
        if slot.is_none() {
            *slot = Some(version.to_owned());
        }
    }
    versions
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_strict_version_is_three_numbers() {
        assert_eq!(parse("0.1.11"), Some((0, 1, 11)));
        assert_eq!(parse(" 2.0.5\n"), Some((2, 0, 5)));
    }

    #[test]
    fn anything_but_three_numbers_is_no_version() {
        for text in [
            "",
            "1.5",
            "1.5.2.1",
            "1.5.2-dev",
            "1.5.2+build",
            "v1.5.2",
            "1.5.x",
            "+1.5.2",
            "-1.5.2",
            "latest",
        ] {
            assert_eq!(parse(text), None, "{text:?} is not a strict x.y.z");
        }
    }

    #[test]
    fn a_bigger_patch_minor_or_major_is_newer() {
        assert!(
            is_older("0.1.9", "0.1.11"),
            "reads the numbers, not the text"
        );
        assert!(is_older("0.1.9", "0.2.0"));
        assert!(is_older("1.5.2", "2.0.5"));
        assert!(is_older("1.9.0", "1.10.0"), "10 is after 9, not before a 0");
    }

    #[test]
    fn the_same_or_a_smaller_version_is_not_older() {
        assert!(!is_older("0.1.11", "0.1.11"));
        assert!(!is_older("0.1.11", "0.1.9"));
        assert!(!is_older("2.0.5", "1.5.2"));
    }

    #[test]
    fn a_prerelease_is_never_older() {
        // The shell's own rule: a prerelease is not compared at all, so it can
        // never announce an update.
        assert!(!is_older("0.1.11", "0.1.12-beta.1"));
        assert!(!is_older("0.1.12-beta.1", "0.1.11"));
        assert!(!is_older("1.5.2", "2.0.5-rc.1"));
    }

    #[test]
    fn garbage_is_never_older() {
        assert!(!is_older("0.1.11", "latest"));
        assert!(!is_older("garbage", "2.0.5"));
        assert!(!is_older("", "2.0.5"));
    }

    #[test]
    fn a_row_with_both_versions_answers_the_comparison() {
        assert_eq!(
            row(Some("1.5.2"), Some("2.0.5")),
            VersionRow {
                current: Some("1.5.2".to_owned()),
                latest: Some("2.0.5".to_owned()),
                update_available: Some(true),
            }
        );
        assert_eq!(
            row(Some("4.7.0"), Some("4.7.0")).update_available,
            Some(false)
        );
    }

    #[test]
    fn a_row_with_a_missing_side_never_claims_an_update() {
        // A probe that failed, or a registry that did not answer: unknown is not
        // "no update" and it is not an update either.
        assert_eq!(row(None, Some("2.0.5")).update_available, None);
        assert_eq!(row(Some("1.5.2"), None).update_available, None);
        assert_eq!(row(None, None).update_available, None);
        assert_eq!(row(Some("  "), Some("2.0.5")).current, None);
    }

    #[test]
    fn the_shell_version_output_names_both_versions() {
        // The real output of `gentle-shell --version` on a linked home.
        let output =
            "gentle-shell 4.7.0\npi 1.5.2\nhome isolated C:\\Users\\u\\.gentle-shell\\agent\n";
        assert_eq!(
            parse_version_output(output),
            ShellVersions {
                shell: Some("4.7.0".to_owned()),
                pi: Some("1.5.2".to_owned()),
            }
        );
    }

    #[test]
    fn an_output_without_the_lines_leaves_both_unknown() {
        assert_eq!(
            parse_version_output(""),
            ShellVersions {
                shell: None,
                pi: None
            }
        );
        assert_eq!(
            parse_version_output("gentle-shell\npi\nwarning: something\n"),
            ShellVersions {
                shell: None,
                pi: None
            }
        );
    }

    #[test]
    fn the_first_answer_of_each_tool_wins() {
        let output = "pi 1.5.2\ngentle-shell 4.7.0\npi 9.9.9\n";
        let versions = parse_version_output(output);
        assert_eq!(versions.pi.as_deref(), Some("1.5.2"));
        assert_eq!(versions.shell.as_deref(), Some("4.7.0"));
    }
}
