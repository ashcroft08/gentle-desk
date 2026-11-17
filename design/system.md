# gentle-desk design system

Synced 2026-11-16 from `apps/desktop/src/theme.css` (every color, shadow and row
token, all six theme blocks) and `apps/desktop/src/app.css` (layout token,
typography, geometry, motion, component rules). Values are recorded exactly as
the sources declare them; nothing invented.

Sources read, and never edited: `apps/desktop/src/theme.css`,
`apps/desktop/src/app.css`. One component rule is cited to the file that owns it
(`apps/desktop/src/helpers/HelperThread.svelte`).

Theme selection is one `data-theme` value on `<html>`; `system` is only the
default of a run that has not chosen one (`theme.css`).

## Colors — theme `dark` (the default: `:root` and `[data-theme='dark']`)

Source: `apps/desktop/src/theme.css`.

| Token | Value | Usage |
| --- | --- | --- |
| `--bg` | `#0e1116` | window and main-area base; text on a filled accent control |
| `--surface` | `#151a21` | sidebar, composer box, menus, popovers, dialog cards |
| `--surface2` | `#1c222b` | hover fill of rows/buttons/icons, notice and card bodies, inline code and code blocks |
| `--border` | `#2b333d` | 1px hairlines, inset hover rings, dashed separators, scrollbar thumb |
| `--text` | `#e4e8ee` | body copy and every primary label |
| `--muted` | `#9aa4b2` | secondary labels, hints, quiet status lines, icons at rest |
| `--faint` | `#7d8593` | tertiary text: placeholder, key hints, uppercase section labels, disabled tint |
| `--accent` | `#7aa2f7` | focus ring, selected text, links, filled primary action, focus borders |
| `--tint` | `rgba(122, 162, 247, 0.14)` | selected row/tab/menu-item band, user bubble, gate card background |
| `--success` | `#7edc9a` | ok tag and success tag text |
| `--warning` | `#e7b455` | warning notice edge, warning tag, reached limit, ≥75 % gauge tone |
| `--danger` | `#ff8585` | error edge and title, destructive button, helper stop square, ≥95 % gauge tone |
| `--shadow` | `0 8px 24px rgba(0, 0, 0, 0.45)` | dialog cards, menus, theme/chat popovers |
| `--row-h` | `34px` | fixed height of one sidebar tree row |

## Colors — theme `light` (`[data-theme='light']`)

Source: `apps/desktop/src/theme.css`. Same roles as the dark block.

| Token | Value | Usage |
| --- | --- | --- |
| `--bg` | `#f2f4f7` | window base |
| `--surface` | `#ffffff` | cards, menus, composer, sidebar |
| `--surface2` | `#e9edf2` | hover fills, notice bodies, inline code |
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
| `--surface2` | `#e9edf2` | hover fills, notice bodies, inline code |
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
| `--chat-max` | `880px` | max width of `.thread-in` and `.composer-dock`, so the box lines up with the thread; ~75 characters at the base font | `apps/desktop/src/app.css` (`:root`) |

## Component-scoped properties

Both are custom properties that app code sets on an element; neither has a fixed
value in CSS.

| Token | Value | Usage | Source |
| --- | --- | --- | --- |
| `--swatch` | set per item (`var(--swatch, var(--accent))` fallback) | the accent dot beside a theme's name in the picker list, `.tpl .swatch` | `apps/desktop/src/app.css`, set by `apps/desktop/src/sidebar/ThemePicker.svelte` |
| `--d` | set per cell, a delay in ms | stagger of the wordmark's typing animation, `.wm-cell`/`.wm-shadow` | `apps/desktop/src/app.css`, set by `apps/desktop/src/brand/WordMark.svelte` |

## Typography

- Base: `font: 14px/1.5 -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue',
  Arial, sans-serif` on `body` (`app.css`). UI text inherits it; `button, input,
  select, textarea` re-declare `font: inherit` and `color: inherit`.
- Monospace: `ui-monospace, 'Cascadia Code', Consolas, monospace` (`app.css`;
  the welcome card's paths use `'Cascadia Mono'`). Used at 11.5–13px for code,
  paths, model names, versions, the context chip (13px/600 there and in the
  usage gauge).
- Scale in use (`app.css`): 11px (tags, table headers, uppercase labels), 11.5px
  (mode note, usage window label and reset line), 12px (hints, secondary lines,
  footers, pills, key hints — the most used size, 40 declarations), 12.5px (quiet
  pane lines, subtab labels, provider names, dialog path and consequence lines),
  13px (base UI text, menus, rows, notices, buttons, activity line), 14px (body,
  header title under 900px, helper thread name), 15px (dialog titles, pane
  headings, profile name), 16px (markdown h2, welcome h2, whats-new title), 18px
  (markdown h1), 26px (welcome h1).
- Weights: 600 for titles, section labels, filled actions and selected tabs;
  nothing else is heavier than body.
- Section labels are 11–12px uppercase with `letter-spacing: 0.05em` in
  `--faint`, or 12px/600 in `--muted` with `letter-spacing: 0.02em` for sidebar
  group headers and pane headings.
- Line heights outside the base: 1.2 (row sub-line), 1.3 (markdown headings),
  1.4 (menu items), 1.5 (hints and consequence lines), 1 (preview chip, context
  chip, whats-new version pill).

## Spacing

- Rhythm declared across `app.css`: 2 / 3 / 4 / 6 / 7 / 8 / 9 / 10 / 12 / 14 /
  16 / 18 / 20 / 24 / 28 / 32 / 36px.
- Gaps: 8px is the house default (13 declarations — buttons row, row icon slot,
  composer, dialog actions); 10px for notice/fatal rows and the header title
  line; 4px 10px for wrapping metadata rows; 18px between thread messages and
  between the settings rail and pane.
- Card padding: dialogs 18px, settings cards 12px 14px / 12px 16px, whats-new
  header 20px 24px 14px and notes 16px 24px, welcome card 32px 36px.
- Inset text offsets: dock lines (`.why`, `.keys`, `.drop`, `.refused`) take
  `padding-left: 16px` so they align with the composer's text; notices pad
  `8px 8px 8px 12px`.

## Radius

4px (inline code, user-bubble corner), 6px (small chips, menu items in a
popover, tooltips, inline rename fields, skip link), 8px (buttons, rows, inputs,
icon buttons, rail tabs, subtabs, segmented controls), 10px (cards, notices,
menus, popovers, `.new`, thumbnails, welcome found/wrapper choice), 12px
(settings cards, provider list, popover list, welcome choice), 14px (dialog
cards, user bubble), 16px (welcome card), 18px (composer), 999px (pills, tags,
dots, thumbs, bars, send button), 50% (activity dots, theme swatches),
`8px 8px 0 0` (folder tabs).

## Shadows and elevation

- `--shadow` (`0 8px 24px …` per theme) on `.dlg`, `.menu`, `.tpl`, and the
  settings dialog's card.
- Popovers use `box-shadow: 0 8px 24px rgba(0, 0, 0, 0.4)` — a fixed value, not
  the token.
- Dialog scrim `.ovl` `rgba(0, 0, 0, 0.5)` at z-40, the dialog card at z-41;
  the floating sidebar's scrim is `rgba(0, 0, 0, 0.45)` at z-30 with the sidebar
  at z-40 and `box-shadow: 8px 0 24px rgba(0, 0, 0, 0.45)`.
- Inset ring `inset 0 0 0 1px var(--border)` is the house hover outline for rows,
  rail tabs, menu items and icon buttons; the danger/warning/info notice edge is
  `inset 3px 0 0` in its tone.
- Menus at z-15, theme list at z-25, popovers at z-30, tooltips at z-20.

## Motion

- Transitions are 120ms ease and only on hover/selection: rail tabs, subtabs,
  theme list items, row timestamps, settings buttons — written either as
  `120ms ease` or as explicit `background-color 120ms ease, color 120ms ease`.
- Keyframes: `wm-type` / `wm-shadow` 460ms ease-out (wordmark typing),
  `pane-in` 120ms (settings pane swap), `pulse` 1.2s ease-in-out infinite (the
  Working mark, 50% opacity 0.35), `activity-pulse` 1.2s staggered 0.2/0.4s
  (three dots, 0.25→1 opacity), `install-progress-travel` 1.6s ease-in-out
  infinite (npm wait bar).
- `@media (prefers-reduced-motion: reduce)` kills every transition and animation;
  the Working mark keeps its still glyph, the install bar becomes a still full
  track at opacity 0.6.

## Component rules

- **Shell** — `.app` is a grid of `280px minmax(0, 1fr)`; collapsed becomes one
  column, and the narrow-window overlay floats the sidebar at
  `min(280px, 85vw)`. `.side` is `--surface` with a 1px right border and
  `padding: 12px 10px 8px`; `.side-toggle` is a 30px square, radius 8px, hover
  `--surface2`. `.foot` is bottom-anchored (`margin-top: auto`), 6px gaps, top
  border, and its icon buttons suppress the hover ring. `.update` row: `--surface2`,
  bottom border, 13px, `padding: 8px 16px`; its bar is 160×6 and its two buttons
  are radius 8px `padding: 3px 10px` at 12px (`.update-go` filled accent,
  `.update-quiet` transparent with `--border`).
- **Chat header** (`.mh`): 52px tall, `padding: 0 20px`, bottom border, h1 15px/600
  with ellipsis; at ≤900px it tightens to 48px / `0 12px` and the title drops to
  14px. Inline rename (`.mh .rn`) is 26px high, `--bg` fill, 1px accent, radius
  6px.
- **Thread** — `.thread` `padding: 28px 24px 12px`, scrolls; `.thread-in` is a
  column at `max-width: var(--chat-max)` with 18px gaps. A user message `.msg.u`
  is `--tint`, `border-radius: 14px 14px 4px 14px`, `padding: 8px 14px`,
  `max-width: 78%`, right-aligned; an assistant message `.msg.a` is
  `padding: 0 4px` of plain text.
- **Markdown** (`.md`): blocks `margin: 0 0 12px`; h1/h2/h3-h6 are 18/16/14px with
  `margin: 16px 0 8px` and line-height 1.3; inline `code` is `--surface2`, radius
  4px, `padding: 1px 5px`, 13px; `pre` is `--surface2` with a 1px `--border`,
  radius 10px, `padding: 10px 12px`; `blockquote` takes a 2px `--border` left edge,
  `padding-left: 12px`, `--muted`; links are `--accent`; table cells are 1px
  `--border` with `padding: 4px 8px`. `.copy` under a reply is 12px `--muted`,
  radius 6px, hover `--surface2` + `--text`.
- **Model error** (`.err`): `--surface2` fill, 1px `--danger` border plus a 3px
  inset danger left edge, radius 10px, `padding: 10px 12px`; its title `.et` is
  600 in `--danger`. The fatal launcher bar (`.fatal`) reuses that language
  across the window top.
- **Placeholder / empty states**: `.placeholder` is `--muted` on
  `padding: 28px 24px`; `.empty` is `padding: 18px 10px` with a 13px/600 `.et`
  and a 12px `--muted` `.notice-line`.
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
  6px below the button. The composer's send/stop circle is 36px at radius 999;
  `.ic.fill` is accent-on-`--bg` and brightens with `filter: brightness(1.1)` on
  hover.
- **Composer**: `.composer-dock` is `width: min(var(--chat-max), calc(100% - 48px))`
  centred with `margin: 0 auto 20px`; `.composer` is a flex row with 8px gaps,
  1px `--border`, radius 18px, `--surface`, `padding: 10px 10px 10px 16px`, and a
  2px accent outline on `:focus-within`. Its textarea is transparent, 24px
  minimum to 120px maximum height, `line-height: 1.5`, `--faint` placeholder; a
  drag over it (`.over`) turns the border accent. Under it: `.why` (12px
  `--muted`), `.keys` (12px `--faint`), `.drop` (12px accent), `.refused` (12px
  `--danger`), and `.dock-bottom` (flex wrap, space-between, 8px top margin)
  holding the keys and the chip row.
- **Model chip**: `.pill` is border-less, radius 10px, `padding: 3px 6px`, 12px
  `--faint`, hover `--surface2`; the value is 600 when the pill is not `.faint`.
  `.popover` opens upward (`bottom: calc(100% + 4px)`), `--surface`, 1px border,
  radius 12px, `box-shadow: 0 8px 24px rgba(0,0,0,0.4)`, `padding: 4px`,
  `max-width: 320px`, `max-height: 300px`; its `.filter` is radius 8px with an
  accent border on focus; `.option` is 12px, radius 8px, `padding: 6px 8px`, with
  `.active` on `--surface2` and `[aria-selected='true']` in `--accent`; a `.hint`
  inside is 11px `--faint`.
- **Context chip** (`.ctx`): a 999px pill, 1px `--border`, `padding: 4px 10px`,
  monospace 12px/600, holding a 46×5 radius-3px track (background `--border`)
  whose fill is `--accent` with `min-width: 4px`. `.warning` from 75 % and
  `.danger` from 95 % recolour the text, border and fill; `.unknown` is `--faint`
  and carries no bar fill.
- **Attachments**: `.thumb` is 72×72 with `object-fit: contain`, radius 10px, 1px
  `--border`, `--surface` fill; `.take` removes one at 16px radius 999 over
  `color-mix(in srgb, var(--bg) 78%, transparent)`, turning `--danger` on hover.
- **Sidebar**: wordmark cells fill `--accent` and animate in with
  `wm-type 460ms ease-out var(--d) backwards`, each over a shadow cell of
  `color-mix(in srgb, var(--accent) 24%, transparent)`. `.new` is the 38px
  full-width New session button: 1px `--border`, radius 10px, `--surface2`,
  weight 600, hover accent border, `aria-disabled` opacity 0.55 with the border
  held. `.wh` pane header `margin: 14px 0 6px`, `padding-left: 10px`, h2 12px/600
  `--muted` `letter-spacing: 0.02em`; `.gh` group heading `padding: 14px 10px 4px`
  in the same style.
- **Sidebar rows** (`.row`): height `var(--row-h)` (34px), radius 8px,
  `padding: 0 10px`, 13px; hover `--surface2` + inset 1px ring; selected
  `background: --tint`; folder rows weight 600, chat rows `padding-left: 42px`
  and a real 44px right slot for the actions icon; `.sub` is 12px `--muted` at
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
  sits in `--danger` and whose hover *is* the fill (`background: --danger`,
  glyph `--bg`, no ring); while its helper is not running it goes `--faint` and
  keeps its place, with no hover fill.
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
  `.acts` is right-aligned flex with 8px gaps and `margin-top: 16px`. A dialog's
  consequence line `.why` is 13px at line-height 1.55 with only its scope in
  `--text`/600; `--danger` when the line reports a failure. `.path` is monospace
  12.5px `--muted`; a `details` disclosure is a plain list of 12.5px `--muted`
  rows with dashed separators and an accent summary line that underlines on
  hover.
- **Whats-new modal** (`.dlg.whatsnew`): `padding: 0`, header `20px 24px 14px`
  over a bottom border, title 16px, a version pill (999px, `--tint`, accent,
  monospace 600 12px) and a 12px `--faint` "since" line pushed right. Its language
  tabs are folder tabs: radius `8px 8px 0 0`, `padding: 5px 13px`, 12.5px
  `--muted`, and the selected tab takes `--surface` with `--accent` text and
  weight 600 while sitting 1px lower to cover the separator. Notes paragraphs are
  `--muted` under 11px/600 uppercase `--faint` headings
  (`letter-spacing: 0.06em`), each list item marked by a 6px accent dot; actions
  sit in a `14px 24px 20px` footer over a top border.
- **Settings dialog**: `.dlg.settings` is a 680px card with 14px gaps;
  `.settings-body` puts a fixed 184px rail beside the pane (`gap: 18px`,
  `min-height: 240px`); the rail keeps a right border and 2px gaps. Rail tabs are
  32px, `padding: 0 10px`, radius 8px, 13px, `--muted` with 9px glyph gaps; hover
  `--surface2` + `--text`; selected `--tint` + `--accent`; focus is a 1px inset
  accent ring. `.settings-pane` is a column with a 120ms `pane-in` fade; its `h3`
  is 15px/600, `.hint` 12px `--muted`, `.quiet` 12.5px `--muted`, `.note` 12px
  `--faint` italic, and its one action row (`.acts`) is left-aligned with 8px
  gaps. `.btn.accent` is the filled accent action (`--bg` text, 600, hover
  `brightness(1.08)`).
- **Tags** (`.tag`): 11px/600 pill, `padding: 3px 8px`, radius 999,
  `1px solid currentColor`; tone from `.ok` `--success`, `.warn` `--warning`,
  `.quiet` `--muted`, `.bad` `--danger`.
- **Settings rows and cards**: `.mode` is the orchestrator strip (bottom border,
  12px bottom padding, 14px bottom margin) with a 12px/600 `--muted` label and an
  11.5px `--faint` note. Segmented radios (`.seg`) share one 1px border and radius
  8px: options are `padding: 5px 12px` at 12.5px `--muted`, hover `--surface2`,
  checked fills `--accent` with `--bg` text and 600. `.profile`, `.providers`,
  `.card.updates` and `.card.usage-provider` are `--surface2` cards with a 1px
  `--border` and radius 12px (`padding: 12px 14px` / `12px 16px`); the profile's
  `.pname` is 15px/600 `--accent` over a 12px `--faint` `.pmark`, and its routes
  are `dt` labels at 11px/600 `--faint` uppercase (`letter-spacing: 0.05em`,
  92px wide) with the model in monospace 12px and the effort in accent monospace.
  `.subtabs` is the quiet tab pair: 1px `--border`, radius 10px, `padding: 2px`,
  options `padding: 4px 14px` at 12.5px, selected `--tint` + `--accent` + 600 with
  an inset accent ring.
- **Tables** (`.agents`): 13px, headers 11px/600 uppercase `--faint`
  (`letter-spacing: 0.05em`) over a bottom border, cells `padding: 5px 8px`, the
  model column monospace 12px `--muted` with ellipsis, the effort column right
  aligned at 64px, and rows highlighted on hover or `focus-within`.
- **Usage gauge**: `.ubar` is a 76×5 radius-3px track (`--border`) with a
  `min-width: 4px` fill in `--accent`, `--warning` or `--danger` by share, beside
  a monospace 12px/600 `.upercent` and a 11.5px `--faint` reset line; a reached
  limit turns its name `--warning` as well as carrying its tag. The gate line
  (`.gate`) is a radius-10px `--tint` card in 12px `--muted`.
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
  offset −1/−2px; segmented radios take a 1px `--text` inset outline.
- **Scrollbars**: 10px, transparent track, thumb in `--border` at radius 999 with
  a 2px transparent border, hover `--faint`; arrow buttons removed; webkit-only
  rules, so `scrollbar-width`/`scrollbar-color` are never set.
- **Accessibility**: color never carries meaning alone (numbers, words or icons
  repeat it); `[hidden]` is `display: none !important`; `.vh` is the
  screen-reader-only class; the skip link sits off-screen at `--accent`/`--bg`
  (radius 6px, `padding: 6px 10px`) and lands at `left: 8px` on focus;
  `aria-disabled` (not `disabled`) is used where a reason line must stay
  reachable; every motion rule has a `prefers-reduced-motion` counterpart.
