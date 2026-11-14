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
