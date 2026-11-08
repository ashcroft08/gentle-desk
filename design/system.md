# gentle-desk - starter design system on the gentle-shell themes

**PROPOSED system, theme values sourced from the real theme files.** The user asked that the
PROPOSED starter system for this board (token board) use the SAME THEMES as gentle-shell.
gentle-shell's picker offers six entries: `system`, `dark`, `Gentle`, `Gentleman-Cute`,
`Gentleman-Sexy`, `light`. This document re-organises the starter tokens BY THEME.

Three of the six entries come from real, read theme files; the other three are built-in picker
entries with no theme file in the folder I was allowed to read, so their values stay exactly as
proposed in round 1 and remain labelled PROPOSED. Details under "Theme sources".

Status vocabulary, type scale, spacing, radius, shadows, focus and motion are unchanged from
round 1 and stay PROPOSED (no design sources exist in this repo; `crates/ui/src/view/helpers.rs`
status colours `0x7aa2f7`, `0x7edc9a`, `0xff7b7b` still hold for the entry that needs them, below).

## Theme sources (six picker entries, in picker order)

| Picker entry | Resolves to | Source file | State of values |
|---|---|---|---|
| `system` | `dark` or `light` via `prefers-color-scheme` | built-in; no theme file under `C:/Users/josep/AppData/Roaming/npm/node_modules/gentle-pi/themes/` | PROPOSED (uses this document's `dark` / `light` tables) |
| `dark` | dark built-in | built-in; no theme file found | PROPOSED (round-1 values, unchanged) |
| `Gentle` | - | `C:/Users/josep/AppData/Roaming/npm/node_modules/gentle-pi/themes/Gentle.json` | real values from the file |
| `Gentleman-Cute` | - | `C:/Users/josep/AppData/Roaming/npm/node_modules/gentle-pi/themes/Gentleman-Cute.json` | real values from the file |
| `Gentleman-Sexy` | - | `C:/Users/josep/AppData/Roaming/npm/node_modules/gentle-pi/themes/Gentleman-Sexy.json` | real values from the file |
| `light` | light built-in | built-in; no theme file found | PROPOSED (round-1 values, unchanged) |

Legend for the Source column in the tables below: `real: <terminal var>` means the value is that
variable in the named JSON file; `PROPOSED` means no source file was available and the value is a
round-1 proposal.

## Terminal variable to web token mapping (rule per choice)

gentle-pi theme files are two-layer: `vars` holds literal colours, `colors` maps semantic roles
onto them, and `export` names page/card/info backgrounds. Mapping:

| Web token | Taken from (terminal) | Rule |
|---|---|---|
| `--bg` | `bg` (also `export.pageBg`) | whole-page background in web = terminal full-screen background |
| `--surface` | `bgPanel` / `bgElement` (`export.cardBg`) | terminal panels/chat cards become web panes: sidebar, thread cards, dialogs |
| `--surface-2` | `bgSubtle` (also `toolPendingBg` / `infoBg`) | terminal subtle fill (custom-message, pending-tool, info areas) becomes secondary fill: rows, composer, secondary buttons. In the Gentleman files `bgSubtle` equals `surface`, so Gentleman rows keep contrast through `--border`, not through fill - recorded, not changed |
| `--border` | `border` (`colors.border`) | pane/row/card separators only, never decorative |
| `--text` | `text` | body text, titles |
| `--muted` | `muted` (via `colors.muted`) | secondary text, timestamps, queued/opening badges, icons |
| `--faint` | `dim` (via `colors.dim`) | in pi, `dim` covers disabled and minimal thinking text; the nearest placeholder-only role |
| `--accent` | `colors.accent` (resolved per file) | running mark, primary button, focus ring, selection. Gentle resolves to literal `blue` `#7FB4CA`; Gentleman-Cute to `accent` `#F095C8`; Gentleman-Sexy to `accent` `#F43888` |
| `--success` | `colors.success` (resolved per file) | finished-unread badge, "Done" helper, ok tool call. Gentle: `green`; Cute: `mint`; Sexy: `pearl` |
| `--warning` | `colors.warning` | needs-attention badge, warnings, waiting helpers. Gentle: `DEBA87`; Gentleman files: `F2B86D` |
| `--danger` | `colors.error` (resolved per file) | ended badge, errors, failed helpers. Gentle: `red`; Gentleman files: `error` |
| `--accent-tint` | `userMessageBg` (real fill, not alpha) | selected rows, user bubbles, info-flavoured selected surfaces. Reason: terminal already tints the user's message area toward the accent hue per theme |
| `--success-tint` | `toolSuccessBg` (real fill) | success notice surfaces. In the Gentleman files this fill is near-neutral `#151316`, recorded as-is |
| `--danger-tint` | `toolErrorBg` (real fill) | error notices, model-error box |
| `--warning-tint` | no terminal source | KEPT DERIVED: `warning` at 14% over `--surface` (round-1 rule); no warning-area fill exists in the files |
| dark / light tints | PROPOSED | alpha formulas from round 1 (e.g. `accent @ 14% over surface`) - no built-in theme file was readable |

Rule kept from round 1: colour never carries state alone - every status also has a **shape**
(badge glyph, icon, prefix) and a **text label** (badge title, role name, `aria-label`). Never
invent a ninth meaning for the six status colours.

## Theme: system (follows prefers-color-scheme)

`system` is the first picker entry: it resolves to this document's `dark` table under
`prefers-color-scheme: dark` and to the `light` table under `prefers-color-scheme: light`.
Every token, badge and component below therefore inherits from those two tables; `system`
itself has no separate values. Usage note for the app: default the app palette to `system` and
let the user pin one of the six entries.

## Theme: dark (PROPOSED values - built-in entry, no source file readable)

| Token | Value | Usage | Source |
|---|---|---|---|
| `--bg` | `#0e1116` | app background behind all panes | PROPOSED |
| `--surface` | `#151a21` | panes: sidebar, thread cards, panels | PROPOSED |
| `--surface-2` | `#1c222b` | rows (hover/zebra), secondary buttons, composer | PROPOSED |
| `--border` | `#2b333d` | separators (not decorative) | PROPOSED |
| `--text` | `#e4e8ee` | body text, titles | PROPOSED |
| `--muted` | `#9aa4b2` | secondary text, timestamps, badges, icons | PROPOSED |
| `--faint` | `#7d8593` | placeholder text only (non-essential) | PROPOSED |
| `--accent` | `#7aa2f7` | running mark, primary button, focus, selection | PROPOSED (round-1 iced `0x7aa2f7`) |
| `--accent-tint` | accent at 14% over surface (`rgba(122,162,247,.14)`) | selected rows, user bubbles | PROPOSED |
| `--success` | `#7edc9a` | finished-unread badge, "Done" helper | PROPOSED (round-1 iced `0x7edc9a`) |
| `--success-tint` | success at 12% over surface | success notice surfaces | PROPOSED |
| `--warning` | `#e7b455` | needs-attention badge, warnings | PROPOSED |
| `--warning-tint` | warning at 14% over surface | warning notices | PROPOSED |
| `--danger` | `#ff8585` | ended badge, errors | PROPOSED (round-1 iced `0xff7b7b` tuned) |
| `--danger-tint` | danger at 14% over surface | error notices | PROPOSED |
| `--on-danger` (button text) | ADJUSTED `#0e1116` (was white `#ffffff`) | text on filled danger button | PROPOSED + ADJUSTED for contrast |

## Theme: Gentle (real file: `themes/Gentle.json`, read verbatim)

| Token | Value | Usage | Source |
|---|---|---|---|
| `--bg` | `#06080f` | app background | real: `bg` |
| `--surface` | `#06080f` | panes (same as bg in this file: `bgPanel` = `bgElement` = `bg`) | real: `bgPanel`/`bgElement` |
| `--surface-2` | `#0d0f14` | rows, composer, secondary buttons | real: `bgSubtle` |
| `--border` | `#313342` | separators | real: `border` |
| `--text` | `#F3F6F9` | body text, titles | real: `text` |
| `--muted` | ADJUSTED `#8394A3` (original `#5C6170`) | secondary text, timestamps, badges | real: `muted`, see contrast deviations |
| `--faint` | ADJUSTED `#96A2B0` (original `#5C6170` via `dim`) | placeholder text only | real: `dim`, see contrast deviations |
| `--accent` | `#7FB4CA` | running mark, primary button, focus, selection | real: `colors.accent` = `blue` |
| `--accent-tint` | `#161D2E` | selected rows, user bubbles | real: `userMessageBg` |
| `--success` | `#B7CC85` | finished-unread badge, "Done" helper | real: `colors.success` = `green` |
| `--success-tint` | `#1a2e1a` | success notice surfaces | real: `toolSuccessBg` |
| `--warning` | `#DEBA87` | needs-attention badge, warnings | real: `warning` |
| `--warning-tint` | warning at 14% over surface (derived `rgba(222,186,135,.14)`) | warning notices | PROPOSED rule, no source var |
| `--danger` | `#CB7C94` | ended badge, errors | real: `colors.error` = `red` |
| `--danger-tint` | `#2e1a1a` | error notices | real: `toolErrorBg` |
| `--on-danger` (button text) | ADJUSTED `#06080f` (was white `#ffffff`) | text on filled danger button | real + ADJUSTED for contrast |

Dark-family notes from the file, usable later: `syntaxComment`/`brightBlack` `#8394A3`,
`syntaxPunctuation` `#96A2B0`, `heading` `#B5B2D0`, `selection` `#232A40`.

## Theme: Gentleman-Cute (real file: `themes/Gentleman-Cute.json`, read verbatim)

Pink accent on a very dark plum background, per the user's description.

| Token | Value | Usage | Source |
|---|---|---|---|
| `--bg` | `#060407` | app background | real: `bg` |
| `--surface` | `#100A0F` | panes, sidebar, thread cards | real: `bgPanel`/`bgElement` |
| `--surface-2` | `#100A0F` (equals `--surface` in this file) | rows, composer, secondary buttons - rows read through `--border`, not fill | real: `bgSubtle` |
| `--border` | `#563040` | separators | real: `border` |
| `--text` | `#F6EFF3` | body text, titles | real: `text` |
| `--muted` | `#A78E9B` | secondary text, timestamps, badges | real: `muted` |
| `--faint` | ADJUSTED `#D7A0B8` (original `#76616B` via `dim`) | placeholder text only | real: `dim`, see contrast deviations |
| `--accent` | `#F095C8` | running mark, primary button, focus, selection | real: `accent` |
| `--accent-tint` | `#2A1523` | selected rows, user bubbles | real: `userMessageBg` |
| `--success` | `#B4E7C7` | finished-unread badge, "Done" helper | real: `colors.success` = `mint` |
| `--success-tint` | `#151316` | success notice surfaces | real: `toolSuccessBg` |
| `--warning` | `#F2B86D` | needs-attention badge, warnings | real: `warning` |
| `--warning-tint` | warning at 14% over surface (derived `rgba(242,184,109,.14)`) | warning notices | PROPOSED rule, no source var |
| `--danger` | `#FF718F` | ended badge, errors | real: `colors.error` = `error` |
| `--danger-tint` | `#261019` | error notices | real: `toolErrorBg` |
| `--on-danger` (button text) | ADJUSTED `#060407` (was white `#ffffff`) | text on filled danger button | real + ADJUSTED for contrast |

Sibling vars from the file kept available: `activePink` `#FFB1DD`, `champagne` `#E0C27A`,
`deepPink` `#C96AA2`, `selection` `#28121E`.

## Theme: Gentleman-Sexy (real file: `themes/Gentleman-Sexy.json`, read verbatim)

Same dark plum base as Cute; accent moves to hot pink `#F43888`.

| Token | Value | Usage | Source |
|---|---|---|---|
| `--bg` | `#060407` | app background | real: `bg` |
| `--surface` | `#100A0F` | panes, sidebar, thread cards | real: `bgPanel`/`bgElement` |
| `--surface-2` | `#100A0F` (equals `--surface` in this file) | rows, composer, secondary buttons - rows read through `--border`, not fill | real: `bgSubtle` |
| `--border` | `#563040` | separators | real: `border` |
| `--text` | `#F6EFF3` | body text, titles | real: `text` |
| `--muted` | `#A78E9B` | secondary text, timestamps, badges | real: `muted` |
| `--faint` | ADJUSTED `#D7A0B8` (original `#76616B` via `dim`) | placeholder text only | real: `dim`, see contrast deviations |
| `--accent` | `#F43888` | running mark, primary button, focus, selection | real: `accent` |
| `--accent-tint` | `#2E0F1F` | selected rows, user bubbles | real: `userMessageBg` |
| `--success` | `#D2CBD0` (pearl) | finished-unread badge, "Done" helper | real: `colors.success` = `pearl` |
| `--success-tint` | `#151316` | success notice surfaces | real: `toolSuccessBg` |
| `--warning` | `#F2B86D` | needs-attention badge, warnings | real: `warning` |
| `--warning-tint` | warning at 14% over surface (derived `rgba(242,184,109,.14)`) | warning notices | PROPOSED rule, no source var |
| `--danger` | `#FF718F` | ended badge, errors | real: `colors.error` = `error` |
| `--danger-tint` | `#261019` | error notices | real: `toolErrorBg` |
| `--on-danger` (button text) | ADJUSTED `#060407` (was white `#ffffff`) | text on filled danger button | real + ADJUSTED for contrast |

Sibling vars from the file kept available: `activePink` `#FF4F9A`, `champagne` `#E0C27A`,
`violet` `#C49BFF`, `deepPink` `#BF0F50`, `selection` `#28121E`.

## Theme: light (PROPOSED values - built-in entry, no source file readable)

| Token | Value | Usage | Source |
|---|---|---|---|
| `--bg` | `#f2f4f7` | app background behind all panes | PROPOSED |
| `--surface` | `#ffffff` | panes: sidebar, thread cards, panels | PROPOSED |
| `--surface-2` | `#e9edf2` | rows, secondary buttons, composer | PROPOSED |
| `--border` | `#cfd5dd` | separators | PROPOSED |
| `--text` | `#17202a` | body text, titles | PROPOSED |
| `--muted` | `#576272` | secondary text, timestamps, badges, icons | PROPOSED |
| `--faint` | `#6b7686` | placeholder text only | PROPOSED |
| `--accent` | `#2e5bd7` | running mark, primary button, focus, selection | PROPOSED (light re-derived in round 1) |
| `--accent-tint` | accent at 10% over surface | selected rows, user bubbles | PROPOSED |
| `--success` | `#166e38` | finished-unread badge, "Done" helper | PROPOSED |
| `--success-tint` | success at 10% over surface | success notice surfaces | PROPOSED |
| `--warning` | `#8a5800` | needs-attention badge, warnings | PROPOSED |
| `--warning-tint` | warning at 14% over surface | warning notices | PROPOSED |
| `--danger` | `#c22f37` | ended badge, errors | PROPOSED |
| `--danger-tint` | danger at 10% over surface | error notices | PROPOSED |
| `--on-danger` (button text) | `#ffffff` - PASSES (5.59) | text on filled danger button | PROPOSED, no adjustment |

## Contrast deviations (WCAG 2.1, computed; nothing changed silently)

AA thresholds used: 4.5:1 for text, 3.0:1 for non-text UI. Script run in a scratch directory,
deleted afterwards. Full per-theme numbers are in the task report; only failing pairs are listed
here, each with the kept ORIGINAL and the recorded ADJUSTED value:

| Theme | Pair (foreground on background) | Original | Original ratio | Verdict | ADJUSTED | Adjusted ratio |
|---|---|---|---|---|---|---|
| Gentle | `muted` on `surface` | `#5C6170` / `#06080f` | 3.24 | FAIL | `#8394A3` (`syntaxComment`/`brightBlack`) | 6.41 |
| Gentle | `muted` on `surface-2` | `#5C6170` / `#0d0f14` | 3.10 | FAIL | `#8394A3` | 6.14 |
| Gentle | `faint` on `surface` | `#5C6170` / `#06080f` | 3.24 | FAIL | `#96A2B0` (`syntaxPunctuation`) | 7.71 |
| Gentle | white text on `danger` (filled button) | `#ffffff` / `#CB7C94` | 3.07 | FAIL | `#06080f` (`--bg`) | 6.53 |
| Gentleman-Cute | `faint` on `surface` | `#76616B` / `#100A0F` | 3.44 | FAIL | `#D7A0B8` (`softRose`) | 8.96 |
| Gentleman-Cute | white text on `danger` (filled button) | `#ffffff` / `#FF718F` | 2.62 | FAIL | `#060407` (`--bg`) | 7.79 |
| Gentleman-Sexy | `faint` on `surface` | `#76616B` / `#100A0F` | 3.44 | FAIL | `#D7A0B8` (`softRose`) | 8.96 |
| Gentleman-Sexy | white text on `danger` (filled button) | `#ffffff` / `#FF718F` | 2.62 | FAIL | `#060407` (`--bg`) | 7.79 |
| dark (PROPOSED) | white text on `danger` (filled button) | `#ffffff` / `#ff8585` | 2.35 | FAIL | `#0e1116` (`--bg`) | 8.06 |

Notes:
- Adjusted values prefer sibling variables from the SAME real theme file (no invented hues):
  Gentle takes `#8394A3` and `#96A2B0` from its own vars block; the Gentleman files take
  `softRose` `#D7A0B8`. Filled-danger text switches from white to the theme's own `--bg`,
  matching the round-1 primary-button rule (accent bg + `--bg` text).
- `dark` faint `#7d8593` on `#151a21` passes at 4.70 but with almost no margin; `light` faint is
  4.60. Both kept (PROPOSED, not a real-file value), flagged as thin.
- The consequence rule changed and is now explicit: the round-1 component rule "danger bg +
  white text" is replaced by "danger bg + the theme's `--on-danger`" (dark text on dark-family
  themes, white only where it passes, currently the light entry).

## Status vocabulary (unchanged from round 1; colour values now per-theme)

Marks and text labels are IDENTICAL across all six themes; only the token value behind the
colour role changes per theme. Two states never differ by colour alone.

| State | Mark | Colour role (token per theme tables) | Text label (title/aria) | Extra cue |
|---|---|---|---|---|
| Working | `*` (glyph: solid dot) | `--accent` | "Working" | pulsing; "2 of 2 slots in use" meta |
| Finished-unread | `*`-variant (glyph: filled diamond) | `--success` | "New reply" | title bold; "reply ready" |
| Queued | `*` (glyph: three dots) | `--muted` | "Queued" | counters in title line ("waiting for a slot") |
| Opening | `*` (glyph: hollow circle) | `--muted` | "Opening" | caption "starting..." |
| Needs-attention | `!` | `--warning` | "Needs attention" | dialog counters + inline affordance |
| Ended | `x` (glyph: multiply sign) | `--danger` | "Ended" | notice with Restart + reason |

The glyphs themselves (`solid dot`, `filled diamond`, `three dots`, `hollow circle`, `!`,
`multiply sign`) carry the same names as round 1 (`solid dot, diamond, three dots, hollow circle, !, x`) and are drawn identically in
the artboards; the table above is given in ASCII so the vocabulary itself is text-readable.
Queued and Opening use the (possibly ADJUSTED) `--muted` value plus the `--border` outline from
each theme table, so Gentle's badge text stays readable (original would have been 3.10:1).

Legacy deviation, still needs routing through `crates/runtime` (S6): today `badge.rs` draws the
same working mark for both Working and Finished-unread; unread reuses `--success` plus a
distinct diamond mark and bold title. Backend keeps sending plain badge data; the mark rule
changes before T6. Helper statuses reuse the roles: Running=`--accent`, Waiting/Queued=`--muted`,
Done=`--success`, Failed/Cancelled=`--danger` - always as text labels, glyphs only decoration.

## Typography, spacing, radius, shadows, focus (unchanged, PROPOSED)

- System font stack only (no external fonts, ever):
  `-apple-system, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif`; code:
  `ui-monospace, "Cascadia Code", Consolas, monospace`.
- Scale: 11 (row meta), 12 (badges/meta), 13 (sidebar titles, buttons, composer), 14 (body),
  16 (pane/dialog headers), 20 (view titles); weight 600/700 for bold-unread and headings;
  line height 1.5 body, 1.2 headings.
- Spacing `4 / 8 / 12 / 16 / 24 / 32`; 8 inside groups, 12-16 between groups, 24 between pane
  sections; rows fixed at `--row-h: 34px` for the virtualized 50 000-chat list (S3).
- Radius `4 / 8 / 10 / 999`. Shadows: `--shadow-1` popovers, `--shadow-2` modals; elevation
  never hue. Shadows are theme-contextual dark-family values only for dark entries (PROPOSED);
  in `Gentle`/`Gentleman-*` they read over near-black `--bg`.
- Focus: `:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }` on every
  control. Modals: first-control focus, `Esc` closes, focus returns. All motion off under
  `prefers-reduced-motion` (working pulse 1.2 s included).
- Buttons rule updated by the contrast table above: primary = accent bg + `--bg` text; danger =
  danger bg + theme `--on-danger`; secondary = `--surface-2` + `--text`; ghost icon buttons get
  accessible names; disabled controls state why, never colour alone.
