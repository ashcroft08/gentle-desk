//! Every catalog message, in both languages (0.2.0, S3, D6).
//!
//! The English column is the pin: each expected string here is the one the code
//! said before the catalog existed, byte for byte, so a drift in either language
//! is a red test instead of a quiet change of the app's words. The Spanish
//! column is the behaviour this slice adds: neutral Spanish, no voseo and no
//! regionalisms.
//!
//! Nothing here reads the code's own output as its expectation: the literals are
//! written out, so a message that renames its words has to change this file,
//! which is the point.

use gentle_core::helpers::HelperStatus;
use gentle_core::i18n::{messages, Language};

/// Asserts one message in both languages, and that the two really differ.
fn pair(name: &str, english: &str, expected_english: &str, spanish: &str, expected_spanish: &str) {
    assert_eq!(
        english, expected_english,
        "{name}: the English text is the pin"
    );
    assert_eq!(spanish, expected_spanish, "{name}: the Spanish text");
    assert_ne!(
        english, spanish,
        "{name}: a message that reads the same in both languages is untranslated"
    );
}

/// The messages with no parameters of their own.
#[test]
fn every_message_without_parameters_carries_its_two_texts() {
    // The workspace folder's status (S3).
    pair(
        "workspace_folder_missing",
        messages::workspace_folder_missing(Language::En),
        "This folder no longer exists.",
        messages::workspace_folder_missing(Language::Es),
        "Esta carpeta ya no existe.",
    );
    pair(
        "workspace_not_a_folder",
        messages::workspace_not_a_folder(Language::En),
        "That path is a file, not a folder.",
        messages::workspace_not_a_folder(Language::Es),
        "Esa ruta es un archivo, no una carpeta.",
    );

    // The helpers panel's status labels.
    pair(
        "helper_status_queued",
        messages::helper_status_queued(Language::En),
        "queued",
        messages::helper_status_queued(Language::Es),
        "en cola",
    );
    pair(
        "helper_status_running",
        messages::helper_status_running(Language::En),
        "running",
        messages::helper_status_running(Language::Es),
        "en ejecución",
    );
    pair(
        "helper_status_waiting",
        messages::helper_status_waiting(Language::En),
        "waiting",
        messages::helper_status_waiting(Language::Es),
        "en espera",
    );
    pair(
        "helper_status_done",
        messages::helper_status_done(Language::En),
        "done",
        messages::helper_status_done(Language::Es),
        "terminado",
    );
    pair(
        "helper_status_failed",
        messages::helper_status_failed(Language::En),
        "failed",
        messages::helper_status_failed(Language::Es),
        "fallido",
    );
    pair(
        "helper_status_cancelled",
        messages::helper_status_cancelled(Language::En),
        "cancelled",
        messages::helper_status_cancelled(Language::Es),
        "cancelado",
    );

    // The chat runtime's own two refusals.
    pair(
        "chat_has_no_process_to_open",
        messages::chat_has_no_process_to_open(Language::En),
        "this chat has no process to open",
        messages::chat_has_no_process_to_open(Language::Es),
        "este chat no tiene ningún proceso que abrir",
    );
    pair(
        "chat_has_no_process_yet",
        messages::chat_has_no_process_yet(Language::En),
        "this chat has no process yet",
        messages::chat_has_no_process_yet(Language::Es),
        "este chat todavía no tiene proceso",
    );

    // Ages and countdowns.
    pair(
        "time_now",
        messages::time_now(Language::En),
        "now",
        messages::time_now(Language::Es),
        "ahora",
    );

    // The chats shell.
    pair(
        "chat_working_refusal",
        messages::chat_working_refusal(Language::En),
        "This chat is working. Stop it first",
        messages::chat_working_refusal(Language::Es),
        "Este chat está trabajando. Detenlo primero",
    );

    // A workspace name's rules.
    pair(
        "workspace_name_empty",
        messages::workspace_name_empty(Language::En),
        "A workspace name cannot be empty.",
        messages::workspace_name_empty(Language::Es),
        "El nombre de un espacio de trabajo no puede estar vacío.",
    );
    pair(
        "workspace_name_trailing_space",
        messages::workspace_name_trailing_space(Language::En),
        "A workspace name cannot end with a space.",
        messages::workspace_name_trailing_space(Language::Es),
        "El nombre de un espacio de trabajo no puede terminar con un espacio.",
    );
    pair(
        "workspace_name_path_separator",
        messages::workspace_name_path_separator(Language::En),
        "A workspace name cannot contain a path separator.",
        messages::workspace_name_path_separator(Language::Es),
        "El nombre de un espacio de trabajo no puede contener un separador de rutas.",
    );
    pair(
        "workspace_name_double_dot",
        messages::workspace_name_double_dot(Language::En),
        "A workspace name cannot contain two dots in a row.",
        messages::workspace_name_double_dot(Language::Es),
        "El nombre de un espacio de trabajo no puede contener dos puntos seguidos.",
    );
    pair(
        "workspace_name_forbidden_chars",
        messages::workspace_name_forbidden_chars(Language::En),
        "A workspace name cannot contain any of these characters: < > : \" | ? *",
        messages::workspace_name_forbidden_chars(Language::Es),
        "El nombre de un espacio de trabajo no puede contener ninguno de estos caracteres: < > : \" | ? *",
    );
    pair(
        "workspace_name_trailing_dot",
        messages::workspace_name_trailing_dot(Language::En),
        "A workspace name cannot end with a dot.",
        messages::workspace_name_trailing_dot(Language::Es),
        "El nombre de un espacio de trabajo no puede terminar con un punto.",
    );
    pair(
        "workspace_name_device",
        messages::workspace_name_device(Language::En),
        "That name is reserved by Windows.",
        messages::workspace_name_device(Language::Es),
        "Ese nombre está reservado por Windows.",
    );

    // The new folder on disk.
    pair(
        "new_folder_file_exists",
        messages::new_folder_file_exists(Language::En),
        "There is already a file with that name.",
        messages::new_folder_file_exists(Language::Es),
        "Ya existe un archivo con ese nombre.",
    );
    pair(
        "new_folder_not_empty",
        messages::new_folder_not_empty(Language::En),
        "That folder already holds files.",
        messages::new_folder_not_empty(Language::Es),
        "Esa carpeta ya contiene archivos.",
    );

    // The terminal window's labels.
    pair(
        "terminal_action_profiles",
        messages::terminal_action_profiles(Language::En),
        "Profiles",
        messages::terminal_action_profiles(Language::Es),
        "Perfiles",
    );
    pair(
        "terminal_action_login",
        messages::terminal_action_login(Language::En),
        "Sign in",
        messages::terminal_action_login(Language::Es),
        "Iniciar sesión",
    );
    pair(
        "terminal_action_logout",
        messages::terminal_action_logout(Language::En),
        "Sign out",
        messages::terminal_action_logout(Language::Es),
        "Cerrar sesión",
    );
    pair(
        "terminal_action_update",
        messages::terminal_action_update(Language::En),
        "Update",
        messages::terminal_action_update(Language::Es),
        "Actualizar",
    );
    pair(
        "terminal_action_update_extensions",
        messages::terminal_action_update_extensions(Language::En),
        "Update extensions",
        messages::terminal_action_update_extensions(Language::Es),
        "Actualizar extensiones",
    );
    pair(
        "terminal_action_install_usage_sources",
        messages::terminal_action_install_usage_sources(Language::En),
        "Install usage sources",
        messages::terminal_action_install_usage_sources(Language::Es),
        "Instalar fuentes de uso",
    );

    // The folder picker's two titles.
    pair(
        "choose_workspace_folder_title",
        messages::choose_workspace_folder_title(Language::En),
        "Choose a workspace folder",
        messages::choose_workspace_folder_title(Language::Es),
        "Elige una carpeta de espacio de trabajo",
    );
    pair(
        "choose_new_workspace_parent_title",
        messages::choose_new_workspace_parent_title(Language::En),
        "Choose where the new workspace folder goes",
        messages::choose_new_workspace_parent_title(Language::Es),
        "Elige dónde va la nueva carpeta del espacio de trabajo",
    );

    // The chat header and the rename refusal.
    pair(
        "new_chat_title",
        messages::new_chat_title(Language::En),
        "New chat",
        messages::new_chat_title(Language::Es),
        "Chat nuevo",
    );
    pair(
        "rename_working_refusal",
        messages::rename_working_refusal(Language::En),
        "This chat is working. Rename it when the turn ends.",
        messages::rename_working_refusal(Language::Es),
        "Este chat está trabajando. Cámbiale el nombre cuando termine el turno.",
    );
}

/// The subscription usage's reset countdown: a wait, or none at all.
#[test]
fn the_usage_countdown_reads_in_both_languages() {
    pair(
        "usage_resets_now",
        messages::usage_resets_now(Language::En),
        "resets now",
        messages::usage_resets_now(Language::Es),
        "reinicia ahora",
    );
    pair(
        "usage_resets_in",
        &messages::usage_resets_in(Language::En, "4h"),
        "resets in 4h",
        &messages::usage_resets_in(Language::Es, "4h"),
        "reinicia en 4h",
    );
    assert_eq!(
        messages::usage_resets_in(Language::En, messages::time_now(Language::En)),
        "resets in now",
        "the wait is carried whole, whatever the age rule drew"
    );
}

/// The helpers panel's counters line, with its four counts.
#[test]
fn the_helpers_counters_line_carries_its_four_counts() {
    pair(
        "helpers_summary",
        &messages::helpers_summary(Language::En, 1, 2, 3, 4),
        "1 running · 2 queued · 3 waiting · 4 finished",
        &messages::helpers_summary(Language::Es, 1, 2, 3, 4),
        "1 en ejecución · 2 en cola · 3 en espera · 4 terminados",
    );
}

/// A refused working chat: the rule, then this action's verb.
#[test]
fn each_working_refusal_ends_with_its_own_verb() {
    pair(
        "close_working_refusal",
        &messages::close_working_refusal(Language::En),
        "This chat is working. Stop it first, then close it.",
        &messages::close_working_refusal(Language::Es),
        "Este chat está trabajando. Detenlo primero, luego ciérralo.",
    );
    pair(
        "delete_working_refusal",
        &messages::delete_working_refusal(Language::En),
        "This chat is working. Stop it first, then delete it.",
        &messages::delete_working_refusal(Language::Es),
        "Este chat está trabajando. Detenlo primero, luego bórralo.",
    );
}

/// A death notice names the chat, and the child's own last words when it left
/// some: the reason travels as it arrived (D7).
#[test]
fn a_death_notice_names_the_chat_and_the_reason() {
    pair(
        "death_notice",
        &messages::death_notice(Language::En, "a.jsonl"),
        "a.jsonl: the child that runs this chat exited. Restart to keep talking.",
        &messages::death_notice(Language::Es, "a.jsonl"),
        "a.jsonl: el proceso que ejecuta este chat terminó. Reinícialo para seguir hablando.",
    );
    pair(
        "death_notice_with_reason",
        &messages::death_notice_with_reason(Language::En, "a.jsonl", "cannot open the session"),
        "a.jsonl: the child that runs this chat exited (cannot open the session). Restart to keep talking.",
        &messages::death_notice_with_reason(Language::Es, "a.jsonl", "cannot open the session"),
        "a.jsonl: el proceso que ejecuta este chat terminó (cannot open the session). Reinícialo para seguir hablando.",
    );
}

/// A failed command names the command, which is the shell's own id and stays as
/// it arrives (D7), and the error beside it.
#[test]
fn a_failed_command_names_the_command_and_the_error() {
    pair(
        "command_failed",
        &messages::command_failed(Language::En, "get_available_models", "boom"),
        "get_available_models failed: boom",
        &messages::command_failed(Language::Es, "get_available_models", "boom"),
        "get_available_models falló: boom",
    );
}

/// The workspace row a delete leaves empty is named when it goes.
#[test]
fn a_workspace_that_lost_its_last_chat_is_named() {
    pair(
        "workspace_gone",
        &messages::workspace_gone(Language::En, "proj"),
        "proj has no chats left, so its workspace row is gone.",
        &messages::workspace_gone(Language::Es, "proj"),
        "proj ya no tiene chats, así que su fila de espacio de trabajo desapareció.",
    );
}

/// The terminal window's four refusals, each carrying what it names.
#[test]
fn every_terminal_refusal_carries_what_it_names() {
    pair(
        "terminal_shell_not_found",
        &messages::terminal_shell_not_found(Language::En, "x"),
        "The terminal could not be opened: gentle-shell could not be found (x)",
        &messages::terminal_shell_not_found(Language::Es, "x"),
        "No se pudo abrir la terminal: no se encontró gentle-shell (x)",
    );
    pair(
        "terminal_part_has_quote",
        &messages::terminal_part_has_quote(Language::En, "C:/a\"b.mjs"),
        "The terminal could not be opened: C:/a\"b.mjs carries a double quote, which a cmd line cannot pass on",
        &messages::terminal_part_has_quote(Language::Es, "C:/a\"b.mjs"),
        "No se pudo abrir la terminal: C:/a\"b.mjs contiene una comilla doble, que una línea de cmd no puede transmitir",
    );
    pair(
        "terminal_wrapper_unwritable",
        &messages::terminal_wrapper_unwritable(Language::En, "C:/t/w.cmd", "denied"),
        "The terminal could not be opened: C:/t/w.cmd could not be written (denied)",
        &messages::terminal_wrapper_unwritable(Language::Es, "C:/t/w.cmd", "denied"),
        "No se pudo abrir la terminal: no se pudo escribir C:/t/w.cmd (denied)",
    );
    pair(
        "terminal_open_failed",
        &messages::terminal_open_failed(Language::En, "boom"),
        "The terminal could not be opened: boom",
        &messages::terminal_open_failed(Language::Es, "boom"),
        "No se pudo abrir la terminal: boom",
    );
}

/// The three refusals of making a folder, each carrying the disk's own reason.
#[test]
fn every_new_folder_refusal_carries_the_disks_reason() {
    pair(
        "new_folder_create_failed",
        &messages::new_folder_create_failed(Language::En, "denied"),
        "The folder could not be created: denied",
        &messages::new_folder_create_failed(Language::Es, "denied"),
        "No se pudo crear la carpeta: denied",
    );
    pair(
        "new_folder_path_unreadable",
        &messages::new_folder_path_unreadable(Language::En, "denied"),
        "That path could not be read: denied",
        &messages::new_folder_path_unreadable(Language::Es, "denied"),
        "No se pudo leer esa ruta: denied",
    );
    pair(
        "new_folder_read_failed",
        &messages::new_folder_read_failed(Language::En, "denied"),
        "That folder could not be read: denied",
        &messages::new_folder_read_failed(Language::Es, "denied"),
        "No se pudo leer esa carpeta: denied",
    );
}

/// The six sentences a refused image reaches the user with, each naming the file
/// and, where it matters, the limit.
#[test]
fn every_image_refusal_names_the_file_it_refused() {
    pair(
        "image_not_an_image",
        &messages::image_not_an_image(Language::En, "a.txt"),
        "a.txt was not attached: only png, jpg, jpeg, webp and gif images can be.",
        &messages::image_not_an_image(Language::Es, "a.txt"),
        "a.txt no se adjuntó: solo se pueden adjuntar imágenes png, jpg, jpeg, webp y gif.",
    );
    pair(
        "image_missing",
        &messages::image_missing(Language::En, "a.txt"),
        "a.txt was not attached: the file is no longer there.",
        &messages::image_missing(Language::Es, "a.txt"),
        "a.txt no se adjuntó: el archivo ya no está.",
    );
    pair(
        "image_unreadable",
        &messages::image_unreadable(Language::En, "a.txt"),
        "a.txt was not attached: the file cannot be read.",
        &messages::image_unreadable(Language::Es, "a.txt"),
        "a.txt no se adjuntó: no se puede leer el archivo.",
    );
    pair(
        "image_is_folder",
        &messages::image_is_folder(Language::En, "a.txt"),
        "a.txt was not attached: it is a folder, not an image file.",
        &messages::image_is_folder(Language::Es, "a.txt"),
        "a.txt no se adjuntó: es una carpeta, no un archivo de imagen.",
    );
    pair(
        "image_count",
        &messages::image_count(Language::En, "a.png", 8),
        "a.png was not attached: a message holds at most 8 images.",
        &messages::image_count(Language::Es, "a.png", 8),
        "a.png no se adjuntó: un mensaje admite como máximo 8 imágenes.",
    );
    pair(
        "image_weight",
        &messages::image_weight(Language::En, "a.png", 64),
        "a.png was not attached: a message holds at most 64 MB of images.",
        &messages::image_weight(Language::Es, "a.png", 64),
        "a.png no se adjuntó: un mensaje admite como máximo 64 MB de imágenes.",
    );
}

/// A command that could not even run says why, in the shell's own words.
#[test]
fn a_command_that_could_not_run_says_why() {
    pair(
        "delete_failed",
        &messages::delete_failed(Language::En, "denied"),
        "the delete could not run: denied",
        &messages::delete_failed(Language::Es, "denied"),
        "no se pudo ejecutar la eliminación: denied",
    );
    pair(
        "shell_not_available",
        messages::shell_not_available(Language::En),
        "gentle-shell is not available",
        messages::shell_not_available(Language::Es),
        "gentle-shell no está disponible",
    );
}

/// A rename that could not run says why, in the shell's own words.
#[test]
fn a_rename_that_could_not_run_says_why() {
    pair(
        "rename_failed",
        &messages::rename_failed(Language::En, "denied"),
        "the rename could not run: denied",
        &messages::rename_failed(Language::Es, "denied"),
        "no se pudo ejecutar el cambio de nombre: denied",
    );
}

/// Every status of the helpers panel reaches one of the six labels, in the
/// language it was asked in.
#[test]
fn every_helper_status_reaches_a_label_in_the_asked_language() {
    let statuses = [
        (HelperStatus::Queued, "queued", "en cola"),
        (HelperStatus::Running, "running", "en ejecución"),
        (HelperStatus::Waiting, "waiting", "en espera"),
        (HelperStatus::Done, "done", "terminado"),
        (HelperStatus::Failed, "failed", "fallido"),
        (HelperStatus::Cancelled, "cancelled", "cancelado"),
    ];
    for (status, english, spanish) in statuses {
        assert_eq!(messages::helper_status_label(Language::En, status), english);
        assert_eq!(
            messages::helper_status_label(Language::Es, status),
            spanish,
            "{status:?} keeps its own label"
        );
    }
}

/// A sentence the shell composes from a catalog message keeps its join: the
/// prefix the catalog holds is the prefix the older constant spelled.
#[test]
fn a_composed_sentence_keeps_its_join() {
    let refusal = messages::chat_working_refusal(Language::En);
    assert_eq!(
        format!("{refusal}, then close it."),
        messages::close_working_refusal(Language::En),
        "Close chat builds the sentence the catalog holds whole"
    );
    assert_eq!(
        format!("{refusal}, then delete it."),
        messages::delete_working_refusal(Language::En),
        "Delete chat builds the sentence the catalog holds whole"
    );
}
