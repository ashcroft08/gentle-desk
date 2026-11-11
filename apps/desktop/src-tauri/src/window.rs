//! Where the window opens, and how it was closed: the window's half of
//! `gentle_runtime::app_config::window`.
//!
//! Two directions, and the rule between them is [`placement`]'s (slice 1):
//!
//! - ON THE WAY IN, [`place`] reads the screens that exist right now, asks
//!   `placement(stored, screens)` and runs the calls that answer it: Tauri's own
//!   centring when the rectangle no longer lands on a screen, or the stored
//!   rectangle and its flags otherwise. The calls are a list of [`Step`]s
//!   ([`steps`]) so the ORDER is a value a windowless test pins: size, then
//!   position, then fullscreen, then maximized, then minimized - each flag is
//!   what the window manager may undo the one before it with, so nothing that
//!   sets a rectangle comes after a flag.
//! - THE WINDOW IS CREATED HIDDEN (`tauri.conf.json` sets `"visible": false`) and
//!   is shown by that list's last step, so it never appears at the created place
//!   for a moment before the stored one lands. [`Step::Show`] is the last call of
//!   EVERY plan, the minimized one included: a minimized hidden window is not
//!   shown by minimizing it (a real window stays invisible and off screen, with
//!   no taskbar entry, and never comes back), so the show is the one call that
//!   cannot be skipped. Minimizing first and showing after is the order that
//!   keeps the window off a normal-size frame and out of the user's way while it
//!   still gives the run a window that can be seen and reopened. Nothing here can
//!   leave the window hidden - [`run`] never lets a failed step skip a later one,
//!   and [`place`] ends by showing a window that is still hidden.
//! - ON THE WAY OUT, [`SystemWindowRecorder`] reads the same numbers off the
//!   window and stores them through the run's config writer. It runs as a step of
//!   the close flow (`close::flow`), bounded by its own deadline, because a
//!   window that is already gone answers nothing and must not hold the close.
//!
//! Nothing here focuses the window: a run that was minimized is minimized before
//! it is shown, and no step asks to raise or activate it in front of what the user
//! is reading.

use std::sync::Arc;

use gentle_runtime::app_config::{placement, Placement, Screen, WindowState};
use tauri::{PhysicalPosition, PhysicalSize, WebviewWindow, Wry};

use crate::bridge::AppStateHolder;

/// What one window reports, as it reports it: the raw numbers, before any rule.
///
/// Each half of the rectangle is absent when that one reading failed, which is
/// what a window that is already gone does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Readings {
    /// `outer_position`, in physical pixels.
    pub position: Option<(i32, i32)>,
    /// `inner_size`, in physical pixels.
    pub size: Option<(u32, u32)>,
    pub maximized: bool,
    pub fullscreen: bool,
    pub minimized: bool,
}

/// The state to store for these readings, or `None` when the window would not
/// report both its position and its size.
///
/// A rectangle with only half of it is no rectangle to open at, and a window that
/// answered nothing is a window that is already gone: nothing is stored then, so
/// the close keeps whatever the file says instead of inventing a rectangle.
pub fn stored_state(readings: Readings) -> Option<WindowState> {
    let (x, y) = readings.position?;
    let (width, height) = readings.size?;
    Some(WindowState {
        x,
        y,
        width,
        height,
        maximized: readings.maximized,
        fullscreen: readings.fullscreen,
        minimized: readings.minimized,
    })
}

/// A monitor as the window API reports it, as the [`Screen`] the placement rule
/// reads. A screen left of or above the primary keeps its negative origin.
pub fn screen_at(position: (i32, i32), size: (u32, u32)) -> Screen {
    Screen {
        x: position.0,
        y: position.1,
        width: size.0,
        height: size.1,
    }
}

/// One window call of an opening, in the order [`steps`] puts them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    /// Tauri's own centring, which leaves the size alone.
    Center,
    Size {
        width: u32,
        height: u32,
    },
    Position {
        x: i32,
        y: i32,
    },
    /// Fullscreen on: the first of the three flags, because a maximized or
    /// minimized window is what it can undo.
    Fullscreen,
    Maximized,
    /// Minimized last of the flags: no flag follows it, so no flag takes it back,
    /// and no call here focuses the window it minimizes. The plan's show follows
    /// it, which does not undo the minimize: it is what makes the already
    /// minimized window visible in the taskbar.
    Minimized,
    /// Shown, on a window that was created hidden: the last call of every plan, so
    /// the window is never left hidden and appears where it was put and nowhere
    /// else.
    Show,
}

/// The calls that carry out `placement`, in order.
///
/// [`Placement::Center`] is one call and no size: an uncentred or resized window
/// would be one this slice changed behind the user's back. [`Placement::Restore`]
/// sizes and positions first and only then applies the flags of the stored state.
///
/// Every plan here ends with [`Step::Show`], because the window is created
/// hidden: centred-then-shown for the centre, and
/// size-then-position-then-flags-then-shown for a restore, so the first frame the
/// user sees is already where the last run left it.
pub fn steps(placement: Placement) -> Vec<Step> {
    let Some(stored) = restore(placement) else {
        return vec![Step::Center, Step::Show];
    };
    let mut steps = vec![
        Step::Size {
            width: stored.width,
            height: stored.height,
        },
        Step::Position {
            x: stored.x,
            y: stored.y,
        },
    ];
    if stored.fullscreen {
        steps.push(Step::Fullscreen);
    }
    if stored.maximized {
        steps.push(Step::Maximized);
    }
    if stored.minimized {
        steps.push(Step::Minimized);
    }
    // THE ONE PLACE the minimized order is decided. A minimized plan is shown too,
    // last: minimizing a hidden window does NOT show it - a real window stays
    // invisible, parked off screen, with no taskbar entry and no way back for the
    // user - so `Minimized` followed by `Show` is the plan that leaves the window
    // minimized AND visible. On a real window (DPI-aware probe, 20 s watch, 3 runs)
    // a minimized plan with `Show` leaves the window minimized and visible in the
    // taskbar, with at most a brief foreground flash (0 to ~125 ms) and no
    // spontaneous restore observed; `Show` stays because hidden is the worse
    // outcome.
    steps.push(Step::Show);
    steps
}

/// Whether `plan` leaves the window minimized.
///
/// It is read off the steps themselves, never off the stored flags, so the plan a
/// window gets and this answer cannot disagree. It answers a question about the
/// plan only: [`Step::Show`] follows [`Step::Minimized`] in such a plan, and no
/// caller may read this to skip the show.
pub fn ends_minimized(plan: &[Step]) -> bool {
    plan.contains(&Step::Minimized)
}

/// The state a `Restore` carries, `None` for the centre.
fn restore(placement: Placement) -> Option<WindowState> {
    match placement {
        Placement::Center => None,
        Placement::Restore(state) => Some(state),
    }
}

/// Runs `plan` in order, through `apply`, and logs every failure instead of
/// returning it.
///
/// The loop has no `break` and no `?`: a step a window manager refuses costs the
/// user that step, never the ones after it, and in particular never the plan's
/// last one ([`Step::Show`]). This is the guarantee as a value a windowless test
/// drives (`tests/window_state.rs`), which is why the window is only reached
/// through `apply`.
pub fn run<F, E>(plan: &[Step], mut apply: F)
where
    F: FnMut(Step) -> Result<(), E>,
    E: std::fmt::Display,
{
    for &step in plan {
        if let Err(error) = apply(step) {
            eprintln!("window: {step:?} could not be applied: {error}");
        }
    }
}

/// Where the window opens: the stored rectangle when [`placement`] lands it on
/// one of `screens`, the centre otherwise.
///
/// A call that fails is logged and left behind, so a window manager that refuses
/// one step still gets the rest: the geometry first, the flags last, and none of
/// them worth failing the start over. The window was created hidden, so the run
/// ends by showing it - and the call that guarantees the user is never left with
/// an invisible window is `show_if_hidden(window)` at the end of this function:
/// even a plan whose own show failed, or a window that refused to report whether
/// it is visible, gets one more attempt once every step has run.
pub fn place(window: &WebviewWindow<Wry>, stored: Option<WindowState>, screens: &[Screen]) {
    let plan = steps(placement(stored, screens));
    run(&plan, |step| carry_out(window, step));
    // Every plan, with no exception for a minimized one: a hidden window is the
    // worst outcome there is - no window on screen, no taskbar entry, no way for
    // the user to reopen the app - so nothing is allowed to skip the show.
    show_if_hidden(window);
}

/// Shows `window` if the plan left it hidden, and says so when it could not.
///
/// The net under every path into [`place`]: a step that failed, a window manager
/// that ignored the show, or a plan whose show did not take still ends with a
/// window the user can see. It applies to every plan, the minimized one included,
/// and asking first is what keeps the extra call off a window that is already
/// visible.
fn show_if_hidden(window: &WebviewWindow<Wry>) {
    match window.is_visible() {
        Ok(true) => {}
        Ok(false) => {
            if let Err(error) = window.show() {
                eprintln!("window: the window could not be shown: {error}");
            }
        }
        Err(error) => eprintln!("window: whether it is visible could not be asked: {error}"),
    }
}

/// The screens that exist right now, as the placement rule reads them.
///
/// A run that cannot ask has no screen to land on, which is what an empty list
/// means: the window opens centred instead of at a rectangle nobody can check.
pub fn screens(window: &WebviewWindow<Wry>) -> Vec<Screen> {
    match window.available_monitors() {
        Ok(monitors) => monitors
            .iter()
            .map(|monitor| screen_at(as_pair(*monitor.position()), as_size(*monitor.size())))
            .collect(),
        Err(error) => {
            eprintln!("window: the screens could not be asked for: {error}");
            Vec::new()
        }
    }
}

/// Records how the window was closed, as a step of the close flow (`close::flow`
/// runs it, bounded by its own deadline).
///
/// The seam: the real window needs a running app, which only the setup step and
/// the close have, so the flow holds this instead of a Tauri type and a
/// windowless test records the step order through it.
pub trait WindowRecorder: Send + Sync {
    /// Reads the window as it is now and stores it in the config of this run,
    /// through the run's one writer. A window that answers nothing stores
    /// nothing: it never fails the close and never delays it.
    fn record(&self);
}

/// The real window: the numbers it reports, written into the config this run
/// holds.
pub struct SystemWindowRecorder {
    window: WebviewWindow<Wry>,
    holder: Arc<AppStateHolder>,
}

impl SystemWindowRecorder {
    /// The recorder of `window`, writing through `holder`'s config (which is what
    /// asks the run's writer).
    pub fn new(window: WebviewWindow<Wry>, holder: Arc<AppStateHolder>) -> Self {
        Self { window, holder }
    }
}

impl WindowRecorder for SystemWindowRecorder {
    fn record(&self) {
        let Some(stored) = stored_state(readings(&self.window)) else {
            // Already gone: the file keeps what it says, and the close goes on.
            return;
        };
        // One edit under the config lock, and the same rectangle again changes
        // nothing: a close that records what the file already holds writes no
        // file and moves no revision.
        self.holder.update_config(|config| {
            if config.window == Some(stored) {
                return false;
            }
            config.window = Some(stored);
            true
        });
    }
}

/// What `window` reports right now. Every reading that failed counts as the
/// window being gone, never as an error worth reporting: this runs while the app
/// is closing.
fn readings(window: &WebviewWindow<Wry>) -> Readings {
    Readings {
        position: window.outer_position().ok().map(as_pair),
        size: window.inner_size().ok().map(as_size),
        maximized: window.is_maximized().unwrap_or(false),
        fullscreen: window.is_fullscreen().unwrap_or(false),
        minimized: window.is_minimized().unwrap_or(false),
    }
}

/// One window call of a [`Step`], on the real window.
fn carry_out(window: &WebviewWindow<Wry>, step: Step) -> tauri::Result<()> {
    match step {
        Step::Center => window.center(),
        Step::Size { width, height } => window.set_size(PhysicalSize::new(width, height)),
        Step::Position { x, y } => window.set_position(PhysicalPosition::new(x, y)),
        Step::Fullscreen => window.set_fullscreen(true),
        Step::Maximized => window.maximize(),
        Step::Minimized => window.minimize(),
        Step::Show => window.show(),
    }
}

/// The position as the pair this module stores.
fn as_pair(position: PhysicalPosition<i32>) -> (i32, i32) {
    (position.x, position.y)
}

/// The size as the pair this module stores.
fn as_size(size: PhysicalSize<u32>) -> (u32, u32) {
    (size.width, size.height)
}
