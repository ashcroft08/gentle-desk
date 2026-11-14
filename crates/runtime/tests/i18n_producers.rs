//! The app's own words AS THE PRODUCERS ANSWER THEM (0.2.0, S3, D6): each of the
//! runtime's three producers takes the language its caller holds, so the Spanish
//! column reaches the caller and not only the catalog's own test.
//!
//! These are the producer pins: the catalog proves the two texts, these prove
//! that the function the app calls really words its answer in the language it
//! was given, and that the English answer is the one that was there before.

use gentle_core::helpers::{HelperStatus, HelpersSummary};
use gentle_core::i18n::Language;
use gentle_runtime::app_config::{workspace_status, WorkspaceStatus};
use gentle_runtime::helper_text::{status_label, summary_line};
use gentle_runtime::time_ago::time_ago;

#[test]
fn the_workspace_reason_is_worded_in_the_language_the_caller_holds() {
    let dir = tempfile::tempdir().unwrap();
    let gone = dir.path().join("removed");

    assert_eq!(
        workspace_status(&gone, Language::Es),
        WorkspaceStatus::Missing("Esta carpeta ya no existe.".to_owned())
    );
    assert_eq!(
        workspace_status(&gone, Language::En),
        WorkspaceStatus::Missing("This folder no longer exists.".to_owned()),
        "the English reason is the one the app already showed"
    );

    let file = dir.path().join("notes.txt");
    std::fs::write(&file, b"x").unwrap();
    assert_eq!(
        workspace_status(&file, Language::Es).reason(),
        Some("Esa ruta es un archivo, no una carpeta.")
    );
}

#[test]
fn the_helpers_panel_is_worded_in_the_language_the_caller_holds() {
    let summary = HelpersSummary {
        running: 1,
        queued: 2,
        waiting: 3,
        finished: 4,
    };

    assert_eq!(
        summary_line(Language::Es, &summary),
        "1 en ejecución · 2 en cola · 3 en espera · 4 terminados"
    );
    assert_eq!(
        summary_line(Language::En, &summary),
        "1 running · 2 queued · 3 waiting · 4 finished",
        "the English counters line is the one the app already showed"
    );
    assert_eq!(
        status_label(Language::Es, HelperStatus::Cancelled),
        "cancelado"
    );
    assert_eq!(
        status_label(Language::En, HelperStatus::Cancelled),
        "cancelled"
    );
}

#[test]
fn an_age_says_now_in_the_language_the_caller_holds() {
    const NOW: u64 = 1_790_000_000_000;

    assert_eq!(time_ago(Language::Es, NOW, NOW), "ahora");
    assert_eq!(time_ago(Language::En, NOW, NOW), "now");
    assert_eq!(
        time_ago(Language::Es, NOW, NOW - 3 * 60 * 60_000),
        "3h",
        "the unit suffixes are the same symbols in both languages"
    );
}
