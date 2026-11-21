//! The app's own words, in the two languages it speaks (0.2.0, S3, D6).
//!
//! One function per message, taking the [`Language`] first and its parameters
//! after, so a caller reads as the sentence it produces:
//! `messages::workspace_added(language, name)`. The English column is the text
//! the app already said, byte for byte, and the Spanish one is neutral: no
//! voseo and no regionalisms.
//!
//! Each function is localized WHERE IT IS PRODUCED, so nothing here is a lookup
//! by a code that crossed a boundary: the caller holds the language (it read the
//! config) and answers the final string. Only the app's own notices, errors,
//! validations and labels live here; everything that comes from the shell or pi
//! — session titles, model names, shell output, error text propagated from a
//! child — is shown exactly as it arrives (D7) and is never a message.
//!
//! The functions that take no parameters answer `&'static str` and are `const`,
//! so a constant the app already exported can be defined from the catalog
//! instead of spelling the same English twice. The ones that interpolate a name,
//! a count, a path or a reason answer `String`.
//!
//! Two strings of gentle-desk's own are deliberately NOT here:
//!
//! - the damaged-config notice (`ConfigNotice::message`), which stays in English:
//!   it describes the config file that could not be read, and that file is the
//!   one that would have named the preference, so no language exists to word it
//!   in.
//! - `Attached image: <path>`, which travels INSIDE the prompt handed to pi:
//!   translating it would change the model's input, so it is prompt content and
//!   not a message (D7).
//!
//! One more joins them for the same reason as the damaged config (0.2.0, T4b,
//! Log L7):
//!
//! - the no-config-path notice (`config::NO_PATH_NOTICE`), which says the
//!   settings folder could not be found. That folder is the one that would have
//!   held the preference, so the run that reports it has no language to report
//!   it in.

use super::Language;
use crate::helpers::HelperStatus;

// --- The folder of a workspace (runtime::app_config::status) -----------------

/// "This folder no longer exists."
pub const fn workspace_folder_missing(language: Language) -> &'static str {
    match language {
        Language::En => "This folder no longer exists.",
        Language::Es => "Esta carpeta ya no existe.",
    }
}

/// "That path is a file, not a folder."
pub const fn workspace_not_a_folder(language: Language) -> &'static str {
    match language {
        Language::En => "That path is a file, not a folder.",
        Language::Es => "Esa ruta es un archivo, no una carpeta.",
    }
}

// --- The helpers panel (runtime::helper_text) -------------------------------

/// "1 running · 0 queued · 0 waiting · 2 finished"
pub fn helpers_summary(
    language: Language,
    running: u64,
    queued: u64,
    waiting: u64,
    finished: u64,
) -> String {
    match language {
        Language::En => {
            format!("{running} running · {queued} queued · {waiting} waiting · {finished} finished")
        }
        Language::Es => format!(
            "{running} en ejecución · {queued} en cola · {waiting} en espera · \
             {finished} terminados"
        ),
    }
}

/// "queued"
pub const fn helper_status_queued(language: Language) -> &'static str {
    match language {
        Language::En => "queued",
        Language::Es => "en cola",
    }
}

/// "running"
pub const fn helper_status_running(language: Language) -> &'static str {
    match language {
        Language::En => "running",
        Language::Es => "en ejecución",
    }
}

/// "waiting"
pub const fn helper_status_waiting(language: Language) -> &'static str {
    match language {
        Language::En => "waiting",
        Language::Es => "en espera",
    }
}

/// "done"
pub const fn helper_status_done(language: Language) -> &'static str {
    match language {
        Language::En => "done",
        Language::Es => "terminado",
    }
}

/// "failed"
pub const fn helper_status_failed(language: Language) -> &'static str {
    match language {
        Language::En => "failed",
        Language::Es => "fallido",
    }
}

/// "cancelled"
pub const fn helper_status_cancelled(language: Language) -> &'static str {
    match language {
        Language::En => "cancelled",
        Language::Es => "cancelado",
    }
}

/// The label of any helper status, which is the same six messages above.
pub const fn helper_status_label(language: Language, status: HelperStatus) -> &'static str {
    match status {
        HelperStatus::Queued => helper_status_queued(language),
        HelperStatus::Running => helper_status_running(language),
        HelperStatus::Waiting => helper_status_waiting(language),
        HelperStatus::Done => helper_status_done(language),
        HelperStatus::Failed => helper_status_failed(language),
        HelperStatus::Cancelled => helper_status_cancelled(language),
    }
}

// --- Running one chat command (runtime::chat_runtime) -----------------------

/// "this chat has no process to open"
pub const fn chat_has_no_process_to_open(language: Language) -> &'static str {
    match language {
        Language::En => "this chat has no process to open",
        Language::Es => "este chat no tiene ningún proceso que abrir",
    }
}

/// "this chat has no process yet"
pub const fn chat_has_no_process_yet(language: Language) -> &'static str {
    match language {
        Language::En => "this chat has no process yet",
        Language::Es => "este chat todavía no tiene proceso",
    }
}

// --- Ages and countdowns (runtime::time_ago, runtime::usage) ----------------

/// "now": an age or a wait that is not a minute long yet, or already passed.
pub const fn time_now(language: Language) -> &'static str {
    match language {
        Language::En => "now",
        Language::Es => "ahora",
    }
}

/// "resets now": a subscription window whose reset time has already passed.
pub const fn usage_resets_now(language: Language) -> &'static str {
    match language {
        Language::En => "resets now",
        Language::Es => "reinicia ahora",
    }
}

/// "resets in 4h": a subscription window, with the wait the age rule drew.
pub fn usage_resets_in(language: Language, wait: &str) -> String {
    match language {
        Language::En => format!("resets in {wait}"),
        Language::Es => format!("reinicia en {wait}"),
    }
}

// --- The chats shell (app::chats) -------------------------------------------

/// "This chat is working. Stop it first"
pub const fn chat_working_refusal(language: Language) -> &'static str {
    match language {
        Language::En => "This chat is working. Stop it first",
        Language::Es => "Este chat está trabajando. Detenlo primero",
    }
}

/// "This chat is working. Stop it first, then close it."
pub fn close_working_refusal(language: Language) -> String {
    match language {
        Language::En => "This chat is working. Stop it first, then close it.".to_owned(),
        Language::Es => "Este chat está trabajando. Detenlo primero, luego ciérralo.".to_owned(),
    }
}

/// "This chat is working. Stop it first, then delete it."
pub fn delete_working_refusal(language: Language) -> String {
    match language {
        Language::En => "This chat is working. Stop it first, then delete it.".to_owned(),
        Language::Es => "Este chat está trabajando. Detenlo primero, luego bórralo.".to_owned(),
    }
}

/// "a.jsonl: the child that runs this chat exited. Restart to keep talking."
pub fn death_notice(language: Language, chat: &str) -> String {
    match language {
        Language::En => {
            format!("{chat}: the child that runs this chat exited. Restart to keep talking.")
        }
        Language::Es => {
            format!("{chat}: el proceso que ejecuta este chat terminó. Reinícialo para seguir hablando.")
        }
    }
}

/// "a.jsonl: the child that runs this chat exited (cannot open the session).
/// Restart to keep talking." The reason is the child's own last words and
/// travels as it arrived (D7).
pub fn death_notice_with_reason(language: Language, chat: &str, reason: &str) -> String {
    match language {
        Language::En => format!(
            "{chat}: the child that runs this chat exited ({reason}). Restart to keep talking."
        ),
        Language::Es => format!(
            "{chat}: el proceso que ejecuta este chat terminó ({reason}). Reinícialo para seguir hablando."
        ),
    }
}

/// "get_available_models failed: boom": the command is the shell's own id and
/// stays as it arrives (D7); the error beside it too.
pub fn command_failed(language: Language, command: &str, error: &str) -> String {
    match language {
        Language::En => format!("{command} failed: {error}"),
        Language::Es => format!("{command} falló: {error}"),
    }
}

// --- A new workspace's folder (tauri: folder_name, new_folder) --------------

/// "A workspace name cannot be empty."
pub const fn workspace_name_empty(language: Language) -> &'static str {
    match language {
        Language::En => "A workspace name cannot be empty.",
        Language::Es => "El nombre de un espacio de trabajo no puede estar vacío.",
    }
}

/// "A workspace name cannot end with a space."
pub const fn workspace_name_trailing_space(language: Language) -> &'static str {
    match language {
        Language::En => "A workspace name cannot end with a space.",
        Language::Es => "El nombre de un espacio de trabajo no puede terminar con un espacio.",
    }
}

/// "A workspace name cannot contain a path separator."
pub const fn workspace_name_path_separator(language: Language) -> &'static str {
    match language {
        Language::En => "A workspace name cannot contain a path separator.",
        Language::Es => {
            "El nombre de un espacio de trabajo no puede contener un separador de rutas."
        }
    }
}

/// "A workspace name cannot contain two dots in a row."
pub const fn workspace_name_double_dot(language: Language) -> &'static str {
    match language {
        Language::En => "A workspace name cannot contain two dots in a row.",
        Language::Es => "El nombre de un espacio de trabajo no puede contener dos puntos seguidos.",
    }
}

/// "A workspace name cannot contain any of these characters: < > : \" | ? *"
pub const fn workspace_name_forbidden_chars(language: Language) -> &'static str {
    match language {
        Language::En => "A workspace name cannot contain any of these characters: < > : \" | ? *",
        Language::Es => {
            "El nombre de un espacio de trabajo no puede contener ninguno de estos caracteres: \
             < > : \" | ? *"
        }
    }
}

/// "A workspace name cannot end with a dot."
pub const fn workspace_name_trailing_dot(language: Language) -> &'static str {
    match language {
        Language::En => "A workspace name cannot end with a dot.",
        Language::Es => "El nombre de un espacio de trabajo no puede terminar con un punto.",
    }
}

/// "That name is reserved by Windows."
pub const fn workspace_name_device(language: Language) -> &'static str {
    match language {
        Language::En => "That name is reserved by Windows.",
        Language::Es => "Ese nombre está reservado por Windows.",
    }
}

/// "There is already a file with that name."
pub const fn new_folder_file_exists(language: Language) -> &'static str {
    match language {
        Language::En => "There is already a file with that name.",
        Language::Es => "Ya existe un archivo con ese nombre.",
    }
}

/// "That folder already holds files."
pub const fn new_folder_not_empty(language: Language) -> &'static str {
    match language {
        Language::En => "That folder already holds files.",
        Language::Es => "Esa carpeta ya contiene archivos.",
    }
}

/// "The folder could not be created: Access is denied. (os error 5)"
pub fn new_folder_create_failed(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("The folder could not be created: {error}"),
        Language::Es => format!("No se pudo crear la carpeta: {error}"),
    }
}

/// "That path could not be read: Access is denied. (os error 5)"
pub fn new_folder_path_unreadable(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("That path could not be read: {error}"),
        Language::Es => format!("No se pudo leer esa ruta: {error}"),
    }
}

/// "That folder could not be read: Access is denied. (os error 5)"
pub fn new_folder_read_failed(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("That folder could not be read: {error}"),
        Language::Es => format!("No se pudo leer esa carpeta: {error}"),
    }
}

// --- The folder picker's titles (tauri: workspace_add, workspace_create) ----

/// "Choose a workspace folder"
pub const fn choose_workspace_folder_title(language: Language) -> &'static str {
    match language {
        Language::En => "Choose a workspace folder",
        Language::Es => "Elige una carpeta de espacio de trabajo",
    }
}

/// "Choose where the new workspace folder goes"
pub const fn choose_new_workspace_parent_title(language: Language) -> &'static str {
    match language {
        Language::En => "Choose where the new workspace folder goes",
        Language::Es => "Elige dónde va la nueva carpeta del espacio de trabajo",
    }
}

// --- The terminal window (tauri: terminal) ----------------------------------

/// "The terminal could not be opened: gentle-shell could not be found (...)"
pub fn terminal_shell_not_found(language: Language, error: &str) -> String {
    match language {
        Language::En => {
            format!("The terminal could not be opened: gentle-shell could not be found ({error})")
        }
        Language::Es => {
            format!("No se pudo abrir la terminal: no se encontró gentle-shell ({error})")
        }
    }
}

/// "The terminal could not be opened: C:/a\"b.mjs carries a double quote, which
/// a cmd line cannot pass on"
pub fn terminal_part_has_quote(language: Language, part: &str) -> String {
    match language {
        Language::En => format!(
            "The terminal could not be opened: {part} carries a double quote, which a cmd line \
             cannot pass on"
        ),
        Language::Es => format!(
            "No se pudo abrir la terminal: {part} contiene una comilla doble, que una línea de \
             cmd no puede transmitir"
        ),
    }
}

/// "The terminal could not be opened: C:/tmp/w.cmd could not be written (...)"
pub fn terminal_wrapper_unwritable(language: Language, path: &str, error: &str) -> String {
    match language {
        Language::En => {
            format!("The terminal could not be opened: {path} could not be written ({error})")
        }
        Language::Es => {
            format!("No se pudo abrir la terminal: no se pudo escribir {path} ({error})")
        }
    }
}

/// "The terminal could not be opened: Access is denied. (os error 5)"
pub fn terminal_open_failed(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("The terminal could not be opened: {error}"),
        Language::Es => format!("No se pudo abrir la terminal: {error}"),
    }
}

/// "Profiles"
pub const fn terminal_action_profiles(language: Language) -> &'static str {
    match language {
        Language::En => "Profiles",
        Language::Es => "Perfiles",
    }
}

/// "Sign in"
pub const fn terminal_action_login(language: Language) -> &'static str {
    match language {
        Language::En => "Sign in",
        Language::Es => "Iniciar sesión",
    }
}

/// "Sign out"
pub const fn terminal_action_logout(language: Language) -> &'static str {
    match language {
        Language::En => "Sign out",
        Language::Es => "Cerrar sesión",
    }
}

/// "Update"
pub const fn terminal_action_update(language: Language) -> &'static str {
    match language {
        Language::En => "Update",
        Language::Es => "Actualizar",
    }
}

/// "Update extensions"
pub const fn terminal_action_update_extensions(language: Language) -> &'static str {
    match language {
        Language::En => "Update extensions",
        Language::Es => "Actualizar extensiones",
    }
}

/// "Actualizar Gentle Shell" / "Update Gentle Shell"
pub const fn terminal_action_upgrade_shell(language: Language) -> &'static str {
    match language {
        Language::En => "Update Gentle Shell",
        Language::Es => "Actualizar Gentle Shell",
    }
}

/// "Install usage sources"
pub const fn terminal_action_install_usage_sources(language: Language) -> &'static str {
    match language {
        Language::En => "Install usage sources",
        Language::Es => "Instalar fuentes de uso",
    }
}

// --- The chat list and the chat header (tauri: sessions::view) --------------

/// "New chat": the header's own words for a chat whose session file does not
/// exist yet, never the session's.
pub const fn new_chat_title(language: Language) -> &'static str {
    match language {
        Language::En => "New chat",
        Language::Es => "Chat nuevo",
    }
}

// --- Renaming and deleting a chat (tauri: chat_rename, chat_delete) ---------

/// "This chat is working. Rename it when the turn ends."
pub const fn rename_working_refusal(language: Language) -> &'static str {
    match language {
        Language::En => "This chat is working. Rename it when the turn ends.",
        Language::Es => "Este chat está trabajando. Cámbiale el nombre cuando termine el turno.",
    }
}

/// "A chat with no session file yet has no name to change.": a fresh chat has
/// no file a name could be appended to, so the rename is refused in words
/// instead of reaching a disk that has no target for it.
pub const fn rename_no_session_refusal(language: Language) -> &'static str {
    match language {
        Language::En => "A chat with no session file yet has no name to change.",
        Language::Es => "Un chat sin archivo de sesión todavía no tiene un nombre que cambiar.",
    }
}

/// "the delete could not run: ..."
pub fn delete_failed(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("the delete could not run: {error}"),
        Language::Es => format!("no se pudo ejecutar la eliminación: {error}"),
    }
}

/// "gentle-shell is not available": no child can start, because the shell the
/// app runs was not found. The launcher's own reason is logged, never shown.
pub const fn shell_not_available(language: Language) -> &'static str {
    match language {
        Language::En => "gentle-shell is not available",
        Language::Es => "gentle-shell no está disponible",
    }
}

/// "the rename could not run: ..."
pub fn rename_failed(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("the rename could not run: {error}"),
        Language::Es => format!("no se pudo ejecutar el cambio de nombre: {error}"),
    }
}

/// "proj has no chats left, so its workspace row is gone."
pub fn workspace_gone(language: Language, workspace: &str) -> String {
    match language {
        Language::En => {
            format!("{workspace} has no chats left, so its workspace row is gone.")
        }
        Language::Es => format!(
            "{workspace} ya no tiene chats, así que su fila de espacio de trabajo desapareció."
        ),
    }
}

// --- Removing a workspace (tauri: workspace_remove) -------------------------

/// "This chat is working. Stop it first, then remove the workspace."
///
/// The same sentence Close chat and Delete a chat refuse with, ending in this
/// action's own verb (S1: a workspace with a chat mid-turn is not removed at
/// all, so the user has one thing to do first).
pub fn remove_workspace_working_refusal(language: Language) -> String {
    match language {
        Language::En => {
            "This chat is working. Stop it first, then remove the workspace.".to_owned()
        }
        Language::Es => {
            "Este chat está trabajando. Detenlo primero, luego quita el espacio de trabajo."
                .to_owned()
        }
    }
}

/// "the removal could not run: access is denied": the sentence is the app's,
/// the reason is the disk's and travels as it arrived (D7).
pub fn workspace_removal_failed(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("the removal could not run: {error}"),
        Language::Es => {
            format!("no se pudo ejecutar la eliminación del espacio de trabajo: {error}")
        }
    }
}

// --- The images a chat carries (tauri: images) ------------------------------

/// "a.txt was not attached: only png, jpg, jpeg, webp and gif images can be."
pub fn image_not_an_image(language: Language, name: &str) -> String {
    match language {
        Language::En => {
            format!("{name} was not attached: only png, jpg, jpeg, webp and gif images can be.")
        }
        Language::Es => {
            format!("{name} no se adjuntó: solo se pueden adjuntar imágenes png, jpg, jpeg, webp y gif.")
        }
    }
}

/// "a.txt was not attached: the file is no longer there."
pub fn image_missing(language: Language, name: &str) -> String {
    match language {
        Language::En => format!("{name} was not attached: the file is no longer there."),
        Language::Es => format!("{name} no se adjuntó: el archivo ya no está."),
    }
}

/// "a.txt was not attached: the file cannot be read."
pub fn image_unreadable(language: Language, name: &str) -> String {
    match language {
        Language::En => format!("{name} was not attached: the file cannot be read."),
        Language::Es => format!("{name} no se adjuntó: no se puede leer el archivo."),
    }
}

/// "a.png was not attached: it is a folder, not an image file."
pub fn image_is_folder(language: Language, name: &str) -> String {
    match language {
        Language::En => format!("{name} was not attached: it is a folder, not an image file."),
        Language::Es => format!("{name} no se adjuntó: es una carpeta, no un archivo de imagen."),
    }
}

/// "a.png was not attached: a message holds at most 8 images."
pub fn image_count(language: Language, name: &str, most: usize) -> String {
    match language {
        Language::En => {
            format!("{name} was not attached: a message holds at most {most} images.")
        }
        Language::Es => {
            format!("{name} no se adjuntó: un mensaje admite como máximo {most} imágenes.")
        }
    }
}

/// "a.png was not attached: a message holds at most 64 MB of images."
pub fn image_weight(language: Language, name: &str, megabytes: u64) -> String {
    match language {
        Language::En => {
            format!("{name} was not attached: a message holds at most {megabytes} MB of images.")
        }
        Language::Es => format!(
            "{name} no se adjuntó: un mensaje admite como máximo {megabytes} MB de imágenes."
        ),
    }
}

// --- A config that could not be saved (tauri: config::writer) ---------------

/// "The app config file could not be saved: Access is denied. (os error 5)"
///
/// Worded in the language of the snapshot the writer tried to save: the config
/// was read, so the preference is in hand (unlike the no-path notice above).
pub fn config_save_failed(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("The app config file could not be saved: {error}"),
        Language::Es => {
            format!("No se pudo guardar el archivo de configuración de la app: {error}")
        }
    }
}

// --- Installing gentle-shell (tauri: shell_install) -------------------------

/// "could not start the install": no thread could be started for it.
pub const fn install_could_not_start(language: Language) -> &'static str {
    match language {
        Language::En => "could not start the install",
        Language::Es => "no se pudo iniciar la instalación",
    }
}

/// "could not start npm.cmd: Access is denied. (os error 5)"
pub fn install_command_could_not_start(language: Language, program: &str, error: &str) -> String {
    match language {
        Language::En => format!("could not start {program}: {error}"),
        Language::Es => format!("no se pudo iniciar {program}: {error}"),
    }
}

/// "the install did not finish in 20 minutes and was stopped"
pub const fn install_did_not_finish(language: Language) -> &'static str {
    match language {
        Language::En => "the install did not finish in 20 minutes and was stopped",
        Language::Es => "la instalación no terminó en 20 minutos y se detuvo",
    }
}

/// "could not wait for the install: Access is denied. (os error 5)"
pub fn install_wait_failed(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("could not wait for the install: {error}"),
        Language::Es => format!("no se pudo esperar a la instalación: {error}"),
    }
}

/// "no exit code": a process a signal ended, which has no code to name.
pub const fn no_exit_code(language: Language) -> &'static str {
    match language {
        Language::En => "no exit code",
        Language::Es => "sin código de salida",
    }
}

/// "exit code 2"
pub fn exit_code(language: Language, code: i32) -> String {
    match language {
        Language::En => format!("exit code {code}"),
        Language::Es => format!("código de salida {code}"),
    }
}

/// "the install ended with exit code 2\nnpm said no": the code this catalog
/// worded, then the end of the command's own output, which travels as it
/// arrived (D7).
pub fn install_ended(language: Language, code: &str, tail: &str) -> String {
    match language {
        Language::En => format!("the install ended with {code}\n{tail}"),
        Language::Es => format!("la instalación terminó con {code}\n{tail}"),
    }
}

// --- The Settings panes' reads (tauri: settings) ----------------------------

/// "C:/h/auth.json could not be read: Access is denied. (os error 5)"
pub fn file_could_not_be_read(language: Language, path: &str, error: &str) -> String {
    match language {
        Language::En => format!("{path} could not be read: {error}"),
        Language::Es => format!("no se pudo leer {path}: {error}"),
    }
}

/// "gentle-shell could not be resolved: not found": the launcher the whole
/// settings area runs is not on this machine.
pub fn shell_could_not_be_resolved(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("gentle-shell could not be resolved: {error}"),
        Language::Es => format!("no se pudo resolver gentle-shell: {error}"),
    }
}

/// "gentle-shell could not run: Access is denied. (os error 5)"
pub fn shell_could_not_run(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("gentle-shell could not run: {error}"),
        Language::Es => format!("no se pudo ejecutar gentle-shell: {error}"),
    }
}

/// "the shell's agent folder could not be found on this machine": the Account
/// and Usage panes read the same folder, so they say the same sentence.
pub const fn shell_agent_folder_missing(language: Language) -> &'static str {
    match language {
        Language::En => "the shell's agent folder could not be found on this machine",
        Language::Es => "no se encontró la carpeta del agente de la shell en esta máquina",
    }
}

/// "gentle-pi's config folder could not be found on this machine"
pub const fn agent_profiles_folder_missing(language: Language) -> &'static str {
    match language {
        Language::En => "gentle-pi's config folder could not be found on this machine",
        Language::Es => "no se encontró la carpeta de configuración de gentle-pi en esta máquina",
    }
}

// --- Running the usage sources (tauri: settings::usage) ---------------------

/// "the usage sources are not installed for this shell"
pub const fn usage_sources_not_installed(language: Language) -> &'static str {
    match language {
        Language::En => "the usage sources are not installed for this shell",
        Language::Es => "las fuentes de uso no están instaladas para esta shell",
    }
}

/// "the usage read could not start": no thread could be started for it.
pub const fn usage_read_could_not_start(language: Language) -> &'static str {
    match language {
        Language::En => "the usage read could not start",
        Language::Es => "no se pudo iniciar la lectura de uso",
    }
}

/// "could not start (Access is denied. (os error 5))"
pub fn usage_sources_could_not_start(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("could not start ({error})"),
        Language::Es => format!("no se pudo iniciar ({error})"),
    }
}

/// "did not answer within 30 seconds"
pub fn usage_sources_did_not_answer(language: Language, seconds: u64) -> String {
    match language {
        Language::En => format!("did not answer within {seconds} seconds"),
        Language::Es => format!("no respondió en {seconds} segundos"),
    }
}

/// "could not run (Access is denied. (os error 5))"
pub fn usage_sources_could_not_run(language: Language, error: &str) -> String {
    match language {
        Language::En => format!("could not run ({error})"),
        Language::Es => format!("no se pudo ejecutar ({error})"),
    }
}

/// "stopped without printing a document (exit code 1)": what stands in the
/// brackets is the code this catalog worded, or the script's own dying words,
/// which travel as they arrived (D7).
pub fn usage_sources_stopped(language: Language, what: &str) -> String {
    match language {
        Language::En => format!("stopped without printing a document ({what})"),
        Language::Es => format!("se detuvo sin imprimir un documento ({what})"),
    }
}

// --- The usage document's refusals (runtime::usage) -------------------------
//
// One WHOLE sentence per refusal, instead of the fragment the reader composed
// into "the usage sources answered {what} this pane does not read": the join
// sits in the middle of the sentence, and a Spanish column that nests an
// English-shaped clause reads as no sentence at all. The English is the text
// the old composition produced, byte for byte.

/// The usage sources answered a document of another schema.
pub fn usage_sources_schema(language: Language, schema: &str) -> String {
    match language {
        Language::En => {
            format!("the usage sources answered schema \"{schema}\" this pane does not read")
        }
        Language::Es => format!(
            "las fuentes de uso respondieron el esquema \"{schema}\", que este panel no lee"
        ),
    }
}

/// The usage sources answered a provider with a status this pane does not know.
pub fn usage_sources_status(language: Language, status: &str) -> String {
    match language {
        Language::En => {
            format!("the usage sources answered status \"{status}\" this pane does not read")
        }
        Language::Es => {
            format!("las fuentes de uso respondieron el estado \"{status}\", que este panel no lee")
        }
    }
}

/// The usage document carries no `sources` list.
pub const fn usage_sources_no_provider_list(language: Language) -> &'static str {
    match language {
        Language::En => {
            "the usage sources answered a document with no provider list this pane does not read"
        }
        Language::Es => {
            "las fuentes de uso respondieron un documento sin lista de proveedores, \
             que este panel no lee"
        }
    }
}

/// The provider at 1-based `position` has no name.
pub fn usage_sources_provider_without_name(language: Language, position: usize) -> String {
    match language {
        Language::En => format!(
            "the usage sources answered provider {position} with no name this pane does not read"
        ),
        Language::Es => format!(
            "las fuentes de uso respondieron un proveedor {position} sin nombre, \
             que este panel no lee"
        ),
    }
}

/// `provider` has no `limits` list.
pub fn usage_sources_provider_without_limits(language: Language, provider: &str) -> String {
    match language {
        Language::En => format!(
            "the usage sources answered provider {provider} with no limit list this pane does \
             not read"
        ),
        Language::Es => format!(
            "las fuentes de uso respondieron un proveedor {provider} sin lista de límites, \
             que este panel no lee"
        ),
    }
}

/// A limit of `provider` has no name.
pub fn usage_sources_limit_without_name(language: Language, provider: &str) -> String {
    match language {
        Language::En => format!(
            "the usage sources answered a limit of {provider} with no name this pane does not read"
        ),
        Language::Es => format!(
            "las fuentes de uso respondieron un límite de {provider} sin nombre, \
             que este panel no lee"
        ),
    }
}

/// A limit of `provider` has no `limitReached` mark.
pub fn usage_sources_limit_without_reach_mark(language: Language, provider: &str) -> String {
    match language {
        Language::En => format!(
            "the usage sources answered a limit of {provider} with no reach mark this pane does \
             not read"
        ),
        Language::Es => format!(
            "las fuentes de uso respondieron un límite de {provider} sin marca de alcanzado, \
             que este panel no lee"
        ),
    }
}

/// The limit at 1-based `position` of `provider` has no `windows` list.
pub fn usage_sources_limit_without_windows(
    language: Language,
    position: usize,
    provider: &str,
) -> String {
    match language {
        Language::En => format!(
            "the usage sources answered limit {position} of {provider} with no windows this \
             pane does not read"
        ),
        Language::Es => format!(
            "las fuentes de uso respondieron el límite {position} de {provider} sin ventanas, \
             que este panel no lee"
        ),
    }
}

/// A window of `limit` of `provider` has no label.
pub fn usage_sources_window_without_label(
    language: Language,
    provider: &str,
    limit: &str,
) -> String {
    match language {
        Language::En => format!(
            "the usage sources answered a window of {provider} ({limit}) with no label this \
             pane does not read"
        ),
        Language::Es => format!(
            "las fuentes de uso respondieron una ventana de {provider} ({limit}) sin etiqueta, \
             que este panel no lee"
        ),
    }
}

/// A window of `limit` of `provider` has no `usedPercent`.
pub fn usage_sources_window_without_percentage(
    language: Language,
    provider: &str,
    limit: &str,
) -> String {
    match language {
        Language::En => format!(
            "the usage sources answered a window of {provider} ({limit}) with no percentage \
             this pane does not read"
        ),
        Language::Es => format!(
            "las fuentes de uso respondieron una ventana de {provider} ({limit}) sin \
             porcentaje, que este panel no lee"
        ),
    }
}

/// A window of `provider` names a `resetAt` this pane cannot read.
pub fn usage_sources_without_reset_time(language: Language, provider: &str) -> String {
    match language {
        Language::En => {
            format!("the usage sources answered a reset time of {provider} this pane does not read")
        }
        Language::Es => format!(
            "las fuentes de uso respondieron una hora de reinicio de {provider}, \
             que este panel no lee"
        ),
    }
}

// --- The profiles files' refusals (runtime::profiles) -----------------------
//
// The name in `{:?}` is quoted by the format itself, exactly as the English pin
// quoted it before this catalog existed.

/// "profiles.json is not readable: Access is denied. (os error 5)"
pub fn profiles_file_unreadable(language: Language, file: &str, error: &str) -> String {
    match language {
        Language::En => format!("{file} is not readable: {error}"),
        Language::Es => format!("no se puede leer {file}: {error}"),
    }
}

/// "profiles.json is not a gentle-pi.agent_model_profiles document, it says
/// \"other\""
pub fn profiles_not_a_document(language: Language, file: &str, kind: &str, says: &str) -> String {
    match language {
        Language::En => format!("{file} is not a {kind} document, it says {says:?}"),
        Language::Es => format!("{file} no es un documento {kind}, dice {says:?}"),
    }
}

/// "profiles.json is version 2, this view reads version 1"
pub fn profiles_wrong_version(
    language: Language,
    file: &str,
    version: u64,
    expected: u64,
) -> String {
    match language {
        Language::En => {
            format!("{file} is version {version}, this view reads version {expected}")
        }
        Language::Es => {
            format!("{file} tiene la versión {version}, esta vista lee la versión {expected}")
        }
    }
}

/// "profile-fallbacks.json names a profile no reader accepts: \"bad name\""
pub fn profiles_name_refused(language: Language, file: &str, name: &str) -> String {
    match language {
        Language::En => format!("{file} names a profile no reader accepts: {name:?}"),
        Language::Es => format!("{file} nombra un perfil que ningún lector acepta: {name:?}"),
    }
}

/// "profile-fallbacks.json: \"work\" names the agent \"bad agent\""
pub fn profiles_agent_refused(
    language: Language,
    file: &str,
    profile: &str,
    agent: &str,
) -> String {
    match language {
        Language::En => format!("{file}: {profile:?} names the agent {agent:?}"),
        Language::Es => format!("{file}: {profile:?} nombra el agente {agent:?}"),
    }
}

/// "profile-fallbacks.json: \"work\"/\"agent\" is not a chain"
pub fn profiles_not_a_chain(language: Language, file: &str, profile: &str, agent: &str) -> String {
    match language {
        Language::En => format!("{file}: {profile:?}/{agent:?} is not a chain"),
        Language::Es => format!("{file}: {profile:?}/{agent:?} no es una cadena"),
    }
}

// --- The model's own error, and the close (gentle-core::chat, tauri::close) --

/// "unknown model error": the provider ended a turn on an error and said
/// nothing about it. The text it did send travels as it arrived (D7).
pub const fn unknown_model_error(language: Language) -> &'static str {
    match language {
        Language::En => "unknown model error",
        Language::Es => "error desconocido del modelo",
    }
}

/// "the app is closing": the one line a chat command answers once the close has
/// stopped new work.
pub const fn closing(language: Language) -> &'static str {
    match language {
        Language::En => "the app is closing",
        Language::Es => "la app se está cerrando",
    }
}

// --- The notes read on demand (tauri::commands::read_whats_new, T5, S9, D6) --

/// "The notes could not be read: no connection." - the one quiet line the About
/// pane draws in place of the what's-new modal. A release server that did not
/// answer shares this line: the seam's own words are the client's, never the
/// screen's, and the cause the user can act on is the connection.
pub const fn whats_new_unreadable(language: Language) -> &'static str {
    match language {
        Language::En => "The notes could not be read: no connection.",
        Language::Es => "No se pudieron leer las novedades: sin conexión.",
    }
}

/// "No notes were published for this version." - the release manifest carries
/// ANOTHER version's notes, which are never shown as this one's.
pub const fn whats_new_another_version(language: Language) -> &'static str {
    match language {
        Language::En => "No notes were published for this version.",
        Language::Es => "No se publicaron novedades para esta versión.",
    }
}

/// "This version's notes are empty." - the manifest of this version was read and
/// its text is nothing, so there is no modal to open.
pub const fn whats_new_no_text(language: Language) -> &'static str {
    match language {
        Language::En => "This version's notes are empty.",
        Language::Es => "Las novedades de esta versión están vacías.",
    }
}

#[cfg(test)]
mod tests;
