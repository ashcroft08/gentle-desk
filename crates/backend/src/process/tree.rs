//! The child and its descendants: hiding it, and killing all of it.
//!
//! `Child::kill` alone leaves grandchildren (pi extensions, provider bridges)
//! running on Windows, so a child that outlives its grace period is taken down
//! through `taskkill /T`.

use std::process::{Child, Command, Stdio};

/// Spawns or kills without a console window of its own: a console flash per
/// child is visible in the parent window.
#[cfg(windows)]
pub(super) fn hide_console_window(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
pub(super) fn hide_console_window(_command: &mut Command) {}

/// Kills the child and its descendants. `Child::kill` alone leaves
/// grandchildren (pi extensions, provider bridges) running on Windows.
pub(super) fn kill_tree(child: &mut Child) {
    #[cfg(windows)]
    {
        let mut taskkill = Command::new("taskkill");
        taskkill
            .args(["/T", "/F", "/PID", &child.id().to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        hide_console_window(&mut taskkill);
        let _ = taskkill.status();
    }
    let _ = child.kill(); // no-op if taskkill already did it
}
