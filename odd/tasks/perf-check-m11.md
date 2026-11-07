# perf-check-m11: long threads and big chat lists

Backlog item 4: measure first, optimize only if it hurts. Context: the app is meant for other people too, who may keep thousands of chats.

## Specs

- S1: "medir primero, optimizar solo si molesta" (backlog item 4), for a very long thread and for a sidebar with hundreds of chats (the sidebar draws every row, no virtualization).
- S2: Measure with the real window (release build) and the stand-in gentle-shell, no model.

## Tasks

- [x] T1 `flood` scenario in the stand-in (300 markdown messages: heading, list, code block; `flood-text.mjs`) + terminal e2e test (6 e2e tests now)
- [x] T2 `tools/perf/gen-home.mjs`: synthetic homes of N sessions x K KB
- [x] T3 Measure and decide — result below: no optimization needed in these ranges

## Measurements (Windows 11, release build, window 1393x747; CPU = process CPU time over the interval, as % of one core; scroll = mouse wheel ticks over the pane every 50 ms for 6 s)

| Scenario | Idle (8 s) | Scrolling the pane (6 s) | Working set |
|---|---|---|---|
| 20 chats (baseline) | 1.8 % | 3.9 % | 230 MB |
| 1000 chats x 130 KB | 4.1 % | 5.2 % | 257 MB |
| 5000 chats x 4 KB | 5.3 % | 11.7 % | 389 MB |
| 300-message markdown thread (after the turn) | 2.9 % | 14.1 % | 281 MB |

- 1000 chats loaded within the first seconds (cold read ~0.7 s, off the UI thread); 5000 chats listed and scrolled normally (screenshot checked).
- Memory is the visible cost of a big list: about +30 KB per sidebar row (+160 MB at 5000), on top of a ~230 MB baseline.

## Decision

No optimization now: CPU stays low and nothing stalled in any scenario. Not done on purpose: virtualized/lazy sidebar rows and thread paging.

## What this does NOT prove

- CPU is a proxy: there is no frame-time or input-latency measurement, so a stutter that costs little CPU would not show.
- Thread test used 300 messages of ~400 characters; a thread with very large messages, huge code blocks or many images was not tried.
- A re-render per streamed delta in a long thread was not measured separately (the flood arrives in one burst).
- Idle CPU of 2-6 % of a core is not nothing for a desktop app; the 5 s refresh tick and iced's redraws are the likely sources, not investigated.

## Findings worth acting on (added to the backlog)

- With thousands of chats, finding one by scrolling is the real problem, not speed: **search/filter in the sidebar** moves up in the backlog (item 6: it already listed a project filter).
- A long unbroken word in a title (a URL, a path) is clipped at the sidebar edge instead of wrapping; harmless, minor.

## Log

- L1: "si" (merge PR #5 and continue with item 4). Scratch homes and measurement scripts were temporary files outside the repo and were deleted; the window-automation script is not kept (brittle); only the home generator is.
