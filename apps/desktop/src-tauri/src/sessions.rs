//! The session list slice: which pass runs when, and what the web side sees of
//! it (S2, S3 of the migration).
//!
//! The ROLE is the iced shell's (`crates/ui/src/app/sessions_update.rs`,
//! `crates/ui/src/refresh_flow.rs`, `crates/ui/src/subscription.rs`), never its
//! code, and the decisions are `crates/runtime`'s, not re-implemented here:
//!
//! - [`service`]: the `Flow` of the run, the passes it asks for and the slices
//!   they push. A pass blocks on a `stat` per session file and a full parse of
//!   the new ones, so it runs on its own thread, never on a Tauri executor
//!   thread or the window's.
//! - [`passes`]: the pass thread itself and the input it folds per report.
//! - [`refresh`]: boot and stop of the two sources beside the passes, and what
//!   each of their reports asks the flow for.
//! - [`watch`]: the thread that applies the session watcher's batches, and the
//!   seam that starts a watcher.
//! - [`timer`]: the safety-net timer, re-reading the flow's delay.
//! - [`wiring`]: where a watcher is started from and how long the net waits.
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
//! - [`startup`]: the index, its cache, the flow and the wiring of this run.
//!   The watcher and the safety net start at boot, in the setup step.
//!
//! The seams are [`service::RunPass`] for "run one pass", [`crate::events::Push`]
//! for "emit the slice", [`watch::StartWatch`] for "start a watcher" and
//! [`timer::TickDelay`] for "how long the safety net waits", so the tests run a
//! real home and no window.

pub mod cache;
pub mod commands;
pub mod passes;
pub mod refresh;
pub mod revision;
pub mod service;
pub mod startup;
pub mod timer;
pub mod tree;
pub mod view;
pub mod watch;
pub mod window;
pub mod wiring;

pub use service::SessionsService;
pub use wiring::Wiring;
