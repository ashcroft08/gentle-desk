# UI stack spike: Tauri 2 (WebView2) vs iced 0.14.2, same backend crates

**Status: throwaway spike, DO NOT MERGE its code.** This document is kept here as the evidence behind choosing Tauri 2 over iced: the spike itself lived under `spikes/` and never entered the repository, and the iced app it compared against was deleted in T9 (#65). Branch `spike/tauri` (base `ff99a67`),
spike lives in `spikes/tauri/` (own Cargo workspace; `dist/` is plain HTML/JS;
`marked` vendored from npm). Machine: Windows, Rust 1.99.1, node 24.11.1,
WebView2 runtime 157. No commits made; `.gitignore` gained
`spikes/**/target/` and `spikes/**/node_modules/`; `spikes/Cargo.lock` gained
one line (`serde` under gentle-core) after running the parent-authorized build
commands in `spikes/` (it was behind the main lock; left for the parent to
keep or revert).

## Numbers

Every Tauri number is the sum of `gentle-desk-tauri.exe` + all of its
`msedgewebview2.exe` descendants (count in parens) + its stand-in `node.exe`
child when a chat is open. Every iced number is `gentle-desk.exe` (+ its
stand-in `node.exe` child, none while idle). Sampling:
`spikes/tauri/tools/sample.ps1 -ParentPid <pid> -Names … -SampleS 10`
(working set = sum of `WorkingSet64`, CPU = Δ processor seconds over the
window); launch/paint: `tools/paint-time.ps1`.

| # | Scenario | ICO | Tauri (WebView2) |
|---|----------|-----|------------------|
| 1 | First paint, 50 k sidebar (2 runs, ms) | 296, 289 (first-ever run of the session: 714) | 183, 172 (first-ever run of the session: 391; warm runs 151–191) |
| 2 | Idle WS (MB), 50 k rows, after ≥15 s | 297 / 330 | 389 / 397 (webviews 6–7 each run) |
| 3 | Idle CPU s / 10 s, same | 0.05–0.5 (steady state; see note) | 0.02 / 0.02 |
| 4 | CPU s / 10 s, backend `stream 10` via real `chat_host` bridge (cannot be triggered unattended on iced) | not measured | 0.64 ; 1.11 (exe+webviews+node) |
| 5 | CPU s / 10 s, front-end markdown streaming, 10 s at 50 ms deltas, never-ending doc re-parse each delta (spike's strategy) | not measurable on real app; proxy `spike-iced` (iced markdown widget, scripted 25 ms deltas): 0.52 / 0.75 in its streaming-ish window | 5.33 / 5.44 |
| 6 | UIA elements / named / TextPattern elements from the main window | real app 1 / 1 / 0 (3 probes); spike-iced 1 / 1 / 0 (2 probes) | 62 / 45 / 8 + 27 list-like roles; thread and row text exposed |
| 7 | Build artifact | `gentle-desk.exe` 15.4 MiB | exe 8.66 MiB; NSIS 1.87 MiB; MSI 2.83 MiB |
| 8 | Build time | `-p gentle-ui` on warm deps 16.2 s; `spikes` workspace tree from genuinely cold: 1 m47 s | `cargo build --release` on warm target 24–27 s; `npx tauri build` incl. NSIS+MSI 57.5 s (WiX and NSIS downloaded once, successfully); genuinely cold total not captured (target was never fully clean while the clock ran) |

Notes on fairness, stated plainly:

- Idle comparisons are indicative, not apples-to-apples: the Tauri sidebar was
  50 k synthetic in-memory chats (its spike's scenario), the iced app a real
  50 k-session home read from disk (`node tools/perf/gen-home.mjs <tmp>/home
  50000 4`, `--home <tmp>/home`, throw-away `LOCALAPPDATA`). Different memory
  sources, different parse work.
- Row 3: iced needs ~10–25 s of busy first pass on a 50 k home (10.3–10.4 CPU s
  inside its first 15–25 s window, 1.16 in a tail window, then 0.05–0.5 steady).
  Only steady-state idle is compared.
- Row 5 is not workload-equal (iced proxy streams ~400 chars of scripted
  markdown once; the Tauri scenario re-parses a growing ~100 KB document 200
  times). It shows the out-of-the-box cost of the chosen render strategy on
  each stack; a web "render only the last paragraph" fix would cut it sharply.
- Row 4 could not be run on iced: sending a prompt needs real keyboard/mouse.

## Architecture proof (Tauri)

`open_chat` (`ChatKey::new_chat`) → `ChatHost::real(Limits::new(8, 2, 600 s))`,
`take_events` → one forwarder thread → `emit("chat-event", …)` per tagged
event; `send_prompt` → `host.send(key, RpcCommand::Prompt)`. Both commands
live in `spikes/tauri/src-tauri/src/backend.rs` (116 lines) on
`gentle_backend::chat_host` unchanged. FAKE_SHELL_LOG proved the prompt reached
the stand-in (`"message":"stream 10"`, child `node.exe fake.mjs --mode rpc`
parented by the exe). Nothing of `crates/backend` moved or changed; no real
model or real shell used anywhere.

What had to be rewritten from `crates/ui`: the whole emit/bridge glue (~470
lines of Rust+JS+CSS in this spike, replacing iced's `chat_runtime::execute`
scheduler and subscriptions). `crates/ui/src/chat_runtime` (862 lines incl.
tests) and `chat_keys` (185 lines) contain **zero iced imports** — they would
move out of the iced crate mechanically, not be rewritten.

## What is NOT measured (and could not be, inside this spike's rules)

- Streaming in the real iced app: no documented flag/CLI to trigger a prompt,
  and input injection would use the user's keyboard/mouse (forbidden). Row 4/5
  are therefore stack-proxy numbers, each and both declared above.
- Selection, IME, and dialog flows by hand (accents, dead keys, composed
  input, select-and-copy); not automatable without user input. WebView2 is the
  browser's own text engine; iced 0.14.2 `text.rs`/`markdown.rs` contain no
  selection code at all (grep over the registry sources; selection only exists
  in `text_editor`/`text_input`).
- Real screen-reader UX (NVDA/JAWS) — only the UIA sample above; control view
  capped at 12 000 nodes/depth 30. WebView2 exposes its DOM lazily (first
  probe still empty ~7 s after launch; full tree from loop 2 on). The
  virtualized list exposes only its rendered rows (26 DOM rows for 50 k).
- Long-run memory drift, DPI scaling, low-power/GPU contention, multi-monitor,
  MSI/NSIS install run, macOS/Linux, audio/VSync settings of either stack.
- The iced app's own first-pass cost is real measured work on 416 MB of
  synthetic session files; the Tauri spike could not load the same home
  without porting the session-index code, which was out of scope.

## Migration cost estimate (measured line counts)

| Piece | Lines | Files | Fate |
|---|---|---|---|
| `crates/gentle-core` + `crates/backend` + `crates/app` | 16 584 | 123 | survives untouched |
| `crates/ui/src/chat_runtime` | 862 | 11 | move out of the iced crate (no iced imports) |
| `crates/ui/src/chat_keys` | 185 | 2 | move out of the iced crate (no iced imports) |
| `crates/ui` rest (view/, app glue, windowing, thread_cache, subscription, text, time_ago, …) | ≈4 717 | ≈58 | rewritten or dropped (iced-only rendering lives in `view/`, 1 341 lines) |
| New web-side glue | ≈480 | 8 | new (this spike's `backend.rs`, `dist/*`; marked vendored, no framework) |
| Dependency tree | 447 lines (`spike-iced`) | — | Tauri: 594 lines + node 24 toolchain, @tauri-apps/cli, WiX/NSIS bundling |

## Risks of the evidence

- Two runs per number, same machine, release builds; but single CPU package,
  single session, modest sample count; the machine had unrelated apps running
  (counted only the measured process trees, never by name).
- First paint is "first visible non-iconic window", not the presented frame.
- The iced streaming row is a proxy in a different app binary; the Tauri
  markdown row bakes in my chosen (full-document) re-render strategy.
- WebView2 `dev.gentleai.tauri-spike` user-data folder (~8.9 MB) was created by
  the spike under `%LOCALAPPDATA%`; the spike never touched the user's real
  gentle-desk data (iced ran with a throw-away LOCALAPPDATA).

## Recommendation (not a decision)

On this evidence, Tauri looks like the better frontend for gentle-desk **if**
the deciding weights are text selection/IME/accessibility — the only row where
the gap is categorical, not a matter of tuning — plus a visibly faster first
paint (151–191 ms vs 282–714 ms) and bundle size (8.7 MiB exe + ~2 MiB
installer vs 15.4 MiB exe). Against Tauri: a heavier toolchain (node in the
dev loop, WiX/NSIS bundling, deeper dependency tree), a bigger idle working
set (≈390 vs ≈300–330 MB) and a rewrite of ≈4 717 lines of iced-crate code,
of which ≈1 000 lines simply move out. Against iced: no text selection and no
OS accessibility surface in 0.14.2's text/markdown widgets, and its 50 k-home
first pass costs ~10 CPU s before going idle. Both stacks idle cheaply
(≤0.5 CPU s / 10 s). Nothing measured here disqualifies either; the
accessibility and selection gap is the one thing the numbers say most clearly,
and that call belongs to the project owner.
