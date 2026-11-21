# gentle-desk design system

Synced 2026-11-21 from `apps/desktop/src/theme.css` (every color, shadow and row
token, all six theme blocks) and `apps/desktop/src/app.css` (layout tokens,
typography, geometry, motion, component rules). The previous sync (2026-11-16)
predated `composer-dock`, `tool-card`, `welcome-screen`'s two views, the
`whats-new` modal, the Notifications section and the settings frame of 0.8.1, so
this pass re-read `app.css` whole and records those components for the first
time. Values are recorded exactly as the sources declare them; nothing invented.

Sources read, and never edited: `apps/desktop/src/theme.css`,
`apps/desktop/src/app.css`. One component rule is cited to the file that owns it
(`apps/desktop/src/helpers/HelperThread.svelte`). `theme.css` itself is unchanged
since the previous sync; `app.css` last changed 2026-11-20 (settings polish T4).

Theme selection is one `data-theme` value on `<html>`; `system` is only the
default of a run that has not chosen one (`theme.css`).

## Colors — theme `dark` (the default: `:root` and `[data-theme='dark']`)

Source: `apps/desktop/src/theme.css`.

| Token | Value | Usage |
| --- | --- | --- |
| `--bg` | `#0e1116` | window and main-area base; text on a filled accent control |
| `--surface` | `#151a21` | sidebar, composer box, menus, popovers, dialog cards, tool card |
| `--surface2` | `#1c222b` | hover fill of rows/buttons/icons, settings cards, notice and error bodies, inline code and code blocks |
| `--border` | `#2b333d` | 1px hairlines, inset hover rings, dashed separators, scrollbar thumb, gauge and bar tracks |
| `--text` | `#e4e8ee` | body copy and every primary label |
| `--muted` | `#9aa4b2` | secondary labels, hints, quiet status lines, icons at rest |
| `--faint` | `#7d8593` | tertiary text: placeholder, key hints, uppercase section labels, metadata lines, disabled tint |
| `--accent` | `#7aa2f7` | focus ring, selected text, links, filled primary action, focus borders, spinner top, context fill |
| `--tint` | `rgba(122, 162, 247, 0.14)` | selected row/tab/menu-item band, user bubble, gate card background |
| `--success` | `#7edc9a` | ok tag, success tag text, a tool card's done mark |
| `--warning` | `#e7b455` | warning notice edge, warning tag, reached limit, ≥75 % gauge tone, `tc-cut` border |
| `--danger` | `#ff8585` | error edge and title, destructive button, helper stop square, ≥95 % gauge tone, a failed call's first line |
| `--shadow` | `0 8px 24px rgba(0, 0, 0, 0.45)` | dialog cards, menus, theme popovers |
| `--row-h` | `34px` | fixed height of one sidebar tree row |

## Colors — theme `light` (`[data-theme='light']`)

Source: `apps/desktop/src/theme.css`. Same roles as the dark block.

| Token | Value | Usage |
| --- | --- | --- |
| `--bg` | `#f2f4f7` | window base |
| `--surface` | `#ffffff` | cards, menus, composer, sidebar |
| `--surface2` | `#e9edf2` | hover fills, settings cards, notice bodies, inline code |
| `--border` | `#cfd5dd` | hairlines, rings, separators |
| `--text` | `#17202a` | primary text |
| `--muted` | `#576272` | secondary text |
| `--faint` | `#6b7686` | tertiary text |
| `--accent` | `#2e5bd7` | focus, selection, primary action |
| `--tint` | `rgba(46, 91, 215, 0.1)` | selected band, user bubble, gate card |
| `--success` | `#166e38` | ok tag |
| `--warning` | `#8a5800` | warning edge and tag |
| `--danger` | `#c22f37` | danger edge, destructive action |
| `--shadow` | `0 8px 24px rgba(23, 32, 42, 0.18)` | cards, menus, popovers |
| `--row-h` | `34px` | one sidebar row |

## Colors — theme `Gentle`

Source: `apps/desktop/src/theme.css`.

| Token | Value | Usage |
| --- | --- | --- |
| `--bg` | `#06080f` | window base |
| `--surface` | `#06080f` | sidebar and cards, flush with the base |
| `--surface2` | `#0d0f14` | hover fills and card bodies |
| `--border` | `#313342` | hairlines and rings |
| `--text` | `#f3f6f9` | primary text |
| `--muted` | `#8394a3` | secondary text |
| `--faint` | `#96a2b0` | tertiary text |
| `--accent` | `#7fb4ca` | focus, selection, primary action |
| `--tint` | `#161d2e` | selected band, user bubble, gate card |
| `--success` | `#b7cc85` | ok tag |
| `--warning` | `#deba87` | warning edge and tag |
| `--danger` | `#cb7c94` | danger edge, destructive action |
| `--shadow` | `0 8px 24px rgba(0, 0, 0, 0.6)` | cards, menus, popovers |
| `--row-h` | `34px` | one sidebar row |

## Colors — theme `Gentleman-Cute`

Source: `apps/desktop/src/theme.css`.

| Token | Value | Usage |
| --- | --- | --- |
| `--bg` | `#060407` | window base |
| `--surface` | `#100a0f` | sidebar and cards |
| `--surface2` | `#100a0f` | hover fills and card bodies (same value as `--surface`) |
| `--border` | `#563040` | hairlines and rings |
| `--text` | `#f6eff3` | primary text |
| `--muted` | `#a78e9b` | secondary text |
| `--faint` | `#d7a0b8` | tertiary text |
| `--accent` | `#f095c8` | focus, selection, primary action |
| `--tint` | `#2a1523` | selected band, user bubble, gate card |
| `--success` | `#b4e7c7` | ok tag |
| `--warning` | `#f2b86d` | warning edge and tag |
| `--danger` | `#ff718f` | danger edge, destructive action |
| `--shadow` | `0 8px 24px rgba(0, 0, 0, 0.6)` | cards, menus, popovers |
| `--row-h` | `34px` | one sidebar row |

## Colors — theme `Gentleman-Sexy`

Source: `apps/desktop/src/theme.css`. Shares every value with `Gentleman-Cute`
except `--accent`, `--tint` and `--success`; `--shadow` is identical.

| Token | Value | Usage |
| --- | --- | --- |
| `--bg` | `#060407` | window base |
| `--surface` | `#100a0f` | sidebar and cards |
| `--surface2` | `#100a0f` | hover fills and card bodies (same value as `--surface`) |
| `--border` | `#563040` | hairlines and rings |
| `--text` | `#f6eff3` | primary text |
| `--muted` | `#a78e9b` | secondary text |
| `--faint` | `#d7a0b8` | tertiary text |
| `--accent` | `#f43888` | focus, selection, primary action |
| `--tint` | `#2e0f1f` | selected band, user bubble, gate card |
| `--success` | `#d2cbd0` | ok tag (a neutral, not a green) |
| `--warning` | `#f2b86d` | warning edge and tag |
| `--danger` | `#ff718f` | danger edge, destructive action |
| `--shadow` | `0 8px 24px rgba(0, 0, 0, 0.6)` | cards, menus, popovers |
| `--row-h` | `34px` | one sidebar row |

## Colors — theme `system` (`[data-theme='system']`)

Source: `apps/desktop/src/theme.css`. Dark values above; under
`@media (prefers-color-scheme: light)` these apply instead.

| Token | Value | Usage |
| --- | --- | --- |
| `--bg` | `#f2f4f7` | window base when the reader prefers light |
| `--surface` | `#ffffff` | cards, menus, composer, sidebar |
| `--surface2` | `#e9edf2` | hover fills, settings cards, notice bodies, inline code |
| `--border` | `#cfd5dd` | hairlines, rings, separators |
| `--text` | `#17202a` | primary text |
| `--muted` | `#576272` | secondary text |
| `--faint` | `#6b7686` | tertiary text |
| `--accent` | `#2e5bd7` | focus, selection, primary action |
| `--tint` | `rgba(46, 91, 215, 0.1)` | selected band, user bubble, gate card |
| `--success` | `#166e38` | ok tag |
| `--warning` | `#8a5800` | warning edge and tag |
| `--danger` | `#c22f37` | danger edge, destructive action |
| `--shadow` | `0 8px 24px rgba(23, 32, 42, 0.18)` | cards, menus, popovers |
| `--row-h` | `34px` | one sidebar row |

`theme.css` also sets `color-scheme`: `dark` for `dark`, `Gentle`,
`Gentleman-Cute`, `Gentleman-Sexy`; `light` for `light`; `light dark` for
`system`.

## Layout tokens

| Token | Value | Usage | Source |
| --- | --- | --- | --- |
| `--chat-max` | `880px` | max width of `.thread-in` and `.composer-dock`, so the box lines up with the thread; ~75 characters at the base font | `app.css` (`:root`) |
| `--settings-chrome` | `200px` | what the Settings card's own title and Close row take out of the window, so `.settings-body` can be bounded at `calc(100vh - 200px)` | `app.css` (`.dlg.settings`) |
| `--row-h` | `34px` | one sidebar tree row | `theme.css` |

## Component-scoped properties

Custom properties the app's code sets on an element; none has a fixed value in
CSS.

| Token | Value | Usage | Source |
| --- | --- | --- | --- |
| `--swatch` | set per item (`var(--swatch, var(--accent))` fallback) | the accent dot beside a theme's name in the picker list, `.tpl .swatch` | `app.css`, set by `apps/desktop/src/sidebar/ThemePicker.svelte` |
| `--dot` | set per dot (`var(--dot)`) | one 6px palette dot of a theme card, `.tc-dot` | `app.css`, set by `apps/desktop/src/settings/ThemeCards.svelte` |
| `--d` | set per cell, a delay in ms | stagger of the wordmark's typing animation, `.wm-cell`/`.wm-shadow` | `app.css`, set by `apps/desktop/src/brand/WordMark.svelte` |
| `--mono` | read with a fallback, never declared (`var(--mono, ui-monospace, …)`) | the monospace stack of `.mode-fail code` | `app.css` |

## Typography

- Base: `font: 14px/1.5 -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue',
  Arial, sans-serif` on `body` (`app.css`). UI text inherits it; `button, input,
  select, textarea` re-declare `font: inherit` and `color: inherit`.
- Monospace: `ui-monospace, 'Cascadia Code', Consolas, monospace` (`app.css`; the
  theme cards' miniature uses `ui-monospace, 'SF Mono', Consolas, monospace` and
  the welcome card's paths `ui-monospace, 'Cascadia Mono', Consolas, monospace`).
  Used at 9px (a theme miniature), 10.5–13px elsewhere: code, paths, model names,
  versions, tool-card command and output (12.5 / 12.2), the context rail's
  percentage (10.5).
- Scale in use (`app.css`): 9px (theme miniature), 10.5px (context rail label,
  session-mode caption), 11px (tags, table headers, uppercase labels, attachment
  chips, session-mode button), 11.5px (mode note, usage window label and reset
  line, tool-card footer and "+N more", home chat age), 12px (hints, secondary
  lines, footers, pills, key hints — the most used size), 12.2px (tool-card
  output), 12.5px (quiet pane lines, subtabs, provider names, dialog path and
  consequence lines, tool-card command, thinking row), 13px (base UI text, menus,
  rows, notices, buttons, activity line, reasoning text), 14px (body, header title
  under 900px, helper thread name), 15px (dialog titles, pane headings, profile
  name), 16px (markdown h2, welcome h2, whats-new title), 18px (markdown h1),
  `clamp(18px, 2.2vh + 10px, 24px)` (the welcome title), 20px (new-session
  title), 26px (welcome card h1).
- Weights: 600 for titles, section labels, filled actions and selected tabs;
  nothing else is heavier than body.
- Section labels are 11–12px uppercase with `letter-spacing: 0.05em`–`0.08em` in
  `--faint`, or 12px/600 in `--muted` with `letter-spacing: 0.02em` for sidebar
  group headers, pane headings and the tool card's tool name.
- Line heights outside the base: 1.2 (row sub-line), 1.3 (markdown headings),
  1.35–1.4 (menu items, session-mode description, usage window), 1.5 (hints,
  consequence lines, reasoning text), 1 (preview chip, context chip, whats-new
  version pill, tool-card kind chip), 1.55 (a dialog's consequence line).

## Spacing

- Rhythm declared across `app.css`: 1 / 2 / 3 / 4 / 5 / 6 / 7 / 8 / 9 / 10 / 12 /
  14 / 16 / 18 / 20 / 22 / 24 / 26 / 28 / 30 / 32 / 36px.
- Gaps: 8px is the house default (13 declarations — buttons row, row icon slot,
  composer, dialog actions, tool-card header); 6px is the composer row's own
  rhythm; 10px for notice/fatal rows and the header title line; 4px 10px for
  wrapping metadata rows; 18px between thread messages and between the settings
  rail and pane.
- Card padding: dialogs 18px, settings cards 12px 14px / 12px 16px, tool card
  8–10px, whats-new header 20px 24px 14px and notes 16px 24px, welcome card
  32px 36px, welcome choice 18px 20px.
- Inset text offsets: dock lines (`.why`, `.keys`, `.drop`, `.refused`) take
  `padding-left: 16px` so they align with the composer's text; notices pad
  `8px 8px 8px 12px`; the thinking row's text pads `0 0 0 20px`.
- Settings geometry (`app.css`): card `min(680px, calc(100vw - 32px))`, padding
  18px, 14px gaps; body = rail 184px + 14px right padding + 1px border, 18px gap,
  pane taking the rest; search field 28px tall; rail entries 32px; theme grid 3
  columns at 10px gaps; the app window's minimum is 800×560
  (`apps/desktop/src-tauri/tauri.conf.json`).

## Radius

4px (inline code, user-bubble corner, tool-card chevron, mode-fail code), 5px
(a message's copy button, the tool card's kind chip and duration chip), 6px (small
chips, menu items in a popover, tooltips, inline rename fields, skip link,
attachment chips), 7px (a theme miniature, session-mode options and buttons), 8px
(buttons, rows, inputs, icon buttons, rail tabs, subtabs, segmented controls), 10px
(cards, notices, menus, popovers, tool card, `.new`, thumbnails, welcome found,
welcome choice, gate, session-mode popover, markdown `pre`), 12px (settings cards,
provider list, popover list, welcome choice), 14px (dialog cards, user bubble),
16px (welcome card), 18px (composer), 999px (pills, tags, dots, thumbs, bars, send
button, install bar), 1–3px (context rails, gauges, bars), 50% (activity dots,
spinners, theme swatches), `8px 8px 0 0` (folder tabs).

## Shadows and elevation

- `--shadow` (`0 8px 24px …` per theme) on `.dlg`, `.menu`, `.tpl`, and the
  settings dialog's card.
- Fixed shadows, not the token: `.popover` `0 8px 24px rgba(0, 0, 0, 0.4)`,
  `.mode-pop` `0 8px 24px rgba(0, 0, 0, 0.35)`, the floating sidebar
  `8px 0 24px rgba(0, 0, 0, 0.45)`.
- Scrims: `.ovl` `rgba(0, 0, 0, 0.5)` at z-40 with the dialog card at z-41; the
  narrow-window sidebar's `.scrim` `rgba(0, 0, 0, 0.45)` at z-30 with the sidebar
  at z-40.
- Inset ring `inset 0 0 0 1px var(--border)` is the house hover outline for rows,
  rail tabs, menu items and icon buttons; the danger/warning/info notice edge is
  `inset 3px 0 0` in its tone, and the model error box is a 1px danger border plus
  a 3px inset danger edge.
- Z order: menus 15, tooltips 20, theme list 25, popovers / session-mode popover /
  skip link / sidebar scrim 30, dialog scrim and floating sidebar 40, dialog card
  41.

## Motion

- Transitions are 120ms ease and only on hover/selection: rail tabs, subtabs,
  theme list items, theme cards, row timestamps, settings buttons, the composer's
  session-mode chevron, the copy button's opacity (100ms).
- Keyframes: `wm-type` / `wm-shadow` 460ms ease-out (wordmark typing), `pane-in`
  120ms (settings pane swap), `pulse` 1.2s ease-in-out infinite (the Working
  mark, 50 % opacity 0.35), `activity-pulse` 1.2s staggered 0.2/0.4s (three dots,
  0.25→1 opacity), `install-progress-travel` 1.6s ease-in-out infinite (npm wait
  bar, a 32 % bar travelling its 4px track), `tc-spin` 0.9s linear infinite (a
  tool card's running state; also the working row's spinner and the session
  mode's pending dot).
- `@media (prefers-reduced-motion: reduce)` kills every transition and animation;
  the Working mark keeps its still glyph, the install bar becomes a still full
  track at opacity 0.6.

## Component rules

- **Shell** — `.app` is a grid of `280px minmax(0, 1fr)`; `.app.scollapsed` and
  `.app.soverlay` become one column, and `.soverlay .side` floats at
  `min(280px, 85vw)` over a `.scrim`. `.side` is `--surface` with a 1px right
  border and `padding: 12px 10px 8px`; `.side-toggle` is a 30px square, radius
  8px, hover `--surface2`. `.foot` is bottom-anchored (`margin-top: auto`), 6px
  gaps, top border, and its icon buttons suppress the hover ring. `.update` row:
  `--surface2`, bottom border, 13px, `padding: 8px 16px`; its bar is 160×6 and its
  two buttons are radius 8px `padding: 3px 10px` at 12px (`.update-go` filled
  accent, `.update-quiet` transparent with `--border`). The gear's `.dot` is 6px
  of `--accent` at `top/right: 5px`, decoration only.
- **Chat header** (`.mh`): 52px tall, `padding: 0 20px`, bottom border, h1 15px/600
  with ellipsis; at ≤900px it tightens to 48px / `0 12px` and the title drops to
  14px. Inline rename (`.mh .rn`) is 26px high, `--bg` fill, 1px accent, radius
  6px.
- **Chat pane** (`.chat-pane`): one tree with two faces. With rows it is the
  thread over the composer; `.chat-pane.is-fresh` centres the hero and the
  composer above the middle (`justify-content: safe center`, `padding: 24px 24px
  clamp(24px, 8vh, 96px)`) and hides the empty thread, and its `.composer-dock`
  narrows to `min(560px, 100%)`. `.new-session` is the hero: rose (a canvas,
  18px under), title 20px/600, `.new-session-where` 12px `--muted`.
- **Warm start** (`.home`): the centred column under the header — rose with 30px
  under it, `.home-title` `clamp(18px, 2.2vh + 10px, 24px)`, `.home-where` with
  the workspace in 600 and the dim part in `--muted`, `.home-go` (accent-filled,
  radius 10px, `padding: 9px 16px`, 600; `:disabled` turns to `--surface2` with
  `--faint` text), `.home-reason` in `--warning`, `.home-recents` at
  `min(400px, 100%)` with an 11px uppercase `.home-kicker` whose inset matches the
  rows' 10px, `.home-chat` rows (13px, radius 8px, `padding: 7px 10px`, hover
  `--surface2`) and `.home-others`/`.home-quiet` as quiet links. A window shorter
  than 600px starts at the top and scrolls; under 640px the sides drop to 16px and
  the one action stretches.
- **Thread** — `.thread` `padding: 28px 24px 12px`, scrolls; `.thread-in` is a
  column at `max-width: var(--chat-max)` with 18px gaps. A user message `.msg.u`
  is `--tint`, `border-radius: 14px 14px 4px 14px`, `padding: 8px 14px`,
  `max-width: 78%`, right-aligned; an assistant message `.msg.a` is
  `padding: 0 4px` of plain text. `.copy` is a 24px text-action that only exists
  under the pointer or when focused (14px stroke glyph, radius 5px).
- **Markdown** (`.md`): blocks `margin: 0 0 12px`; h1/h2/h3-h6 are 18/16/14px with
  `margin: 16px 0 8px` and line-height 1.3; inline `code` is `--surface2`, radius
  4px, `padding: 1px 5px`, 13px; `pre` is `--surface2` with a 1px `--border`,
  radius 10px, `padding: 10px 12px`; `blockquote` takes a 2px `--border` left edge,
  `padding-left: 12px`, `--muted`; links are `--accent`; table cells are 1px
  `--border` with `padding: 4px 8px`.
- **Tool card** (`.tool-card`, `design/tool-card/`): ONE quiet card —
  `--surface` fill, 1px `--border`, radius 10px, `overflow: hidden` (a slice, not
  a scroll box). Header `.tc-head` at `padding: 8px 10px`, `min-height: 34px`, with
  a 14px stroke glyph in `--muted`, `.tc-tool` 12.5px/600 `letter-spacing: 0.02em`,
  a monospace `.tc-chip` 11px on `--surface2`, and `.tc-cut` (11.5px, `--warning`
  border) marking a payload that did not arrive whole. The command `.tc-cmd` is
  monospace 12.5px `--text` on `--surface2` between two hairlines, prefixed `$ `
  for a shell; `.tc-out` is 12.2px `--muted` with one `.tc-line` block per line
  and the first line of a failure in `.tc-line-error` (`--danger`); `.tc-more` is
  a full-width 11.5px `--faint` row counting the folded lines; `.tc-foot` is
  11.5px with tabular numbers, a 13px mark (`--success`, `.bad` `--danger`) and a
  `.tc-spin` while it runs.
- **Reasoning row** (`.think-row`): NOT a card — inline, `align-self: flex-start`,
  12.5px `--muted`, an accent spark, a duration chip (`--surface2`, radius 5px,
  tabular), a 12px chevron that rotates 90° at `aria-expanded`, and the text
  behind a 2px `--border` left edge at `margin-left: 20px`.
- **Working row** (`.work`): one faint line (`padding: 0 4px`, 11.5px) with a 9px
  spinner and the elapsed seconds in tabular numbers, and at its right the
  session-mode button (11px, radius 999, transparent 1px border; `.hot` turns it
  `--danger` with a `currentColor` border). `.mode-pop` is a fixed 292px card at
  radius 10px with `.mode-opt` rows on a `14px 1fr` grid; `.mode-conf` is the same
  card asking one question with a `.mode-btn.go` in danger; `.mode-fail` is one
  11px `--danger` line under the row.
- **Model error** (`.err`): `--surface2` fill, 1px `--danger` border plus a 3px
  inset danger left edge, radius 10px, `padding: 10px 12px`; its title `.et` is
  600 in `--danger`. The fatal launcher bar (`.fatal`) reuses that language across
  the window top.
- **Placeholder / empty states**: `.placeholder` is `--muted` on
  `padding: 28px 24px`; `.empty` is `padding: 18px 10px` with a 13px/600 `.et`
  and a 12px `--muted` `.notice-line` indented 10px.
- **Activity**: `.activity` is 13px `--muted` with three 4px dots (3px gaps) that
  pulse in sequence — reduced motion leaves three still dots.
- **Notices**: `.notices` stacks with 8px gaps at `margin: 10px 20px 0`;
  `.notice` is `--surface2`, 1px `--border`, radius 10px,
  `padding: 8px 8px 8px 12px`, 13px, with `.err`/`.warn`/`.info` adding the 3px
  inset edge in `--danger`/`--warning`/`--accent`.
- **Buttons** (`.btn`): 1px `--border`, `--surface2` fill, `--text`, radius 8px,
  `padding: 6px 14px`; hover `border-color: --accent`; `:disabled` opacity 0.55.
  Variants: `.primary` (accent border; in the whats-new modal and settings panes
  filled accent with `--bg` text and weight 600), `.d` (danger border+fill, `--bg`
  text). Icon buttons (`.ic`): 32px square, transparent, `--muted`, radius 8px,
  hover `--surface2` + `--text` + inset 1px ring; `aria-disabled` = opacity 0.4
  with no hover; `[data-tip]` draws a `--text`-on-`--bg` 12px tooltip, radius 6px,
  6px below the button (`.tip-r` aligns it right). The composer's send/stop circle
  is 36px at radius 999; `.ic.fill` is accent-on-`--bg` and brightens with
  `filter: brightness(1.1)` on hover; `.ic.danger` colours the glyph `--danger`.
- **Composer dock**: `.composer-dock` is `width: min(var(--chat-max), calc(100% -
  48px))` centred with `margin: 0 auto 20px`. `.composer` is a column with 12px
  horizontal rhythm, 1px `--border`, radius 18px, `--surface`, `padding: 10px 12px
  8px 16px`, and a 2px accent outline on `:focus-within`. Its textarea is
  transparent, 24px minimum to 120px maximum, `line-height: 1.5`, `--faint`
  placeholder; a drag over it (`.over`) turns the border accent and the fill
  `--tint`.
  - The composer's ONE row under the input is `.row` and therefore collides with
    the sidebar row, so it resets height, padding, radius, background, shadow and
    cursor and keeps its own 12px layout. `.grp` groups keep a 6px rhythm with no
    group hover; `.vsep` is a 1px × 14px `--border` hairline; `.folder` is the
    only 600 thing in the row (12px `--text`, ellipsized) and `.branch` sits beside
    it in 12px `--muted`.
  - `.att` is the 26px clip button (radius 6px, `--faint`, hover `--surface2`).
    `.chips` holds the attached files: 11px chips, radius 6px, 1px `--border`,
    `padding: 2px 4px 2px 6px`, name in `--text` and an `.x` that turns `--text`;
    a file outside the chat's folder is `.out` (a `color-mix` warning border and
    warning text), `.more` is dashed and quiet, and one `.warn` / `.attach-failed`
    line says it in `--warning` / `--danger`.
  - Under it: `.why` (12px `--muted`), `.keys` (12px `--faint`), `.drop` (12px
    accent), `.refused` (12px `--danger`), each inset 16px.
- **Context rail** (`.composer-dock .ctx-rail`, `design/composer-dock` S1/S5): a
  measure above the box, outside its border — `margin: 0 10px 10px`, a flex label
  row at `padding-bottom: 4px` in 10.5px `--faint` with tabular numbers (the
  percentage and a clock at the left, the window at the right) over a 2px
  `--border` track whose `--accent` fill keeps `min-width: 4px`. `.warning` turns
  the fill `--warning`, `.danger` `--danger`, so the number always carries the
  meaning too.
- **Model chip**: `.pill` is border-less, radius 10px, `padding: 3px 6px`, 12px
  `--faint`, hover `--surface2`; the value is 600 when the pill is not `.faint`,
  and `.pill .g-model` tints the select's glyph `--accent`. `.popover` opens
  upward (`bottom: calc(100% + 4px)`), `--surface`, 1px border, radius 12px,
  `box-shadow: 0 8px 24px rgba(0,0,0,0.4)`, `padding: 4px`, `max-width: 320px`,
  `max-height: 300px`; its `.filter` is 200px, radius 8px with an accent border on
  focus; `.option` is 12px, radius 8px, `padding: 6px 8px`, with `.active` on
  `--surface2` and `[aria-selected='true']` in `--accent`; a `.hint` inside is
  11px `--faint`.
- **Attachments**: `.thumb` is 72×72 with `object-fit: contain`, radius 10px, 1px
  `--border`, `--surface` fill; `.take` removes one at 16px radius 999 over
  `color-mix(in srgb, var(--bg) 78%, transparent)`, turning `--danger` on hover.
- **Sidebar**: wordmark cells fill `--accent` and animate in with
  `wm-type 460ms ease-out var(--d) backwards`, each over a shadow cell of
  `color-mix(in srgb, var(--accent) 24%, transparent)`. `.new` is the 38px
  full-width New session button: 1px `--border`, radius 10px, `--surface2`, weight
  600, hover accent border, `aria-disabled` opacity 0.55 with the border held.
  `.wh` pane header `margin: 14px 0 6px`, `padding-left: 10px`, h2 12px/600
  `--muted` `letter-spacing: 0.02em`; `.gh` group heading `padding: 14px 10px 4px`
  in the same style.
- **Sidebar rows** (`.row`): height `var(--row-h)` (34px), radius 8px,
  `padding: 0 10px`, 13px; hover `--surface2` + inset 1px ring; selected
  `background: --tint`; folder rows weight 600, chat rows `padding-left: 42px` and
  a real 44px right slot for the actions icon; `.sub` is 12px `--muted` at
  line-height 1.2 and `.ago` 12px `--muted`, which fades to opacity 0 while the
  row's actions show. Actions icons are absolutely placed (`right: 6px` for the
  new-chat, `right: 38px` for the workspace menu) and revealed on row hover,
  focus-within or `menu-open`. Inline rename (`.row .rn`) is 22px high, `--bg`,
  1px accent, radius 6px.
- **Working / stop marks**: `.mark` is a 16px stroke glyph (`stroke-width: 2`,
  round caps, `currentColor`); `.mark.m-working` adds
  `animation: pulse 1.2s ease-in-out infinite`, one state class only, so the five
  still marks never move. A helper thread's stop square (`.stop`,
  `apps/desktop/src/helpers/HelperThread.svelte`) is a 32px radius-8px button that
  sits in `--danger` and whose hover *is* the fill (`background: --danger`, glyph
  `--bg`, no ring); while its helper is not running it goes `--faint` and keeps
  its place, with no hover fill.
- **Menus** (`.menu`): `--surface`, 1px `--border`, radius 10px, `--shadow`,
  `padding: 6px 0`, `min-width: 210px` (chat menu 190px; chat and workspace menus
  capped at `calc(280px - 24px)`); items are full-width, left-aligned, radius 0,
  `padding: 7px 12px`, 13px, line-height 1.4, nowrap; hover and
  `[data-highlighted]` fill `--tint`; a danger item (`.d`) takes `--danger` text
  only.
- **Theme picker list** (`.tpl`): opens above its trigger in the sidebar
  (`bottom: 38px`) and below in the dialog (`top: 38px`); `--surface`, 1px
  `--border`, radius 10px, `--shadow`, `padding: 4px`, `min-width: 180px`. Items
  are `padding: 7px 10px`, radius 6px, 13px, with a 120ms background/color
  transition; hover `--surface2` + inset ring; `[aria-selected='true']` is
  `--tint` with `--accent` text. The `.swatch` dot is 10px, `background:
  var(--swatch, var(--accent))`, ringed with `inset 0 0 0 1px rgba(255,255,255,0.16)`.
- **Dialogs** (`.dlg`): fixed centre, `width: min(440px, calc(100vw - 32px))`
  (settings `min(680px, …)`, whats-new `min(560px, …)`), `--surface`, 1px
  `--border`, radius 14px, `padding: 18px`, `--shadow`, over the `.ovl` scrim.
  Title `.dt` 15px/600 `margin-bottom: 6px`; labels 12px/600 `--muted`
  `margin: 14px 0 6px`; fields (`.dlg input`, `.dlg .field`) are full width with
  a 1px `--accent` border, radius 8px, `padding: 7px 10px`, `--bg` fill, 13px;
  `.acts` is right-aligned flex with 8px gaps and `margin-top: 16px` (the settings
  card zeroes that margin, since its own 14px gap already separates the row). A
  dialog's consequence line `.why` is 13px at line-height 1.55 with only its scope
  in `--text`/600; `--danger` when the line reports a failure. `.path` is monospace
  12.5px `--muted`; a `details` disclosure is a plain list of 12.5px `--muted`
  rows with dashed `5px 2px 5px 14px` separators and an accent summary line that
  underlines on hover.
- **Whats-new modal** (`.dlg.whatsnew`): `padding: 0`, `max-height: min(560px,
  calc(100vh - 48px))`, header `20px 24px 14px` over a bottom border, title 16px, a
  version pill (999px, `--tint`, accent, monospace 600 12px) and a 12px `--faint`
  "since" line pushed right. Its language tabs are folder tabs: radius
  `8px 8px 0 0`, `padding: 5px 13px`, 12.5px `--muted`, and the selected tab takes
  `--surface` with `--accent` text and weight 600 while sitting 1px lower to cover
  the separator. Notes paragraphs are `--muted` under 11px/600 uppercase `--faint`
  headings (`letter-spacing: 0.06em`), each list item marked by a 6px accent dot
  at 14px gaps, `hr` a dashed rule; actions sit in a `14px 24px 20px` footer over a
  top border with a 12px `--faint` hint pushed left.
- **Settings dialog** (`.dlg.settings`, `design/settings-polish/`): a 680px card
  with 14px gaps; `.settings-body` puts a fixed 184px rail beside the pane
  (`gap: 18px`) and is bounded at `max-height: calc(100vh - var(--settings-chrome))`.
  `.settings-side` is `flex: none; width: 184px; padding: 2px 14px 0 0` with a 1px
  right border, 12px gaps. `.settings-find` is the search field above the rail:
  28px tall, 1px `--border`, radius 8px, `--surface2`, 9px horizontal padding, a
  14px magnifier in `--faint`, a 12.5px input that declares its own `padding: 0`
  (the generic `.dlg input` padding would otherwise overflow the 28px box), and
  `border-color/outline` accent at `:focus-within`. `.settings-none` is the one
  `--muted` 12.5px line when a query leaves no entry.
- **Settings rail**: `.settings-rail` is a 2px-gap column; each tab is 32px,
  `padding: 0 10px`, radius 8px, 13px `--muted` with an 8px glyph gap, a 120ms
  background/color transition, hover `--surface2` + `--text`, selected `--tint` +
  `--accent`, and a 1px inset accent outline on focus. It is a vertical tablist:
  one tab stop, arrows/Home/End walk it, and the pane it controls is
  `.settings-pane`.
- **Settings pane**: `.settings-pane` takes the rest, `min-height: 0`,
  `overflow-y: auto`, `scrollbar-gutter: stable`, `padding: 2px 0 0 2px`; its
  `.pane-body` fades in with `pane-in 120ms`. Heading `h3` 15px/600,
  `.hint` 12px `--muted` with `margin-bottom: 14px`, `.why` 12px `line-height:
  1.5`, `.quiet` 12.5px `--muted`, `.note` 12px `--faint` italic, `.acts`
  left-aligned with 8px gaps and `margin: 12px 0 0`, and `.pane-label` as the
  visible 12px/600 `--muted` name over a control. `.btn.accent` is the filled
  accent action (`--bg` text, 600, hover `brightness(1.08)`); an `aria-disabled`
  button keeps its place at opacity 0.55 with no hover border change.
- **Theme cards** (`.tc-grid` / `.tc`): three equal columns at 10px gaps, always
  (the card is 680px at every allowed window). Each `.tc` is a button with
  `padding: 7px`, 1px `--border`, radius 10px, transparent fill, `--muted` text,
  6px gaps; hover `--surface2`, `aria-pressed='true'` takes an accent border and
  `--tint` fill, focus a 1px inset accent outline. Its `.tc-shot` is the
  miniature: `padding: 6px`, 1px `--border`, radius 7px, `background: var(--bg)`,
  monospace 9px, carrying its own `data-theme` so every colour in it is that
  theme's token (prompt `--accent`, text `--text`, `ok` `--success`, path
  `--muted`); `.tc-dot` is 6px of `var(--dot)` (7 per card, `danger warning success
  accent muted faint text`), `.tc-name` is 11.5px ellipsized, and `.tc-check` is a
  13px accent stroke check on the chosen one.
- **Settings segmented controls** (`.settings-pane .seg`): one 1px `--border`
  shared by its options at radius 8px; options are `padding: 5px 12px` at 12.5px
  `--muted`, hover `--surface2`, checked fills `--accent` with `--bg` text and 600,
  keyboard focus a 1px `--text` inset outline. The Profile panes' `.subtabs` are
  the quiet pair instead: 1px `--border`, radius 10px, `padding: 2px`, options
  `padding: 4px 14px` at 12.5px, selected `--tint` + `--accent` + 600 with an inset
  accent ring. (Language and Notifications use `.seg`; Shell home still uses the
  older `.home-choice`, `padding: 4px 10px` at 12px with a `--tint` pressed state.)
- **Settings cards**: `.profile`, `.providers`, `.card.updates`, `.card.ext` and
  `.card.usage-provider` are `--surface2` cards with a 1px `--border` and radius
  12px (`padding: 12px 14px` / `12px 16px`). The profile's `.pname` is 15px/600
  `--accent` over a 12px `--faint` `.pmark`, and its `.pair` rows are `dt` labels at
  11px/600 `--faint` uppercase (`letter-spacing: 0.05em`, 92px wide) with the model
  in monospace 12px `--muted` and `.think` in accent monospace.
- **Provider rows** (`.providers` / `.provider`): one `--surface2` card,
  `padding: 4px 14px`, radius 12px; each row is a wrapping flex at
  `gap: 4px 10px`, `padding: 7px 0`, with a dashed `--border` separator between
  rows. `.provider-name` is monospace 12.5px `--text`; the state is a `.tag`; the
  `.reason` is `flex-basis: 100%`, 12px `--faint` (and 12px `line-height: 1.5`
  inside the Updates rows).
- **Update rows** (`.urows` / `.urow`): the provider rhythm without a container —
  `padding: 7px 0`, dashed separators, `.headline` 13px/600, `.mono` 12px
  `--muted`, `.versions` as the card's 12px monospace `--faint` metadata line,
  `.gate` a radius-10px `--tint` band in 12px `--muted`, 10px of space between two
  cards of the same pane.
- **Usage gauge**: `.ubar` is a 76×5 radius-3px track (`--border`) with a
  `min-width: 4px` fill in `--accent`, `--warning` or `--danger` by share, beside
  a monospace 12px/600 `.upercent` (38px) and a 11.5px `--faint` `.ureset`.
  `.ulabel` is a 52px monospace 11.5px label; a reached limit turns its name
  `--warning` as well as carrying its tag; `.uwindow` rows are a non-wrapping flex
  at `padding: 3px 0 3px 10px`.
- **Tags** (`.tag`): 11px/600 pill, `padding: 3px 8px`, radius 999,
  `1px solid currentColor`; tone from `.ok` `--success`, `.warn` `--warning`,
  `.quiet` `--muted`, `.bad` `--danger`.
- **Tables** (`.agents`): 13px, headers 11px/600 uppercase `--faint`
  (`letter-spacing: 0.05em`) over a bottom border, cells `padding: 5px 8px`, the
  model column monospace 12px `--muted` with ellipsis, the effort column right
  aligned at 64px, and rows highlighted on hover or `focus-within`.
- **Welcome / first run**: `.welcome-card` is `min(820px, 100%)` on `--surface`
  with a 1px `--border`, radius 16px, `padding: 32px 36px`; h1 26px, h2 16px with
  a 10px glyph gap; `.welcome-found` is a radius-10px bordered strip; the two
  choices are a 2-column grid (one column at ≤720px) of radius-12px cards
  (`padding: 18px 20px`, 10px gap) with the recommended one edged in `--accent`;
  `.welcome-badge` is an 11px accent pill in `--bg` text; `.welcome-go` is the
  accent-filled 10px-radius action (`padding: 10px 14px`, 600) with a quiet
  transparent variant and 0.6 opacity when disabled. The install wait draws a 4px
  radius-999 track that `--border` fills, with a 32 % accent bar travelling it for
  1.6s, and its failure box is a 1px `--danger` radius-10px card.
- **Focus**: `:focus-visible { outline: 2px solid var(--accent); outline-offset:
  2px }` globally; inside rails, subtabs and folder tabs a 1px accent outline at
  offset −1/−2px; segmented radios take a 1px `--text` inset outline; session-mode
  options and buttons keep the global 2px ring.
- **Scrollbars**: 10px, transparent track, thumb in `--border` at radius 999 with
  a 2px transparent border, hover `--faint`; arrow buttons removed; webkit-only
  rules, so `scrollbar-width`/`scrollbar-color` are never set.
- **Accessibility**: color never carries meaning alone (numbers, words or icons
  repeat it); `[hidden]` is `display: none !important`; `.vh` is the
  screen-reader-only class; the skip link sits off-screen at `--accent`/`--bg`
  (radius 6px, `padding: 6px 10px`) and lands at `left: 8px` on focus;
  `aria-disabled` (not `disabled`) is used where a reason line must stay
  reachable; every motion rule has a `prefers-reduced-motion` counterpart.
