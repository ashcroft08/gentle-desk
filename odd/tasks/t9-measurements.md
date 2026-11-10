# T9 measurements (m19)

Date 2026-11-10. Release build of `gentle-desk-app` (`--features tauri/custom-protocol`),
real window, fake shell (`tools/fake-gentle-shell`), isolated profile (APPDATA,
LOCALAPPDATA, USERPROFILE), one run per row, no repetitions. Observer: Windows UI
Automation, which makes WebView2 build its accessibility tree and so adds work of its
own; every number below includes that cost unless it says otherwise.

## 1. 50 000 chats (`tools/perf/measure-startup.ps1`, home from `tools/perf/gen-home.mjs`)

| Measure | Cold (no cache) | Warm (cache of the cold run) | Spike reference |
|---|---|---|---|
| Window visible | 198 ms | 231 ms | ~183 ms |
| First sidebar text / first row | 857 / 1510 ms | 950 / 1246 ms | n/a |
| List settled ("Reading chats" gone) | 11.2 s | 11.3 s | n/a |
| Working set at rest (app + WebView2 tree) | 495 MB (peak 528) | 487 MB (peak 468) | ~390 MB |
| CPU over two idle 10 s windows | 0 s, 0.031 s | 0 s, 0.031 s | ~0.02 |
| Rows alive in the tree while scrolling | 26 to 36 of 50 000 | 26 to 36 | n/a |
| UIA elements at rest / after scroll | 72 / 61 | n/a | n/a |
| Session cache written | 28.3 MB, no rewrite at rest | n/a | n/a |

Reading: the sidebar stays windowed and idle is quiet. The warm start did not settle
faster than the cold one; the cause is NOT investigated (suspicion: the warm start
revalidates every file in the background; the user sees the first row at ~1.2 s either
way). The spike's memory number may have counted a different set of processes, so the
~100 MB difference is not called a regression.

Limits: single run per row; scroll CPU (3.09 s in 11.9 s) is dominated by the
script's own UIA queries and is not quoted as an app number; scroll used the UIA
ScrollPattern, not a real wheel.

## 2. Whole-tail push per text delta (`tools/perf/measure-stream.ps1`)

Fake shell `stream 20` (about 200 text deltas, one every 100 ms), one chat, window
open; once with a 2-message thread and once after `flood` (300 more messages, so the
pushed tail is the 50-message cap).

| Phase | Process tree CPU over the 22 s turn | CPU over the middle 15 s |
|---|---|---|
| Small thread | 3.00 s | 1.73 s |
| After flood (50-message tail) | 2.33 s | 1.34 s |

Reading: no evidence that pushing the whole tail per delta costs more as the tail
grows (it was not higher with the full tail). About 0.1 core is spent while streaming
in both cases. Backlog 23 is not escalated.

Limits: one run per phase, so 3.00 vs 2.33 may be warm-up or noise; flood messages
are short (~400 characters), so a tail of long messages is NOT measured; bytes per
push and the Svelte re-diff time were NOT measured (no page instrumentation); the
UIA observer inflates the numbers.

## 3. Accessibility probes (`tools/perf/probe-a11y.ps1`)

Small home, one chat open (`echo hola`).

- 14 interactive controls (buttons, edit, tree items, tabs, links, tree, document),
  all with an accessible name; none unnamed.
- Tab order, one full cycle from the window: Send, Skip to chat, New session, New
  workspace, the workspace tree (one roving stop), New chat in <workspace>, Theme
  button, Chat tab, Copy, Message field, back to Send. The Helpers tab is reached
  with the arrow keys of the tab list, not with Tab.
- The sidebar is exposed as Tree with TreeItems and per-row names ("<title> <age>").

Limits: UIA only, no screen reader was run; contrast numbers are not measured; focus
visibility is not judged by UIA. With 50 000 chats the sidebar exposed 72 UIA
elements, so a screen reader sees only the windowed rows.

## 4. Real shell (not the stand-in)

A new chat in a workspace, after its first real turn, appears in the sidebar with the
turn's title and "now". Not confirmed: which workspace header it sits under after the
list reorders (two captures could not scroll to the top), and whether the workspace
moving up the list after its first chat is intended.

## Not verified by the agent (needs the user's real mouse and keyboard)

Hover reveal, real click on the New workspace menu, native folder picker, Copy to the
clipboard, link click opening the browser, queued Stop, restart notice, warning
notices, the fatal bar, the editor dialog, textarea auto-grow, scroll stick, contrast
numbers, system theme under OS light mode, title bar of the four dark themes.
