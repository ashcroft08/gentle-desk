//! The rules for capping and reaping chat children, as pure functions.
//!
//! Nothing here starts, stops or locks anything: a rule takes the caps, the
//! chats as data and the instant the answer is for, and returns a decision. That
//! makes LRU choice, the idle window and the cap errors testable without a
//! process, and lets the host inject the clock.
//!
//! Two words repeat below. *Protected*: a chat that must never be stopped to make
//! room — the foreground one, one running a model call, or one still starting.
//! *Idle*: not protected and last active longer than [`Limits::idle_after`] ago.

use std::fmt;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::ChatKey;

/// The caps and the idle window. All three are configurable because the cost
/// they bound is the user's usage limit, not this process's memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// How many children may live at once.
    pub max_live: usize,
    /// How many of them may be working (running a model call) at once.
    pub max_working: usize,
    /// How long a background child may sit without activity before it is reaped.
    pub idle_after: Duration,
}

impl Limits {
    pub const fn new(max_live: usize, max_working: usize, idle_after: Duration) -> Self {
        Self {
            max_live,
            max_working,
            idle_after,
        }
    }
}

/// Four live children and two working ones are the proposal in the batch
/// document: enough for a chat that keeps working in the background, small
/// enough that parallel model calls do not drain the usage limit.
impl Default for Limits {
    fn default() -> Self {
        Self::new(4, 2, Duration::from_secs(10 * 60))
    }
}

/// Where the host reads "now" from.
pub trait Clock: Send + Sync {
    fn now(&self) -> Instant;
}

/// The clock production uses.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

/// A clock a test moves by hand, so an idle window of ten minutes takes no
/// time at all.
pub struct ManualClock {
    base: Instant,
    offset: Mutex<Duration>,
}

impl ManualClock {
    pub fn new() -> Self {
        Self {
            base: Instant::now(),
            offset: Mutex::new(Duration::ZERO),
        }
    }

    /// Moves the clock forward; it never goes back.
    pub fn advance(&self, by: Duration) {
        let mut offset = self.offset.lock().unwrap_or_else(|p| p.into_inner());
        *offset += by;
    }
}

impl Default for ManualClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Instant {
        self.base + *self.offset.lock().unwrap_or_else(|p| p.into_inner())
    }
}

/// One chat as a rule sees it: the facts a decision depends on, and nothing
/// else (no child handle, no process).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChatView {
    pub key: ChatKey,
    /// Running a model call, as the app derives it from `agent_start` /
    /// `agent_end`.
    pub working: bool,
    /// The chat the user is looking at.
    pub foreground: bool,
    /// Its child is still being started.
    pub pending: bool,
    /// The last instant this chat was touched (opened, sent to, marked working).
    pub last_active: Instant,
}

impl ChatView {
    /// A chat that must never be stopped to make room.
    pub fn protected(&self) -> bool {
        self.foreground || self.working || self.pending
    }
}

/// What an open may do to make room for one more child.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Admission {
    /// There is room: start the child.
    Open,
    /// The live cap is reached: stop this chat's child first.
    Reap(ChatKey),
}

/// The live cap is reached and nothing may be stopped for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TooManyLive {
    pub cap: usize,
    pub live: usize,
}

impl fmt::Display for TooManyLive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "cannot open another chat: {} live children (cap {}) and every one of them is \
             working, foreground or still starting",
            self.live, self.cap
        )
    }
}

impl std::error::Error for TooManyLive {}

/// Decides whether one more child fits: freely under the cap, by stopping the
/// least recently used unprotected chat at the cap, or not at all.
pub fn admit(limits: &Limits, views: &[ChatView]) -> Result<Admission, TooManyLive> {
    if views.len() < limits.max_live {
        return Ok(Admission::Open);
    }
    let victim = views
        .iter()
        .filter(|view| !view.protected())
        .min_by(|left, right| {
            left.last_active
                .cmp(&right.last_active)
                .then_with(|| left.key.cmp(&right.key))
        });
    match victim {
        Some(view) => Ok(Admission::Reap(view.key.clone())),
        None => Err(TooManyLive {
            cap: limits.max_live,
            live: views.len(),
        }),
    }
}

/// The unprotected chats that have been idle longer than the window, oldest
/// first. The foreground and working children are never in this list.
pub fn reap_idle(limits: &Limits, views: &[ChatView], now: Instant) -> Vec<ChatKey> {
    let mut idle: Vec<&ChatView> = views
        .iter()
        .filter(|view| !view.protected())
        .filter(|view| now.saturating_duration_since(view.last_active) > limits.idle_after)
        .collect();
    idle.sort_by(|left, right| {
        left.last_active
            .cmp(&right.last_active)
            .then_with(|| left.key.cmp(&right.key))
    });
    idle.into_iter().map(|view| view.key.clone()).collect()
}

/// How many chats are working right now.
pub fn working_count(views: &[ChatView]) -> usize {
    views.iter().filter(|view| view.working).count()
}

/// Whether another chat may start a model call.
pub fn working_full(limits: &Limits, views: &[ChatView]) -> bool {
    working_count(views) >= limits.max_working
}

#[cfg(test)]
mod tests;
