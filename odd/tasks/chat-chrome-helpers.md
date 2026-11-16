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
- [ ] T2 (S1, S2, S5) web: the Working pulse in `Mark.svelte`/`app.css`; the
      header Rename (reusing the row's inline-rename flow) + the switch pinned
      right and responsive. TDD; suites green.
- [ ] T3 (S3, S5) web: the HelpersPanel two-column rebuild per the refreshed
      artboard; list selection keeps the thread beside it; keyboard walk kept.
      TDD; suites green.
- [ ] T4 (close) battery + real-window check + roadmap note.

## Log

- L1 (maintainer, 2026-11-16): the four design decisions of the survey (pulse
  build; timestamp keep; helpers two-column rebuild; row menu stays + header
  Rename + right-aligned responsive switch), recorded in
  `design/gentle-desk-v2/decisions.md` (merged as PR #174). "arranquemos ahora"
  starts this feature.
