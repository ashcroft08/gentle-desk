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
        "terminal_action_upgrade_shell",
        messages::terminal_action_upgrade_shell(Language::En),
        "Update Gentle Shell",
        messages::terminal_action_upgrade_shell(Language::Es),
        "Actualizar Gentle Shell",
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

// --- T4b: the audit's twelve producer groups (0.2.0, S3, Log L7) ------------

/// A save that failed names the disk's own reason, worded in the language the
/// snapshot it tried to write carried.
#[test]
fn a_failed_config_save_carries_the_disk_reason() {
    pair(
        "config_save_failed",
        &messages::config_save_failed(Language::En, "Access is denied. (os error 5)"),
        "The app config file could not be saved: Access is denied. (os error 5)",
        &messages::config_save_failed(Language::Es, "Access is denied. (os error 5)"),
        "No se pudo guardar el archivo de configuración de la app: Access is denied. (os error 5)",
    );
}

/// Installing gentle-shell: the permission screen's start refusal, then every
/// way the command itself can end without an install.
#[test]
fn the_install_says_why_it_did_not_install() {
    pair(
        "install_could_not_start",
        messages::install_could_not_start(Language::En),
        "could not start the install",
        messages::install_could_not_start(Language::Es),
        "no se pudo iniciar la instalación",
    );
    pair(
        "install_command_could_not_start",
        &messages::install_command_could_not_start(Language::En, "npm.cmd", "denied"),
        "could not start npm.cmd: denied",
        &messages::install_command_could_not_start(Language::Es, "npm.cmd", "denied"),
        "no se pudo iniciar npm.cmd: denied",
    );
    pair(
        "install_did_not_finish",
        messages::install_did_not_finish(Language::En),
        "the install did not finish in 20 minutes and was stopped",
        messages::install_did_not_finish(Language::Es),
        "la instalación no terminó en 20 minutos y se detuvo",
    );
    pair(
        "install_wait_failed",
        &messages::install_wait_failed(Language::En, "denied"),
        "could not wait for the install: denied",
        &messages::install_wait_failed(Language::Es, "denied"),
        "no se pudo esperar a la instalación: denied",
    );
    // The exit code is one message for both things that need it: the install
    // and the usage sources' run.
    pair(
        "no_exit_code",
        messages::no_exit_code(Language::En),
        "no exit code",
        messages::no_exit_code(Language::Es),
        "sin código de salida",
    );
    pair(
        "exit_code",
        &messages::exit_code(Language::En, 2),
        "exit code 2",
        &messages::exit_code(Language::Es, 2),
        "código de salida 2",
    );
    pair(
        "install_ended",
        &messages::install_ended(
            Language::En,
            &messages::exit_code(Language::En, 2),
            "npm said no",
        ),
        "the install ended with exit code 2\nnpm said no",
        &messages::install_ended(
            Language::Es,
            &messages::exit_code(Language::Es, 2),
            "npm said no",
        ),
        "la instalación terminó con código de salida 2\nnpm said no",
    );
}

/// The Settings dialog's reads: a file, the launcher, and the two folders a
/// pane needs before it can read anything at all.
#[test]
fn the_settings_reads_carry_their_reason() {
    pair(
        "file_could_not_be_read",
        &messages::file_could_not_be_read(
            Language::En,
            "C:/h/auth.json",
            "Access is denied. (os error 5)",
        ),
        "C:/h/auth.json could not be read: Access is denied. (os error 5)",
        &messages::file_could_not_be_read(
            Language::Es,
            "C:/h/auth.json",
            "Access is denied. (os error 5)",
        ),
        "no se pudo leer C:/h/auth.json: Access is denied. (os error 5)",
    );
    pair(
        "shell_could_not_be_resolved",
        &messages::shell_could_not_be_resolved(Language::En, "not found"),
        "gentle-shell could not be resolved: not found",
        &messages::shell_could_not_be_resolved(Language::Es, "not found"),
        "no se pudo resolver gentle-shell: not found",
    );
    pair(
        "shell_could_not_run",
        &messages::shell_could_not_run(Language::En, "denied"),
        "gentle-shell could not run: denied",
        &messages::shell_could_not_run(Language::Es, "denied"),
        "no se pudo ejecutar gentle-shell: denied",
    );
    pair(
        "shell_agent_folder_missing",
        messages::shell_agent_folder_missing(Language::En),
        "the shell's agent folder could not be found on this machine",
        messages::shell_agent_folder_missing(Language::Es),
        "no se encontró la carpeta del agente de la shell en esta máquina",
    );
    pair(
        "agent_profiles_folder_missing",
        messages::agent_profiles_folder_missing(Language::En),
        "gentle-pi's config folder could not be found on this machine",
        messages::agent_profiles_folder_missing(Language::Es),
        "no se encontró la carpeta de configuración de gentle-pi en esta máquina",
    );
}

/// The usage sources: the package that is not installed, the read that never
/// started, and every way the run itself ends without a document.
#[test]
fn the_usage_run_says_why_it_has_no_document() {
    pair(
        "usage_sources_not_installed",
        messages::usage_sources_not_installed(Language::En),
        "the usage sources are not installed for this shell",
        messages::usage_sources_not_installed(Language::Es),
        "las fuentes de uso no están instaladas para esta shell",
    );
    pair(
        "usage_read_could_not_start",
        messages::usage_read_could_not_start(Language::En),
        "the usage read could not start",
        messages::usage_read_could_not_start(Language::Es),
        "no se pudo iniciar la lectura de uso",
    );
    pair(
        "usage_sources_could_not_start",
        &messages::usage_sources_could_not_start(Language::En, "denied"),
        "could not start (denied)",
        &messages::usage_sources_could_not_start(Language::Es, "denied"),
        "no se pudo iniciar (denied)",
    );
    pair(
        "usage_sources_did_not_answer",
        &messages::usage_sources_did_not_answer(Language::En, 30),
        "did not answer within 30 seconds",
        &messages::usage_sources_did_not_answer(Language::Es, 30),
        "no respondió en 30 segundos",
    );
    pair(
        "usage_sources_could_not_run",
        &messages::usage_sources_could_not_run(Language::En, "denied"),
        "could not run (denied)",
        &messages::usage_sources_could_not_run(Language::Es, "denied"),
        "no se pudo ejecutar (denied)",
    );
    pair(
        "usage_sources_stopped",
        &messages::usage_sources_stopped(Language::En, "exit code 1"),
        "stopped without printing a document (exit code 1)",
        &messages::usage_sources_stopped(Language::Es, "exit code 1"),
        "se detuvo sin imprimir un documento (exit code 1)",
    );
}

/// The usage document's refusals: one whole sentence each, so the Spanish reads
/// as a sentence and not as an English clause inside a translated frame.
#[test]
fn every_usage_document_refusal_is_one_whole_sentence() {
    pair(
        "usage_sources_schema",
        &messages::usage_sources_schema(Language::En, "pi-usage-sources/usage/v1"),
        "the usage sources answered schema \"pi-usage-sources/usage/v1\" this pane does not read",
        &messages::usage_sources_schema(Language::Es, "pi-usage-sources/usage/v1"),
        "las fuentes de uso respondieron el esquema \"pi-usage-sources/usage/v1\", que este panel no lee",
    );
    pair(
        "usage_sources_status",
        &messages::usage_sources_status(Language::En, "paused"),
        "the usage sources answered status \"paused\" this pane does not read",
        &messages::usage_sources_status(Language::Es, "paused"),
        "las fuentes de uso respondieron el estado \"paused\", que este panel no lee",
    );
    pair(
        "usage_sources_no_provider_list",
        messages::usage_sources_no_provider_list(Language::En),
        "the usage sources answered a document with no provider list this pane does not read",
        messages::usage_sources_no_provider_list(Language::Es),
        "las fuentes de uso respondieron un documento sin lista de proveedores, que este panel no lee",
    );
    pair(
        "usage_sources_provider_without_name",
        &messages::usage_sources_provider_without_name(Language::En, 2),
        "the usage sources answered provider 2 with no name this pane does not read",
        &messages::usage_sources_provider_without_name(Language::Es, 2),
        "las fuentes de uso respondieron un proveedor 2 sin nombre, que este panel no lee",
    );
    pair(
        "usage_sources_provider_without_limits",
        &messages::usage_sources_provider_without_limits(Language::En, "claude-bridge"),
        "the usage sources answered provider claude-bridge with no limit list this pane does not read",
        &messages::usage_sources_provider_without_limits(Language::Es, "claude-bridge"),
        "las fuentes de uso respondieron un proveedor claude-bridge sin lista de límites, que este panel no lee",
    );
    pair(
        "usage_sources_limit_without_name",
        &messages::usage_sources_limit_without_name(Language::En, "claude-bridge"),
        "the usage sources answered a limit of claude-bridge with no name this pane does not read",
        &messages::usage_sources_limit_without_name(Language::Es, "claude-bridge"),
        "las fuentes de uso respondieron un límite de claude-bridge sin nombre, que este panel no lee",
    );
    pair(
        "usage_sources_limit_without_reach_mark",
        &messages::usage_sources_limit_without_reach_mark(Language::En, "claude-bridge"),
        "the usage sources answered a limit of claude-bridge with no reach mark this pane does not read",
        &messages::usage_sources_limit_without_reach_mark(Language::Es, "claude-bridge"),
        "las fuentes de uso respondieron un límite de claude-bridge sin marca de alcanzado, que este panel no lee",
    );
    pair(
        "usage_sources_limit_without_windows",
        &messages::usage_sources_limit_without_windows(Language::En, 1, "claude-bridge"),
        "the usage sources answered limit 1 of claude-bridge with no windows this pane does not read",
        &messages::usage_sources_limit_without_windows(Language::Es, 1, "claude-bridge"),
        "las fuentes de uso respondieron el límite 1 de claude-bridge sin ventanas, que este panel no lee",
    );
    pair(
        "usage_sources_window_without_label",
        &messages::usage_sources_window_without_label(Language::En, "claude-bridge", "week"),
        "the usage sources answered a window of claude-bridge (week) with no label this pane does not read",
        &messages::usage_sources_window_without_label(Language::Es, "claude-bridge", "week"),
        "las fuentes de uso respondieron una ventana de claude-bridge (week) sin etiqueta, que este panel no lee",
    );
    pair(
        "usage_sources_window_without_percentage",
        &messages::usage_sources_window_without_percentage(Language::En, "claude-bridge", "week"),
        "the usage sources answered a window of claude-bridge (week) with no percentage this pane does not read",
        &messages::usage_sources_window_without_percentage(Language::Es, "claude-bridge", "week"),
        "las fuentes de uso respondieron una ventana de claude-bridge (week) sin porcentaje, que este panel no lee",
    );
    pair(
        "usage_sources_without_reset_time",
        &messages::usage_sources_without_reset_time(Language::En, "claude-bridge"),
        "the usage sources answered a reset time of claude-bridge this pane does not read",
        &messages::usage_sources_without_reset_time(Language::Es, "claude-bridge"),
        "las fuentes de uso respondieron una hora de reinicio de claude-bridge, que este panel no lee",
    );
}

/// The two profiles files: a read that failed, a document of another kind or
/// version, and the three names gentle-pi's own rules refuse.
#[test]
fn every_profiles_file_refusal_says_which_file_and_why() {
    pair(
        "profiles_file_unreadable",
        &messages::profiles_file_unreadable(Language::En, "profiles.json", "denied"),
        "profiles.json is not readable: denied",
        &messages::profiles_file_unreadable(Language::Es, "profiles.json", "denied"),
        "no se puede leer profiles.json: denied",
    );
    pair(
        "profiles_not_a_document",
        &messages::profiles_not_a_document(
            Language::En,
            "profile-fallbacks.json",
            "gentle-pi.profile_model_fallbacks",
            "other.kind",
        ),
        "profile-fallbacks.json is not a gentle-pi.profile_model_fallbacks document, it says \"other.kind\"",
        &messages::profiles_not_a_document(
            Language::Es,
            "profile-fallbacks.json",
            "gentle-pi.profile_model_fallbacks",
            "other.kind",
        ),
        "profile-fallbacks.json no es un documento gentle-pi.profile_model_fallbacks, dice \"other.kind\"",
    );
    pair(
        "profiles_wrong_version",
        &messages::profiles_wrong_version(Language::En, "profiles.json", 2, 1),
        "profiles.json is version 2, this view reads version 1",
        &messages::profiles_wrong_version(Language::Es, "profiles.json", 2, 1),
        "profiles.json tiene la versión 2, esta vista lee la versión 1",
    );
    pair(
        "profiles_name_refused",
        &messages::profiles_name_refused(Language::En, "profile-fallbacks.json", "bad name"),
        "profile-fallbacks.json names a profile no reader accepts: \"bad name\"",
        &messages::profiles_name_refused(Language::Es, "profile-fallbacks.json", "bad name"),
        "profile-fallbacks.json nombra un perfil que ningún lector acepta: \"bad name\"",
    );
    pair(
        "profiles_agent_refused",
        &messages::profiles_agent_refused(
            Language::En,
            "profile-fallbacks.json",
            "work",
            "bad agent",
        ),
        "profile-fallbacks.json: \"work\" names the agent \"bad agent\"",
        &messages::profiles_agent_refused(
            Language::Es,
            "profile-fallbacks.json",
            "work",
            "bad agent",
        ),
        "profile-fallbacks.json: \"work\" nombra el agente \"bad agent\"",
    );
    pair(
        "profiles_not_a_chain",
        &messages::profiles_not_a_chain(Language::En, "profile-fallbacks.json", "work", "agent"),
        "profile-fallbacks.json: \"work\"/\"agent\" is not a chain",
        &messages::profiles_not_a_chain(Language::Es, "profile-fallbacks.json", "work", "agent"),
        "profile-fallbacks.json: \"work\"/\"agent\" no es una cadena",
    );
}

/// The two words the app adds to a provider's own error, and the line a command
/// answers once the window is closing.
#[test]
fn the_rest_of_the_apps_own_words_read_in_both_languages() {
    pair(
        "unknown_model_error",
        messages::unknown_model_error(Language::En),
        "unknown model error",
        messages::unknown_model_error(Language::Es),
        "error desconocido del modelo",
    );
    pair(
        "closing",
        messages::closing(Language::En),
        "the app is closing",
        messages::closing(Language::Es),
        "la app se está cerrando",
    );
}
