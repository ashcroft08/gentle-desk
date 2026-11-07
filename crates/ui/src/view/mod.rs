//! The visual components.
//!
//! Convention: one file per component, each exposing a pure `view(...)` over
//! plain data and its own `Event` enum for what the user did. A component
//! never touches app state or another component; `app` maps each `Event` to
//! an intent in one place. New UI goes in a new file here, not into `app.rs`.

pub mod banner;
pub mod composer;
pub mod dialog;
pub mod header;
pub mod helper_thread;
pub mod helpers;
pub mod notices;
pub mod sidebar;
pub mod thread;
