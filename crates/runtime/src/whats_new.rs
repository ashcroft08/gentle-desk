//! What a run follows from the version the last one stored, and what to do with
//! the notes of the version it follows (what's-new S1, S3, S5).
//!
//! Pure: no file, no network, no window. The fetch arrives as data - the manifest
//! it read, or the reason it could not be read - so both branches are decided
//! here and are testable without a network.

use serde::{Deserialize, Deserializer};

/// What the version the last run stored says about this one (S1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateRun {
    /// Nothing stored: the first run of a build that predates this feature, or a
    /// run whose config was never written. Nothing is shown; the run records
    /// its version before it returns, so the next update is detected.
    FirstRun,
    /// The stored version is the one this run runs as: it follows no update.
    SameVersion,
    /// The stored version is another one: this run follows an update.
    Updated,
}

/// What this run follows, from the version the config stores and the one it runs
/// as. Inequality in EITHER direction is an update: a run that went back to an
/// older build also follows an install.
pub fn detected(last_run_version: Option<&str>, app_version: &str) -> UpdateRun {
    match last_run_version {
        None => UpdateRun::FirstRun,
        Some(stored) if stored == app_version => UpdateRun::SameVersion,
        Some(_) => UpdateRun::Updated,
    }
}

/// The part of the updater's `latest.json` the what's-new step reads: the version
/// the manifest announces (compared with this run's own) and the notes the
/// release body was staged into. The rest of the manifest - the platforms, the
/// urls, the signatures - belongs to the updater, never to this step.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize)]
pub struct Latest {
    /// The version the manifest announces. Absent reads as no version, which
    /// matches no run and so shows nothing.
    #[serde(default)]
    pub version: String,
    /// The notes as plain text. Absent, `null` or not text reads as no text, so a
    /// manifest that carries the rest still announces its version (S3).
    #[serde(default, deserialize_with = "text_or_empty")]
    pub notes: String,
}

/// What the step does with what the fetch came to (S3, S5).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Push these notes, and store this run's version when the modal is
    /// dismissed: that is what makes the modal show once per version.
    Show { version: String, notes: String },
    /// Push nothing and store this run's version now.
    Store,
}

/// What to do with the fetch: the notes are shown only for the manifest of THIS
/// version. Anything else - another version, or no manifest read at all - is
/// nothing to show, and the bookkeeping still happens (S3, S5).
pub fn decide(fetched: Result<Latest, String>, app_version: &str) -> Decision {
    match fetched {
        Ok(latest) if latest.version == app_version => Decision::Show {
            // This run's own version, never the manifest's, so what the modal
            // says and what is stored are the same fact (S5).
            version: app_version.to_owned(),
            notes: latest.notes,
        },
        _ => Decision::Store,
    }
}

/// Why a read on demand of this version's notes has no notes to show (T5, S9,
/// D6): the reason the About pane draws as one quiet line instead of an empty
/// modal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Missing {
    /// The manifest was not read at all: no network, no answer inside the
    /// budget, or an answer that is not a success. The seam's own words are
    /// deliberately not carried here (they are the client's, never the
    /// screen's), so every one of them is this same quiet reason.
    Unreachable,
    /// The manifest read announces ANOTHER version - the release after this one,
    /// or an older one - so its notes are never this run's to show.
    AnotherVersion,
    /// The manifest is this version's and its text is empty or only whitespace.
    NoText,
}

/// What one read on demand of the running version's notes came to (T5, S9, D6):
/// the About pane's button, which is not an update run and stores nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OnDemand {
    /// The notes of THIS version, exactly as the manifest read carries them.
    Notes(String),
    /// Nothing to show, for the reason the page words in one line.
    Missing(Missing),
}

/// The notes of this version, read on demand: the same manifest and the same
/// version rule [`decide`] applies at startup - a manifest of another version
/// never reaches the modal - with the reasons the startup step does not need,
/// because it only stores the version in silence.
///
/// Nothing here stores, clears or compares the version the last run was: a read
/// on demand is not an update run (T5).
pub fn on_demand(fetched: Result<Latest, String>, app_version: &str) -> OnDemand {
    match fetched {
        Err(_) => OnDemand::Missing(Missing::Unreachable),
        Ok(latest) if latest.version != app_version => OnDemand::Missing(Missing::AnotherVersion),
        Ok(latest) if latest.notes.trim().is_empty() => OnDemand::Missing(Missing::NoText),
        Ok(latest) => OnDemand::Notes(latest.notes),
    }
}

/// The notes as plain text: an absent, `null` or non-text `notes` is no text.
fn text_or_empty<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let stored = Option::<serde_json::Value>::deserialize(deserializer)?;
    Ok(stored
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_stored_is_a_first_run() {
        // The config of every build before this feature carries no version, and a
        // first run shows no modal: it is not an update of anything.
        assert_eq!(detected(None, "0.1.8"), UpdateRun::FirstRun);
    }

    #[test]
    fn the_stored_version_of_this_one_is_no_update() {
        // What the dismiss of the modal stores: the next run of the same version
        // follows no update, whatever the endpoint says.
        assert_eq!(detected(Some("0.1.8"), "0.1.8"), UpdateRun::SameVersion);
    }

    #[test]
    fn another_stored_version_is_an_update() {
        // Older AND newer: the rule is inequality, because a run of another build
        // is a run after an update in both directions.
        assert_eq!(detected(Some("0.1.7"), "0.1.8"), UpdateRun::Updated);
        assert_eq!(detected(Some("0.1.9"), "0.1.8"), UpdateRun::Updated);
        assert_eq!(detected(Some("0.1.8.1"), "0.1.8"), UpdateRun::Updated);
    }

    #[test]
    fn the_manifest_of_this_version_shows_its_notes() {
        let manifest = Latest {
            version: "0.1.8".to_owned(),
            notes: "The sidebar folds.".to_owned(),
        };
        assert_eq!(
            decide(Ok(manifest), "0.1.8"),
            Decision::Show {
                version: "0.1.8".to_owned(),
                notes: "The sidebar folds.".to_owned(),
            }
        );
    }

    #[test]
    fn the_shown_version_is_this_runs_and_never_the_manifests() {
        // A manifest of another version never reaches the modal, so the version
        // the modal carries is always the one this run runs as (S5).
        let manifest = Latest {
            version: "0.1.8".to_owned(),
            notes: "notes".to_owned(),
        };
        let Decision::Show { version, .. } = decide(Ok(manifest), "0.1.8") else {
            panic!("a manifest of this version shows its notes");
        };
        assert_eq!(version, "0.1.8");
    }

    #[test]
    fn a_manifest_of_another_version_stores_and_shows_nothing() {
        // A stale manifest, or one that announces a release the app never became:
        // nothing is shown, and the version is still stored (S5).
        let stale = Latest {
            version: "0.1.7".to_owned(),
            notes: "The sidebar folds.".to_owned(),
        };
        assert_eq!(decide(Ok(stale), "0.1.8"), Decision::Store);
        let ahead = Latest {
            version: "0.1.9".to_owned(),
            notes: "The sidebar folds.".to_owned(),
        };
        assert_eq!(decide(Ok(ahead), "0.1.8"), Decision::Store);
    }

    #[test]
    fn a_fetch_that_failed_stores_and_shows_nothing() {
        assert_eq!(
            decide(Err("no network".to_owned()), "0.1.8"),
            Decision::Store
        );
    }

    #[test]
    fn the_manifest_reads_the_version_and_the_notes_of_the_updater() {
        let manifest: Latest = serde_json::from_str(
            r#"{"version":"0.1.8","notes":"The sidebar folds.","pub_date":"2026-11-12T00:00:00Z","platforms":{}}"#,
        )
        .expect("a latest.json as the release workflow writes it reads");
        assert_eq!(manifest.version, "0.1.8");
        assert_eq!(manifest.notes, "The sidebar folds.");
    }

    #[test]
    fn a_read_on_demand_answers_the_notes_of_this_version() {
        // The About pane's button (T5, D6): the notes of the version this run
        // runs as, from the same manifest and by the same version rule the
        // startup step uses.
        let manifest = Latest {
            version: "0.1.8".to_owned(),
            notes: "The sidebar folds.".to_owned(),
        };
        assert_eq!(
            on_demand(Ok(manifest), "0.1.8"),
            OnDemand::Notes("The sidebar folds.".to_owned())
        );
    }

    #[test]
    fn a_read_on_demand_never_answers_another_versions_notes() {
        // Older AND newer: the manifest of another release carries notes about a
        // version this run is not, so they are never this run's to show (T5).
        for version in ["0.1.7", "0.1.9", "0.1.8.1"] {
            let manifest = Latest {
                version: version.to_owned(),
                notes: "Notes of another release.".to_owned(),
            };
            assert_eq!(
                on_demand(Ok(manifest), "0.1.8"),
                OnDemand::Missing(Missing::AnotherVersion),
                "a manifest of {version} is not this run's"
            );
        }
    }

    #[test]
    fn a_read_on_demand_that_read_no_manifest_says_it_could_not() {
        assert_eq!(
            on_demand(Err("no network".to_owned()), "0.1.8"),
            OnDemand::Missing(Missing::Unreachable)
        );
    }

    #[test]
    fn a_read_on_demand_of_a_release_with_no_text_has_nothing_to_show() {
        // The manifest of THIS version whose text is empty or whitespace: the
        // read answers that there is no text, never an empty modal.
        for notes in ["", "   ", "\n\t"] {
            let manifest = Latest {
                version: "0.1.8".to_owned(),
                notes: notes.to_owned(),
            };
            assert_eq!(
                on_demand(Ok(manifest), "0.1.8"),
                OnDemand::Missing(Missing::NoText)
            );
        }
    }

    #[test]
    fn a_manifest_without_notes_announces_its_version_with_no_text() {
        // The release body is the notes, and a release written without one is
        // still a release worth announcing: the version travels, the text is empty.
        let manifest: Latest =
            serde_json::from_str(r#"{"version":"0.1.8"}"#).expect("no notes is not a bad manifest");
        assert_eq!(manifest.version, "0.1.8");
        assert_eq!(manifest.notes, "");
        let null: Latest = serde_json::from_str(r#"{"version":"0.1.8","notes":null}"#)
            .expect("null notes is not a bad manifest");
        assert_eq!(null.notes, "");
    }
}
