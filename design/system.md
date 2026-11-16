# gentle-desk design system

Synced 2026-11-16 from `apps/desktop/src/theme.css` (all color tokens, six themes)
and `apps/desktop/src/app.css` (typography, geometry, component rules). Values are
recorded exactly as the sources declare them; nothing invented.

## Colors — theme `dark` (default, `:root` + `[data-theme='dark']`)

| Token | Value |
| --- | --- |
| `--bg` | `#0e1116` |
| `--surface` | `#151a21` |
| `--surface2` | `#1c222b` |
| `--border` | `#2b333d` |
| `--text` | `#e4e8ee` |
| `--muted` | `#9aa4b2` |
| `--faint` | `#7d8593` |
| `--accent` | `#7aa2f7` |
| `--tint` | `rgba(122, 162, 247, 0.14)` |
| `--success` | `#7edc9a` |
| `--warning` | `#e7b455` |
| `--danger` | `#ff8585` |
| `--shadow` | `0 8px 24px rgba(0, 0, 0, 0.45)` |
| `--row-h` | `34px` |

## Colors — theme `light` (+ `[data-theme='system']` under `prefers-color-scheme: light`)

| Token | Value |
| --- | --- |
| `--bg` | `#f2f4f7` |
| `--surface` | `#ffffff` |
| `--surface2` | `#e9edf2` |
| `--border` | `#cfd5dd` |
| `--text` | `#17202a` |
| `--muted` | `#576272` |
| `--faint` | `#6b7686` |
| `--accent` | `#2e5bd7` |
| `--tint` | `rgba(46, 91, 215, 0.1)` |
| `--success` | `#166e38` |
| `--warning` | `#8a5800` |
| `--danger` | `#c22f37` |
| `--shadow` | `0 8px 24px rgba(23, 32, 42, 0.18)` |
| `--row-h` | `34px` |

## Colors — theme `Gentle`

| Token | Value |
| --- | --- |
| `--bg` | `#06080f` |
| `--surface` | `#06080f` |
| `--surface2` | `#0d0f14` |
| `--border` | `#313342` |
| `--text` | `#f3f6f9` |
| `--muted` | `#8394a3` |
| `--faint` | `#96a2b0` |
| `--accent` | `#7fb4ca` |
| `--tint` | `#161d2e` |
| `--success` | `#b7cc85` |
| `--warning` | `#deba87` |
| `--danger` | `#cb7c94` |
| `--shadow` | `0 8px 24px rgba(0, 0, 0, 0.6)` |
| `--row-h` | `34px` |

## Colors — themes `Gentleman-Cute` and `Gentleman-Sexy`

Both share bg/surface/border/text/muted/faint; they differ only in accent, tint,
success and shadow as noted.

| Token | `Gentleman-Cute` | `Gentleman-Sexy` |
| --- | --- | --- |
| `--bg` | `#060407` | `#060407` |
| `--surface` | `#100a0f` | `#100a0f` |
| `--surface2` | `#100a0f` | `#100a0f` |
| `--border` | `#563040` | `#563040` |
| `--text` | `#f6eff3` | `#f6eff3` |
| `--muted` | `#a78e9b` | `#a78e9b` |
| `--faint` | `#d7a0b8` | `#d7a0b8` |
| `--accent` | `#f095c8` | `#f43888` |
| `--tint` | `#2a1523` | `#2e0f1f` |
| `--success` | `#b4e7c7` | `#d2cbd0` |
| `--warning` | `#f2b86d` | `#f2b86d` |
| `--danger` | `#ff718f` | `#ff718f` |
| `--shadow` | `0 8px 24px rgba(0, 0, 0, 0.6)` | `0 8px 24px rgba(0, 0, 0, 0.6)` |
| `--row-h` | `34px` | `34px` |

One more root token: `--chat-max: 880px` (`app.css :root`) — the thread's max line
width, ~75 characters at the base font.

## Typography

- Base: `font: 14px/1.5 -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial,
  sans-serif` on `body` (`app.css`). UI text inherits it; `button/input/select/textarea`
  re-declare `font: inherit`.
- Monospace: `ui-monospace, 'Cascadia Code', Consolas, monospace` — used at 12–13px
  for code, paths, model names, versions, the context chip (weight 600).
- Scale in use (`app.css`): 11px (tags, table headers, uppercase labels),
  11.5–12px (hints, secondary lines, footers, pills), 12.5px (quiet pane lines),
  13px (base UI text, menus, rows, notices), 14px (body), 15px (dialog titles,
  pane headings), 16px (h2, modal title), 18px (h1 in markdown), 26px (welcome h1).
- Headers `h1/h2/h3` weight 600; section labels in panes are 11–12px uppercase with
  `letter-spacing: 0.05em` in `--faint`/`--muted`.

## Spacing, radius, elevation

- Spacing rhythm: 4 / 6 / 8 / 10 / 12 / 14 / 16 / 18 / 20 / 24 / 28 / 32 / 36px
  paddings and gaps as declared per component; dialogs pad 18px; cards 12–16px.
- Radii: 4px (inline code), 6px (small chips, menu items, fields), 8px (buttons,
  rows, inputs, icon buttons), 10px (cards, notices, menus, popovers, `.new`),
  12px (panes' cards, `.tpl`), 14px (dialog cards, user bubbles partial), 16px
  (welcome card), 18px (composer), 999px (pills, tags, dots, thumbs, bars).
- Elevation: `--shadow` token for dialog cards, menus and popovers; dialogs also
  get the `.ovl` scrim `rgba(0,0,0,0.5)` at z-40/41; sidebar overlay shadow
  `8px 0 24px rgba(0,0,0,0.45)`; popover shadow `0 8px 24px rgba(0,0,0,0.4)`.
- Focus: `:focus-visible { outline: 2px solid var(--accent); outline-offset: 2px }`
  globally; inside rails/tabs `outline: 1px solid var(--accent); offset -1/-2px`.
- Motion: transitions 120ms ease on hover/selection only; `@media
  (prefers-reduced-motion: reduce)` kills every transition/animation.
- Scrollbars: 10px webkit-only, transparent track, `--border` thumb rounded,
  brightens to `--faint` on hover.

## Component rules

- **Buttons** (`.btn`): `1px solid --border`, `--surface2` fill, `--text`, radius
  8px, padding `6px 14px`; hover `border-color: --accent`; disabled opacity 0.55.
  Variants: `.d` (destructive: `--danger` border+fill, `--bg` text), `.primary`
  (accent border; in `.whatsnew`/settings panes filled accent + `--bg` text,
  weight 600). Icon buttons (`.ic`): 32px square, transparent, `--muted`, radius
  8px, hover `--surface2` + inset border ring; `aria-disabled` = opacity 0.4, no
  hover. Circle send icon 36px radius 999.
- **Dialogs** (`.dlg`): fixed centre, `width: min(440px, calc(100vw - 32px))`
  (settings 680px, whats-new 560px), `--surface`, `1px --border`, radius 14px,
  padding 18px, `--shadow`, over `.ovl` scrim. Title `.dt` 15px/600 margin-bottom
  6px. Actions `.acts` right-aligned flex, gap 8px, margin-top 16px.
  Destructive button uses `.btn.d`.
- **Menus** (`.menu`): `--surface`, `1px --border`, radius 10px, shadow, padding
  6px, min-width 210px (chat menu 190px, max `calc(280px - 24px)`); items full-width
  text-left, radius 6px, padding `7px 12px`, 13px, nowrap; hover/`data-highlighted`
  `--surface2` + inset border; danger item `.d` colors `--danger`.
- **Sidebar rows** (`.row`): height `--row-h` (34px), radius 8px, padding 0 10px,
  13px; hover `--surface2` + inset border ring; selected `--tint`; folder rows
  weight 600, chat rows indent 42px; chat rows keep a real 44px slot for their
  actions icon, whose reveal replaces the timestamp (opacity swap).
- **Notices / errors / gates**: `.notice` card `--surface2`, `1px --border`,
  radius 10px, 13px, with a 3px inset left edge in `--danger`/`--warning`/
  `--accent`; `.err` box same edge + `--danger` border; `.gate` quiet card on
  `--tint`, `--muted` 12px. Tags `.tag`: 11px/600 pill, `1px currentColor` border,
  tone from `--success/--warning/--muted/--danger`.
- **Inputs** (`.dlg input`, `.field`): full width, `1px --accent` border, radius
  8px, padding `7px 10px`, `--bg` fill, 13px; inline rename field 22px high with
  `--accent` border radius 6px.
- **Rails / tabs**: settings rail tabs 32px, 13px, `--muted`; hover `--surface2`;
  selected `--tint` + `--accent`. Subtabs share the same selected look with an
  inset accent ring, weight 600.
- **Accessibility**: color never carries meaning alone (numbers, words or icons
  repeat it); `[hidden]` display none; `.vh` screen-reader-only class; skip link;
  `aria-disabled` (not `disabled`) when a reason line must stay reachable.
