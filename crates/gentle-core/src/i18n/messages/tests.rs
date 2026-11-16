//! The catalog's own pins for the words of removing a workspace (S1, S5, S7) and
//! of refusing a rename a chat has no file for: every new message exists in both
//! columns, the English one is the sentence the app's other working-chat refusals
//! already say, and the Spanish one is neutral.
//!
//! One test per message, so a column that went missing, an interpolation that
//! stopped happening, or an English line that crept into the Spanish column fails
//! here rather than in a window.

use super::*;

#[test]
fn the_working_refusal_of_a_removal_is_the_family_sentence_with_its_own_verb() {
    assert_eq!(
        remove_workspace_working_refusal(Language::En),
        "This chat is working. Stop it first, then remove the workspace.",
        "the same sentence Close chat and Delete say, ending in this action's verb"
    );
    assert_eq!(
        remove_workspace_working_refusal(Language::Es),
        "Este chat está trabajando. Detenlo primero, luego quita el espacio de trabajo."
    );
    assert!(
        remove_workspace_working_refusal(Language::Es).starts_with("Este chat"),
        "the Spanish column is Spanish, not the English one twice"
    );
}

#[test]
fn a_chat_with_no_session_file_is_refused_in_both_columns() {
    assert_eq!(
        rename_no_session_refusal(Language::En),
        "A chat with no session file yet has no name to change.",
        "the English column says what a fresh chat lacks, not what the disk did"
    );
    assert_eq!(
        rename_no_session_refusal(Language::Es),
        "Un chat sin archivo de sesión todavía no tiene un nombre que cambiar.",
        "the Spanish column is neutral Spanish, not the English one twice"
    );
    assert_ne!(
        rename_no_session_refusal(Language::En),
        rename_no_session_refusal(Language::Es),
        "a column that went missing would read as the other one"
    );
}

#[test]
fn a_removal_that_failed_names_the_reason_the_disk_gave() {
    assert_eq!(
        workspace_removal_failed(Language::En, "access is denied"),
        "the removal could not run: access is denied"
    );
    assert_eq!(
        workspace_removal_failed(Language::Es, "access is denied"),
        "no se pudo ejecutar la eliminación del espacio de trabajo: access is denied",
        "the reason travels as it arrived, only the sentence around it is worded"
    );
}
