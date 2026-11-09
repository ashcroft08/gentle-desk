# tauri-a11y-primitives: the accessible-primitives check for the Tauri frontend (T2c, S6)

Decided whether Bits UI or Melt UI passes a real accessibility check for the three
widgets the app needs: a modal dialog (focus trap, Esc closes, focus returns to the
trigger), a list of options (keyboard arrows, selected state), and a menu (open from
a button, arrow keys, Esc returns focus). Versions tried, on `feat/desktop-a11y`
with a next `apps/desktop` (Svelte 5 / Vite / Vitest):

1. `bits-ui` 2.20.6 - Svelte-5-native components (runes, snippets, bindable props).
2. `@melt-ui/svelte` 0.86.6 - builder API (stores + `use:melt` directives) from the
   Svelte-4 era; needs the separate preprocessor `@melt-ui/pp` 0.3.2 for every
   `use:melt` directive (its absence fails every render with
   "[MELTUI ERROR]: The `use:melt` action cannot be used without MeltUI's
   Preprocessor").

Both were added as local dependencies of `apps/desktop`, one sample per widget per
candidate under `apps/desktop/src/a11y-lab/<bitsui|meltui>/` (one component per
file, under the 200-line lint), mounted in `src/main.ts` only when the URL hash is
`#a11y-lab/bitsui` or `#a11y-lab/meltui` through two dynamic imports; the normal
app path is untouched in that case. Tests (Vitest + testing-library + user-event,
jsdom) live next to the samples: 6 per candidate, 12 total. `tools/probe.mjs` is
the CDP driver for the real-window probe (no source edits; see below).

## Candidates

| axis | bits-ui 2.20.6 | @melt-ui/svelte 0.86.6 (+ @melt-ui/pp 0.3.2) |
|---|---|---|
| Svelte 5 | first-party: components, snippets, `$state`/`bind:` | stores/actions from the Svelte-4 era; compiles under Svelte 5 but its peer range stops at `^5.0.0-next.118` |
| extra package required besides the library | none | `@melt-ui/pp` (preprocessor) added to `svelte.config.js` |
| dialog | `Dialog.Root/Trigger/Portal/Content/Title/Description/Close`; FocusScope traps Tab, returns focus on close | `createDialog()` builder + `use:melt` stores; document-level Esc, focus return on close |
| list of options | no bare listbox; `Select` carries it (trigger role=combobox, popup role=listbox) | its internal listbox builder is NOT in its public exports (no `createListbox`); the closest public surface is `createSelect` (same combobox/listbox shape) |
| menu | `DropdownMenu.Root/Trigger/Content/Item` | `createDropdownMenu()` builder (elements trigger/menu/item) |
| selection write-back | requires `bind:value` on `Select.Root`; a read-only `value` prop silently keeps the old selection on Enter | store-driven (`selectedLabel`), no binding needed |

## Results

### jsdom assertions (Vitest + testing-library + user-event; 12 tests)

All 12 pass (6 per candidate). The suite starts RED (12/12 fail) with inert
skeletons before the libraries are wired, then GREEN:

| widget | assertion | bits-ui | melt-ui |
|---|---|---|---|
| dialog | role=dialog with `aria-modal="true"`, title reachable | PASS | PASS |
| dialog | Esc closes; focus returns to the trigger | PASS (state asserted via `data-state=closed`/unmounted; see caveats) | PASS |
| list | popup shows role=listbox with 3 options; an `aria-selected` option chosen-state | PASS | PASS |
| list | ArrowDown moves the highlight (`data-highlighted` twice); Enter commits | PASS (with `bind:value`; selection lands on the highlighted option) | PASS (arrows advance from the SELECTED option, not from nothing; the second ArrowDown lands on Gamma) |
| menu | button opens role=menu with 3 menuitems | PASS | PASS |
| menu | ArrowDown moves the highlight; Esc closes; focus returns to the trigger | PASS | PASS |

Two library-visible behaviours worth noting (both hit during GREEN):

- bits-ui Select ignores selection commit when `value` is passed read-only; the
  sample needed `bind:value`. In a real app the store must be bound, so this is
  wiring, not a defect.
- melt-ui does not export its listbox builder; an option widget through melt's
  public API goes through `createSelect`, despite it being called a listbox task.

### Real-window UIA probe (WebView2, `apps/desktop/tools/a11y-check.ps1`, one per candidate)

Setup used for both probe runs, no source edits: `npm run build`, then
`cargo build -p gentle-desk-app --release --features tauri/custom-protocol`
(plain `cargo build -p gentle-desk-app` dev/`--release` WITHOUT that feature boots
the `devUrl` localhost:5173 server instead of the embedded dist; the feature is
what makes a release build embed `dist`), then launch the real exe with
`WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9222` and navigate
the running webview over the CDP debugger (`Page.navigate`) to
`http://tauri.localhost/index.html#a11y-lab/<candidate>`, wait for the WebView2
lazy UIA tree settle, open the widgets from CDP `Runtime.evaluate`, and run
`a11y-check.ps1 -AppPid <pid>` once. Both windows verified alive: `windowName:
"gentle-desk"`. Every process was terminated (`taskkill /PID <pid> /T /F`) and
`tasklist /FI "IMAGENAME eq gentle-desk-app.exe"` reported no task running
afterwards.

bits-ui (pid 30196, probe taken with the dialog open):

```json
{"named":11,"textOnWindow":false,"namesSample":["gentle-desk","gentle-desk","gentle-desk: contenido web","gentle-desk","a11y lab: bits-ui","Dialog","Open dialog","List","Alpha","Menu","Actions"],"gridOrList":0,"textPattern":2,"timedOut":false,"total":19,"windowName":"gentle-desk","maxDepth":9}
```

melt-ui (pid 8332, probe taken with the list popup open):

```json
{"named":16,"textOnWindow":false,"namesSample":["gentle-desk","gentle-desk","gentle-desk: contenido web","gentle-desk","a11y lab: melt-ui","Dialog","Open dialog","List","Lab list","Lab list","Alpha","Menu","Actions","Alpha","Beta","Gamma"],"gridOrList":4,"textPattern":3,"timedOut":false,"total":25,"windowName":"gentle-desk","maxDepth":9}
```

Interpreting the two snapshots: the probe is a single-state snapshot and these
two libraries close popovers on outside clicks/blur, so the one allowed probe per
candidate captured (a) bits-ui with its dialog open - the count is dialog-and-
chrome only, `gridOrList: 0` because its list popup was already closed - and
(b) melt-ui with its list popup open - the three options are named UIA elements
and `gridOrList: 4` is the listbox plus its 3 options. The melt snapshot also
exposes the off-screen widget sources: menu/dialog triggers remain named buttons
in the control view after close. A DOM-state dump collected per run (via CDP)
covered the not-currently-open widgets; per-widget UIA numbers for a SECOND open
popover of the same candidate were not sampled under the one-probe rule.

## Recommendation

`bits-ui` is the candidate to take into T3+.

- Equal accessibility result in jsdom on all 12 assertions, so accessibility does
  not decide between them today.
- bits-ui is Svelte 5 first-party; melt-ui is a Svelte-4 architecture (stores +
  a required third-party preprocessor not owned by the library, `@melt-ui/pp`
  0.3.2 from May 2024) with no released 1.0.0. The S6 one-component-per-file and
  typed-bridge layout fits component declarations better than store assembly.
- bits-ui ships its own trap, Esc and focus-return machinery with no additional
  packages; melt adds `@melt-ui/pp` and, through it, mandatory build plumbing.
- For the 50k list both libraries expose only their select-type carrier over a
  listbox (no bare listbox in either public API), so this axis is a tie and the
  sidebar's virtualized list will keep its row markup plain (that list is not a
  popover listbox anyway).
- If a future task needs a bare listbox (multi-select pickers, for example),
  melt-ui's unexported listbox builder is a real gap; bits-ui's is the same gap,
  so the deciding factor remains maintenance posture, not feature breadth.

## Caveats

- The UIA probe counts the CONTROL view via `TreeWalker`, capped at 12000 nodes
  and depth 30, one window per run, per the count logic of the copied
  `a11y-check.ps1`; it sees names where WebView2 chose to expose them ("a11y
  lab" headings reached UIA in both runs), but it does not navigate patterns or
  inspect the full element view.
- jsdom is not a screen reader and not a layout engine: the lab tests stub
  jsdom's missing ResizeObserver/matchMedia/pointer-capture and a viewport-
  sized rect (otherwise floating-ui's hide middleware switches popovers to
  `visibility: hidden`, and bits-ui reads empty `getClientRects()` as "anchor
  hidden" - test-environment artifacts, not library behaviour). bits-ui keeps
  closing popovers mounted awaiting a CSS `transitionend` that jsdom never
  fires, so the Esc-close assertions assert the delivered state
  (`data-state=closed` or complete unmount) plus focus return, with
  `waitFor` covering asynchronous portal effect timing.
- One snapshot per candidate: popovers close on outside clicks/blur, so only
  bits-ui's dialog and melt-ui's list were open at their respective probe
  moments (the numbers above reflect exactly that; the other widgets are covered
  by the jsdom suite and the DOM-state dump, not by UIA).
- The probe start URL was set via `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS` +
  CDP `Page.navigate` only; no source changed for probing, and no
  `tauri.conf.json` default was altered. CDP driving needs Node >= 22 (global
  WebSocket), which the toolchain here has (Node 24).
- Both candidates pass today's assertions; real screen-reader behaviour
  (NVDA/Accessibility Insights) and WebView2 version behaviour are out of scope
  for this check and stay unchecked by it.
