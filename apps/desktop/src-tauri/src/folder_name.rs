//! The name of the folder "Create a new folder" makes (Q1, Q6).
//!
//! Pure: the name is checked before anything is created and before a dialog is
//! opened, so a name Windows would not accept never writes to the disk.
//!
//! The rules are Windows' own, because the folder is made by Windows on this
//! machine: no path separator (the name is one segment, the parent comes from
//! the picker), no `..` (a name never climbs out of the parent), none of the
//! characters Windows forbids, no trailing dot or space (Windows drops both, so
//! the folder would not be the one the user named), and never a device name
//! (`CON`, `NUL`, `COM1`...), which names a device however it is extended.

/// The device names Windows reserves: any of them, with or without an
/// extension, names a device and never a folder.
const DEVICES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// The characters Windows forbids in a folder name.
const FORBIDDEN: [char; 9] = ['<', '>', ':', '"', '|', '?', '*', '/', '\\'];

/// The trimmed name of a new workspace folder, or the reason it cannot be one,
/// as one line of plain text for the user.
pub fn check(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("A workspace name cannot be empty.".to_owned());
    }
    // Checked on the name AS TYPED: `trim` is what would hide the trailing
    // space, and Windows would silently drop it, so the folder the user gets
    // would not be the folder they named.
    if name.chars().next_back().is_some_and(char::is_whitespace) {
        return Err("A workspace name cannot end with a space.".to_owned());
    }
    if trimmed.contains(['/', '\\']) {
        return Err("A workspace name cannot contain a path separator.".to_owned());
    }
    if trimmed.contains("..") {
        return Err("A workspace name cannot contain two dots in a row.".to_owned());
    }
    if trimmed.contains(FORBIDDEN) || trimmed.chars().any(char::is_control) {
        return Err(
            "A workspace name cannot contain any of these characters: < > : \" | ? *".to_owned(),
        );
    }
    if trimmed.ends_with('.') {
        return Err("A workspace name cannot end with a dot.".to_owned());
    }
    if is_device(trimmed) {
        return Err("That name is reserved by Windows.".to_owned());
    }
    Ok(trimmed.to_owned())
}

/// Whether `name` names a Windows device, whatever extension follows it.
fn is_device(name: &str) -> bool {
    let stem = name.split('.').next().unwrap_or(name).to_ascii_uppercase();
    DEVICES.contains(&stem.as_str())
}
