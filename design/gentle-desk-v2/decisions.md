# gentle-desk-v2 — design decisions ledger

The maintainer's rulings on the parts of the chosen v2 design that the built app
followed, dropped or drifted from, recorded 2026-11-16 so no future session
re-argues them. Evidence for each is the artboard selector plus the code path.

## Followed as designed

- **Per-chat status marks**: shape set, colour roles, end-of-row placement,
  "only when the chat has one", screen-reader names — built from
  `01a-tree-chat.html` verbatim (`icons/shapes.ts`, `Mark.svelte`,
  `ChatRow.svelte`).

## Deliberate additions (not in the artboards, kept by decision)

- **Row timestamp**: the chat row shows the chat's age (`modified_ago`) beside
  the title. Inherited from the pre-v2 session list; kept as useful information.
  The artboards' rows carry title + mark only.
- **Row `...` menu**: the chat's actions (Rename / Copy file path / Close chat /
  Delete) live in the sidebar chat row, revealed on hover/focus/menu-open.
  The artboard `01a` drew them in the chat HEADER; the feature doc that built
  them mis-cited the artboard (`sessions-rename-delete.md` S4). This ledger is
  the correction: the row is where the actions live.

## To build (maintainer-approved 2026-11-16)

- **Working pulse**: `01a`'s `m-working` pulse animation (1.2s ease, reduced-motion
  honoured) reaches `Mark.svelte` — the glyph is drawn today but never animated.
- **Chat header**: gains the **Rename** action (the row keeps its full menu), and
  the **Chat | Helpers** switch is right-aligned in the header and responsive on
  narrow windows.
- **Helpers two-column layout**: rebuild per `01b-tree-helpers.html` — the
  helper list (`minmax(200px,280px)`) and the helper thread ALWAYS visible side
  by side, list left with a right border, thread right — replacing the iced-era
  "thread replaces the list" swap. Artboard refresh on the canvas first (with the
  controls that exist today: Back; no Follow/Tools/Stop), then a feature of its
  own.

## Dropped by decision (recorded where they were made)

- Helper thread bar controls: **Follow live** and **Show tool details** are not
  drawn (iced always follows and always shows tools, T7b-1), and there is no
  helper **Stop** — the only control is Back, named for what it does. Recorded
  in `helpers/HelperThread.svelte` and `tauri-migration-m19.md`.
- The chat header's own `...` menu (Rename/Delete) is not drawn today; Rename
  arrives in the header per "To build" above.
