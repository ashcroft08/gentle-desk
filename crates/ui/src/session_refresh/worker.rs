//! The iced transport of a refresh pass: the moved core, wrapped so the window
//! can consume it.
//!
//! [`stream`] runs one pass on its own thread inside an iced channel: partials
//! are dropped when nobody reads them, the final list is waited for. [`run`] is
//! the test harness that drains the stream.

use iced::futures::{executor::block_on, SinkExt, Stream};

use gentle_runtime::refresh_queue::Job;
use gentle_runtime::session_refresh::{work, Update};

#[cfg(test)]
use super::Refreshed;
use super::Shared;

/// Room for several reports and the final item: the worker never drops a list
/// for lack of space, and a partial it cannot hand over waits for nobody.
const CHANNEL: usize = 8;

/// Runs one pass and yields its items: the first full pass reports as it goes,
/// and every pass ends with its final item.
pub fn stream(job: Job, shared: Shared) -> impl Stream<Item = Update> {
    iced::stream::channel(CHANNEL, async move |mut output| {
        // The pass is a `stat` per unchanged file and a full parse of the new
        // ones: it stays off the UI thread, and so does the folding.
        std::thread::spawn(move || {
            work(&job, &shared, &mut || shared.now(), &mut |item| {
                // A partial nobody has read yet is not worth waiting for: the
                // next one comes in half a second, and the index lock is held
                // meanwhile. Nobody listening means the window is gone.
                if matches!(item, Update::Progress(_)) {
                    let _ = output.try_send(item);
                } else {
                    // The final item is the list: the worker waits for room
                    // instead of dropping it. It holds no lock by now.
                    let _ = block_on(output.send(item));
                }
            });
        });
    })
}

/// Runs one pass and returns only its final item, for callers that want the
/// list and nothing else. The shipped path is [`stream`] through
/// `refresh_flow`; this wrapper exists for tests and the module's own helpers.
#[cfg(test)]
pub(crate) async fn run(job: Job, shared: Shared) -> Result<Refreshed, String> {
    use iced::futures::StreamExt;
    let mut items = Box::pin(stream(job, shared));
    let mut last = None;
    while let Some(item) = items.next().await {
        if let Update::Refreshed(result) = item {
            last = Some(result);
        }
    }
    last.unwrap_or_else(|| Err("the session refresh thread ended unexpectedly".to_owned()))
}
