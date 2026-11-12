# close-race: the close flow's races (backlog item 26)

One slice per finding, each with the seam it needs. Nothing here has an observed user symptom;
the goal is that the close flow's guarantees hold by construction, not by comment.

## Specs

- S1: A watcher batch thread accepted between the stop check and the join is never waited for:
  `sessions/watch.rs:100-119` checks `is_stopped`, spawns, then pushes the handle, while `join`
  (`:126-131`) already `mem::take`s the vector - so a thread accepted in that window can call
  `on_batch` after the close flushed the cache. The fix must make accept-and-register atomic
  against the stop so every spawned thread is joined, or prove the existing shape already covers it.
- S2: An in-flight pass can write the session cache concurrently with the close's `flush_cache`:
  `passes.rs:24-28` checks `is_stopped` then spawns a thread that never re-checks, and
  `close/bridge.rs:105-115` flushes; both write the same cache (`session_refresh.rs` write path).
  The fix must order or bar the last write, or make the flush own the final state.
- S3: `SessionsService::stop(budget)` can take up to twice its deadline because it joins timer and
  watch with the full budget each (`sessions/refresh.rs:52-53`), while the close flow's deadline for
  the step is one `watch` (5 s, `close/flow.rs:40-50`); the step is then abandoned mid-join. The fix
  must make `stop` honor one shared budget across both joins.
- S4: The fallback shutdown runs `shutdown()` inline on the `CloseRequested` handler thread
  (`close/flow.rs:151-162` if the `app-close` thread cannot be spawned), the exact freeze the close
  slice exists to avoid. The fix must never run the shutdown on the caller thread, or must bound it.

## Tasks

- T1 (S1) Accept-or-stop: make the watch thread registration atomic with the stop so no spawned
  batch thread escapes the join - route: delegated writer, commit: see git log (fix/close-race-t1)
- T2 (S2) Last-writer rule: the close's flush must not race an in-flight pass - route: delegated
  writer, commit: see git log (fix/close-race-t2)
- T3 (S3) One shared budget for `SessionsService::stop` - route: delegated writer, commit: pending
- T4 (S4) The fallback shutdown leaves the caller thread - route: delegated writer, commit: pending

## Log

- L1 (backlog item 26, fresh read 2026-11-11): "five non-blocking findings from the review of the
  file watcher and close flow (#61, #62); the residual close race documented in T8a stays open."
  The original five findings are not recorded anywhere; the four concrete concerns above came from
  the fresh read (path:line evidence in each S#).
- L2 (2026-11-12, user): after shipping v0.1.7 and the close-up maintenance, the user chose to work
  the backlog starting with this item ("vamos con el mantenimiento de cierre y luego con lo del
  backlog", with #26 the parent's recommended start).
