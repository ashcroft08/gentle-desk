//! UI-free core of gentle-desk.
//!
//! Pure protocol and state code: nothing here spawns processes, touches the
//! file system or knows about any GUI framework. The `ui` layer renders the
//! state this crate produces and the `backend` layer feeds it lines read from
//! `gentle-shell --mode rpc`.

pub mod chat;
pub mod rpc;
