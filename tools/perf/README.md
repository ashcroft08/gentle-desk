# tools/perf

Harnesses that measure the built desktop app from the outside (UI Automation),
against a synthetic pi home. They never build anything and never touch the real
home: each writes a throw-away `USERPROFILE`/`APPDATA`/`LOCALAPPDATA` under its
`-Scratch` folder and points the app at a home you generate.

## The synthetic home

```pwsh
node tools/perf/gen-home.mjs .verify/home-open 5 130   # 5 sessions, ~130 KB each
```

Every session's header names `cwd` `C:/dev/proj<N>`, so the sidebar shows one
workspace per project even when that folder does not exist on the machine.

## measure-open.ps1 — how long a stored chat takes to show content

```pwsh
pwsh -NoProfile -File tools/perf/measure-open.ps1 `
  -PiHome .verify/home-open -Session 'chat 0' `
  -Scratch "$env:TEMP\gentle-desk-open" -OutJson "$env:TEMP\open-1.json"
```

Reports, in the JSON it also prints:

| key | meaning |
| --- | --- |
| `open_ms` / `first_text_ms` | double-click on the sidebar row → first text the window exposes that was not there before the click (the thread answered) |
| `first_content_row_ms` | → first transcript row, i.e. content and not just the opening line (`chat-open-latency.md` S1) |
| `painted_ms` | → that row count stops growing, including `-SettleSeconds` (1 s by default) |
| `empty_frame_before_content` | whether text without any transcript row was exposed before content (T2) |

Reuse the same `-Scratch` for the second run on purpose: the session index cache
the first run wrote is then loaded, which is the returning user's case.

The run uses `GENTLE_SHELL_BIN=tools/fake-gentle-shell/fake.mjs`, so the child
answers immediately: it measures the app's own path to first paint (the session
file pre-paint and the opening state), never a real `gentle-shell` boot.

### What the window needs (learned the hard way)

1. The binary is `target\release\gentle-desk-app.exe`. `gentle-desk.exe` is a
   stale build of an older package name whose window has no webview content.
2. Find the window by **process and class**, never by title. The app owns several
   top-level windows; the first is an invisible winit message window titled
   `Tao Thread Event Target` that exposes nothing. The real one has class
   `Tauri Window` and title `Gentle Shell Desktop` — titles ending in
   `gentle-desk` on a developer machine belong to Windows Terminal or a browser.
3. The scratch config must store `shell_home` (`"isolated"` here). The app asks
   the home question when pi is found, nothing is chosen yet and the launcher
   resolves (`bridge/shell.rs`, `ask`), and the welcome screen is then the only
   thing the window holds.
4. Do **not** name a workspace whose folder does not exist: the sidebar draws its
   header, its chats never appear, and there is nothing to open. Leave
   `workspaces` empty and let the home's own sessions build the list.
5. Keep the window visible and in front while the webview boots. A window left
   behind another one still paints its panes, but its DOM — and so its whole
   accessibility tree — never shows up, which is the "window found, zero
   elements" run.

## Other harnesses

- `measure-startup.ps1` — window/first-row/settle timings, idle CPU and working
  set, and sidebar scrolling for a big home. It still looks for a window titled
  `gentle-desk` and launches `gentle-desk-app.exe`; the title no longer matches
  what the app creates (see 2 above), so it needs the same window lookup.
- `measure-stream.ps1` — how the thread's rows arrive while a turn streams.
- `probe-a11y.ps1` — one-off dump of what the window exposes (named elements, tab
  order); it hardcodes one developer's checkout path.
