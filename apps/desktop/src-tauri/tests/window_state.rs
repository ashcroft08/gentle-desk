//! Windowless tests of where the window opens and how it was closed
//! (window-state slice 2).
//!
//! Nothing here builds a `tauri::WebviewWindow`: a window needs a running app, so
//! every test exercises the seam `window` keeps for it - the numbers a window
//! reports in, a [`WindowState`] out, and the window calls that carry a
//! [`Placement`] back out - or the document that keeps the result across a
//! restart. The one step that touches the real window (`place`) is the same
//! [`steps`] list run against Tauri, so the order pinned here is the order the
//! window gets.

use std::fs;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use gentle_desk_lib::close::flow::WINDOW_SAVE;
use gentle_desk_lib::close::{CloseFlow, CloseSteps, Deadlines};
use gentle_desk_lib::window::{screen_at, steps, stored_state, Readings, Step, WindowRecorder};
use gentle_runtime::app_config::{self, placement, AppConfig, Placement, Screen, WindowState};

/// How long a test waits for a close that its own deadlines bound.
const LIMIT: Duration = Duration::from_secs(10);

/// A 1280x800 window that is not maximized, fullscreen or minimized.
const PLAIN: WindowState = WindowState {
    x: 100,
    y: 80,
    width: 1280,
    height: 800,
    maximized: false,
    fullscreen: false,
    minimized: false,
};

/// The readings of a window that is up and answering, with the three flags it
/// reports.
fn readings(maximized: bool, fullscreen: bool, minimized: bool) -> Readings {
    Readings {
        position: Some((100, 80)),
        size: Some((1280, 800)),
        maximized,
        fullscreen,
        minimized,
    }
}

#[test]
fn the_numbers_a_window_reports_are_stored_as_they_are() {
    assert_eq!(stored_state(readings(false, false, false)), Some(PLAIN));
}

#[test]
fn a_maximized_window_is_stored_maximized_with_the_rectangle_it_reported() {
    // The decision: a maximized window is saved as `maximized: true` AND with the
    // rectangle the window reports at that moment. The rectangle is applied
    // first and `maximize()` after it (see the order test below), so the flag has
    // the last word whatever the window manager did with the size, and a run that
    // later opens on a smaller screen still has a rectangle to fall back to.
    assert_eq!(
        stored_state(readings(true, false, false)),
        Some(WindowState {
            maximized: true,
            ..PLAIN
        })
    );
}

#[test]
fn a_fullscreen_or_minimized_window_is_stored_with_every_flag_it_reported() {
    assert_eq!(
        stored_state(readings(false, true, false)),
        Some(WindowState {
            fullscreen: true,
            ..PLAIN
        })
    );
    assert_eq!(
        stored_state(readings(false, false, true)),
        Some(WindowState {
            minimized: true,
            ..PLAIN
        })
    );
}

#[test]
fn a_window_that_reports_no_position_or_no_size_stores_nothing() {
    // A window that is already gone answers neither, and one without both is not
    // a rectangle worth restoring: nothing is stored, so the close has nothing to
    // fail on.
    let no_position = Readings {
        position: None,
        ..readings(false, false, false)
    };
    let no_size = Readings {
        size: None,
        ..readings(false, false, false)
    };
    assert_eq!(stored_state(no_position), None);
    assert_eq!(stored_state(no_size), None);
}

#[test]
fn nothing_stored_centres_the_window_and_touches_no_size() {
    // Tauri's own centring, and no `Size` call at all: an uncentred or resized
    // window would be one this slice changed behind the user's back.
    assert_eq!(steps(Placement::Center), [Step::Center]);
}

#[test]
fn a_restored_window_is_sized_then_positioned_then_flagged_last() {
    // Fullscreen, then maximized, then minimized last: each flag can undo the
    // one before it in the window manager, so it must not be able to undo the
    // rectangle, and minimizing must not be undone by anything.
    let stored = WindowState {
        maximized: true,
        fullscreen: true,
        minimized: true,
        ..PLAIN
    };
    assert_eq!(
        steps(Placement::Restore(stored)),
        [
            Step::Size {
                width: 1280,
                height: 800
            },
            Step::Position { x: 100, y: 80 },
            Step::Fullscreen,
            Step::Maximized,
            Step::Minimized,
        ]
    );
}

#[test]
fn a_plain_window_is_sized_and_positioned_and_flagged_nowhere() {
    assert_eq!(
        steps(Placement::Restore(PLAIN)),
        [
            Step::Size {
                width: 1280,
                height: 800
            },
            Step::Position { x: 100, y: 80 },
        ]
    );
}

#[test]
fn a_minimized_restore_ends_minimized_and_names_no_focus_step() {
    // The last call is the one that decides how the window appears, and nothing
    // here focuses or raises it: a run closed minimized comes back minimized
    // instead of pushing itself in front of what the user is reading.
    for flags in [
        WindowState {
            minimized: true,
            ..PLAIN
        },
        WindowState {
            maximized: true,
            minimized: true,
            ..PLAIN
        },
    ] {
        assert_eq!(
            steps(Placement::Restore(flags)).last(),
            Some(&Step::Minimized),
            "a stored {flags:?} ends minimized"
        );
    }
}

#[test]
fn a_rectangle_that_no_longer_lands_on_a_screen_centres_without_a_size() {
    // The slice-1 rule, through the step list the app actually runs: the screen a
    // monitor that was unplugged used to hold is no screen, so the window opens
    // centred and keeps the size it was created with.
    let screen = screen_at((0, 0), (1920, 1080));
    let off_screen = WindowState {
        x: 3000,
        y: 400,
        ..PLAIN
    };
    assert_eq!(
        steps(placement(Some(off_screen), &[screen])),
        [Step::Center]
    );
    assert_eq!(
        steps(placement(Some(PLAIN), &[screen])),
        [
            Step::Size {
                width: 1280,
                height: 800
            },
            Step::Position { x: 100, y: 80 },
        ],
        "and one that still lands is restored"
    );
}

#[test]
fn a_monitor_becomes_the_screen_the_rule_reads() {
    // The mapping from what the window API reports to the rectangle the decision
    // reads: a monitor left of the primary has a negative origin, and it is the
    // origin the rule compares against.
    let left = screen_at((-1440, 40), (1440, 900));
    assert_eq!(
        left,
        Screen {
            x: -1440,
            y: 40,
            width: 1440,
            height: 900,
        }
    );
    let stored = WindowState {
        x: -1440,
        y: 80,
        ..PLAIN
    };
    assert_eq!(
        placement(Some(stored), &[screen_at((0, 0), (1920, 1080)), left]),
        Placement::Restore(stored)
    );
}

#[test]
fn the_stored_window_round_trips_through_save_and_load() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    let config = AppConfig {
        window: Some(WindowState {
            maximized: true,
            minimized: true,
            ..PLAIN
        }),
        ..AppConfig::default()
    };
    app_config::save(&path, &config).unwrap();

    let loaded = app_config::load(&path);

    assert_eq!(loaded.notice, None, "the file was written, not damaged");
    assert_eq!(loaded.config.window, config.window);
    assert_eq!(
        loaded.config.theme, config.theme,
        "and the rest of the document came back with it"
    );
}

#[test]
fn a_config_that_never_recorded_a_window_is_written_without_the_field() {
    // Like the chosen home: a run that has not closed a window yet writes the
    // document older builds wrote, so the field only appears once it means
    // something.
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.json");
    app_config::save(&path, &AppConfig::default()).unwrap();

    let written = fs::read_to_string(&path).unwrap();

    assert!(
        !written.contains("\"window\""),
        "nothing was recorded, so nothing is written: {written}"
    );
    assert_eq!(app_config::load(&path).config.window, None);
}

/// The steps of a close, recorded: the window's is this slice's, the rest are
/// the flow's. With `hang` naming one of them, that step never returns, and the
/// flow must abandon it and go on.
struct Recorded {
    ran: Arc<Mutex<Vec<&'static str>>>,
    hang: Option<&'static str>,
}
impl Recorded {
    fn new(hang: Option<&'static str>) -> (Self, Arc<Mutex<Vec<&'static str>>>) {
        let ran = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                ran: Arc::clone(&ran),
                hang,
            },
            ran,
        )
    }

    fn record(&self, step: &'static str) {
        lock(&self.ran).push(step);
        if self.hang == Some(step) {
            // Long enough to outlive every deadline, and the thread is abandoned.
            std::thread::sleep(Duration::from_secs(60));
        }
    }
}

impl CloseSteps for Recorded {
    fn stop_work(&self) {
        self.record("work");
    }

    fn stop_children(&self) {
        self.record("children");
    }

    fn join_pump(&self, _budget: Duration) {
        self.record("pump");
    }

    fn stop_watch(&self, _budget: Duration) {
        self.record("watch");
    }

    fn flush_cache(&self) {
        self.record("cache");
    }

    fn flush_config(&self, _budget: Duration) {
        self.record("config");
    }

    fn exit(&self) {
        self.record("exit");
    }
}

/// The window of a close, recorded into the same list as the steps, so the order
/// is one value. With `hang`, it never answers, and the flow must abandon it.
struct RecordedWindow {
    ran: Arc<Mutex<Vec<&'static str>>>,
    hang: bool,
}

impl WindowRecorder for RecordedWindow {
    fn record(&self) {
        lock(&self.ran).push("window");
        if self.hang {
            std::thread::sleep(Duration::from_secs(60));
        }
    }
}

#[test]
fn a_close_records_the_window_before_it_flushes_the_config() {
    // The order that makes the state survive: the window step writes through the
    // run's writer, and the config flush after it is what puts that write on the
    // disk before the process ends.
    let (steps, ran) = Recorded::new(None);
    let flow = CloseFlow::new(Arc::new(steps), Deadlines::default()).with_window(Arc::new(
        RecordedWindow {
            ran: Arc::clone(&ran),
            hang: false,
        },
    ));

    assert!(
        flow.request(),
        "the first close request is the one that starts the shutdown"
    );
    wait_for(&ran, "exit");

    let order = lock(&ran).clone();
    let window = order
        .iter()
        .position(|step| *step == "window")
        .unwrap_or_else(|| panic!("the window step never ran, ran {order:?}"));
    let config = order
        .iter()
        .position(|step| *step == "config")
        .unwrap_or_else(|| panic!("the config was never flushed, ran {order:?}"));
    assert!(
        window < config,
        "the window is recorded before the config is flushed, ran {order:?}"
    );
}

#[test]
fn a_window_that_never_answers_is_left_behind_and_the_close_goes_on() {
    // A window that refuses to answer (already gone, or a window manager that
    // does not reply) is bounded by its own deadline: the close still reaches its
    // last step, and still writes the config.
    let (steps, ran) = Recorded::new(None);
    let flow = CloseFlow::new(Arc::new(steps), Deadlines::default()).with_window(Arc::new(
        RecordedWindow {
            ran: Arc::clone(&ran),
            hang: true,
        },
    ));
    let started = Instant::now();

    assert!(flow.request());
    wait_for(&ran, "exit");
    let waited = started.elapsed();

    assert!(
        waited < WINDOW_SAVE * 2 + Duration::from_secs(2),
        "the close waited {waited:?} for a window that never answered"
    );
    assert!(
        lock(&ran).contains(&"config"),
        "and went on to flush the config, ran {:?}",
        lock(&ran)
    );
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// Waits for the shutdown to reach `step`, failing the test instead of hanging.
fn wait_for(ran: &Arc<Mutex<Vec<&'static str>>>, step: &str) {
    let deadline = Instant::now() + LIMIT;
    while !lock(ran).contains(&step) {
        assert!(
            Instant::now() < deadline,
            "waited {LIMIT:?} for the close to reach {step}, ran {:?}",
            lock(ran)
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
