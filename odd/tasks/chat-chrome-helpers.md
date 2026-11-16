# chat-chrome-helpers: the chat header gains its actions, the Helpers view returns to two columns

The maintainer's 2026-11-16 rulings (design/gentle-desk-v2/decisions.md, made after
the design-gap survey): the Working pulse comes back, the chat header gains Rename
with a right-aligned responsive Chat|Helpers switch, and the Helpers view rebuilds
to artboard 01b's two-column layout. The row `...` menu and the row timestamp stay.
Design refresh on the gentle-design canvas BEFORE any Svelte (S6 of the house
rules).

## Specs

- S1 (pulse, decision "Construir el pulso"): `01a-tree-chat.html`'s Working mark
  animates again — the artboard's `m-working` pulse (1.2s ease), honouring
  `prefers-reduced-motion`. The glyph, colour and placement do not change.
- S2 (header, decision verbatim): "Que quede en la fila pero tambien agreguemos en
  el header del chat lo de renombrar y tambien el switch de chat|helper debe estar
  pechado a la derecha y ser responsive". So: the row `...` menu stays as is; the
  chat header gains **Rename** (opens the same inline-rename flow the row uses);
  and the **Chat | Helpers** switch sits pinned to the right edge of the header
  and behaves on narrow windows (no crush, no overflow).
- S3 (helpers two-column, decision "Reconstruir a dos columnas"): rebuild the
  Helpers view per `01b-tree-helpers.html` — the helper list column
  (`minmax(200px,280px)`, right border) and the helper thread ALWAYS visible side
  by side; selecting a helper shows its thread in the right column without
  replacing the list. The thread bar keeps today's controls (Back; no
  Follow/Tools/Stop — recorded drop). Artboard refresh FIRST.
- S4 (records, already done): the row timestamp is a deliberate addition and the
  row menu's placement correction live in `design/gentle-desk-v2/decisions.md`
  (PR #174) — no code change here.
- S5 (bilingual, standing rule): new copy in both columns with parity + flip
  tests; nothing new is expected beyond small labels.

## Tasks

- [ ] T1 (design) canvas refresh: `01b-tree-helpers.html` (two columns with
      today's controls) and the header part of `01a-tree-chat.html` (Rename in
      the header, switch right-aligned) — maintainer review on the canvas
      before any Svelte. Route: parent.
- [x] T2 (S1, S2, S5) web — DONE, commit 8ece406. Writer report: S1 (Mark.svelte
      now draws 01a's per-state class beside `mark`; app.css gains the pulse
      (1.2s ease-in-out, keyframes 50% opacity .35) with the reduced-motion
      kill); S2 (Header.svelte draws 01a's header: pencil with the artboard's
      Lucide outline inline (no pencil glyph in shapes.ts), local inline-rename
      flow — Enter commits trimmed, Escape cancels, blur commits, unchanged
      sends nothing — reporting through a new `onRename` prop that App.svelte
      wires to the row's own `renameSession(foreground_chat, name)`; ViewTabs
      switch stays last with the spacer pinning it right; 900px responsive step
      ported); S5 (`renameChat` "Rename chat"/"Renombrar el chat" + `chatName`
      reuse, parity + same-tick flip tests). TDD: RED 27 failed/9 passed, GREEN
      36 passed; svelte-check 0, eslint clean.
- [x] T3 (S3, S5) web — DONE, commit b95033f. Writer report: HelpersPanel
      rebuilt to 01b's two columns — the list column always drawn (counters
      above the listbox, right border, `minmax(200px,280px)` with the artboard's
      1000px → 200px collapse, scoped styles), the selected helper's thread
      beside it swapping in place, empty-chat chats keep the plain line;
      HelperThread no longer takes focus on mount (the list keeps the keyboard,
      arrow walk works with a thread beside it); the bar stays Back + heading +
      mark, Back still dispatches the existing CloseHelper (no new intents);
      copy unchanged (S5 nothing new). The one obsolete App-level pin ("thread
      replaces the list") was swapped to "the list is drawn beside the thread"
      with the parent's approval. TDD: RED 6 failed/42 passed, GREEN 48;
      validation battery 711/711, svelte-check 0, eslint clean. Open for the
      real-window check: the right column is blank when nothing is selected
      (no placeholder copy was added), and Back's CloseHelper is visually a
      selection-clear with the list always drawn.
- [ ] T4 (close) battery + real-window check + roadmap note.

## Log

- L1 (maintainer, 2026-11-16): the four design decisions of the survey (pulse
  build; timestamp keep; helpers two-column rebuild; row menu stays + header
  Rename + right-aligned responsive switch), recorded in
  `design/gentle-desk-v2/decisions.md` (merged as PR #174). "arranquemos ahora"
  starts this feature.
