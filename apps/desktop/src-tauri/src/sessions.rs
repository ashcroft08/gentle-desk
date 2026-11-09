//! The session list slice: which pass runs when, and what the web side sees of
//! it (S2, S3 of the migration).
//!
//! The ROLE is the iced shell's (`crates/ui/src/app/sessions_update.rs`,
//! `crates/ui/src/refresh_flow.rs`), never its code, and the decisions are
//! `crates/runtime`'s, not re-implemented here:
//!
//! - [`service`]: the `Flow` of the run, the passes it asks for and the slices
//!   they push. A pass blocks on a `stat` per session file and a full parse of
//!   the new ones, so it runs on its own thread, never on a Tauri executor
//!   thread or the window's.
//! - [`view`]: the slice a `ViewState` carries: the list status and the count.
//! - [`tree`]: the flattened workspace tree, one row per workspace header and
//!   one per chat of the workspaces that are not collapsed.
//! - [`cache`]: the tree of the last revision, so a page of rows never regroups
//!   the whole list again.
//! - [`window`]: the `offset`/`length` pages of tree rows the web side asks for
//!   as it scrolls, instead of a pushed list (S3).
//! - [`revision`]: when the page a web side holds went stale.
//! - [`commands`]: the two read commands of the list, `sessions_window` and
//!   `get_view_state`, registered with the rest of the contract.
//! - [`startup`]: the index, its cache and the flow of this run. It starts no
//!   watcher: the watcher is not part of this slice.
//!
//! The seam is [`service::RunPass`] for "run one pass" and [`crate::events::Push`] for
//! "emit the slice", so the tests run a real home and no window.

pub mod cache;
pub mod commands;
pub mod revision;
pub mod service;
pub mod startup;
pub mod tree;
pub mod view;
pub mod window;

pub use service::SessionsService;
