# gentle-desk — design system (sourced from the app's real CSS)

Synced 2026-11-13 from the two authoritative sources. Every value below is copied from
them, never proposed: `apps/desktop/src/theme.css` (six themes, one `[data-theme]` block
each, on `<html>`) and `apps/desktop/src/app.css` (component rules). If a value is not in
those files, this document says so explicitly.

## Themes (theme.css:8-141)

Six picker entries; `system` resolves via `prefers-color-scheme` (`theme.css:96-113`):
dark table below, or the `light` table. Blocks: `:root,[data-theme='dark']`,
`[data-theme='light']`, `[data-theme='Gentle']`, `[data-theme='Gentleman-Cute']`,
`[data-theme='Gentleman-Sexy']`.

| Token | dark | light | Gentle | Cute | Sexy | usage (app.css) |
|---|---|---|---|---|---|---|
| `--bg` | `#0e1116` | `#f2f4f7` | `#06080f` | `#060407` | `#060407` | page, inputs' well, code well |
| `--surface` | `#151a21` | `#ffffff` | `#06080f` | `#100a0f` | `#100a0f` | dialogs, cards |
| `--surface2` | `#1c222b` | `#e9edf2` | `#0d0f14` | `#100a0f` | `#100a0f` | `.btn` bg, secondary rows |
| `--border` | `#2b333d` | `#cfd5dd` | `#313342` | `#563040` | `#563040` | separators, button borders |
| `--text` | `#e4e8ee` | `#17202a` | `#f3f6f9` | `#f6eff3` | `#f6eff3` | body text, titles, strong |
| `--muted` | `#9aa4b2` | `#576272` | `#8394a3` | `#a78e9b` | `#a78e9b` | secondary text, labels |
| `--faint` | `#7d8593` | `#6b7686` | `#96a2b0` | `#d7a0b8` | `#d7a0b8` | hints, timestamps, Esc hint |
| `--accent` | `#7aa2f7` | `#2e5bd7` | `#7fb4ca` | `#f095c8` | `#f43888` | running mark, primary border/bg, focus |
| `--tint` | `rgba(122,162,247,.14)` | `rgba(46,91,215,.1)` | `#161d2e` | `#2a1523` | `#2e0f1f` | selected rows, version pill |
| `--success` | `#7edc9a` | `#166e38` | `#b7cc85` | `#b4e7c7` | `#d2cbd0` | ok badges, finished-unread |
| `--warning` | `#e7b455` | `#8a5800` | `#deba87` | `#f2b86d` | `#f2b86d` | needs-attention, warnings |
| `--danger` | `#ff8585` | `#c22f37` | `#cb7c94` | `#ff718f` | `#ff718f` | errors, delete items |
| `--shadow` | `0 8px 24px rgba(0,0,0,.45)` | `…rgba(23,32,42,.18)` | `…rgba(0,0,0,.6)` | `…rgba(0,0,0,.6)` | `…rgba(0,0,0,.6)` | dialog card lift |
| `--row-h` | `34px` | `34px` | `34px` | `34px` | `34px` | list rows |

Gentleman tints are real fills, not alphas (theme.css:60-93). There are NO
`--surface-2`, `--accent-tint`, `--success-tint`, `--danger-tint`, `--warning-tint`
tokens in the sources — rules named that way in earlier design docs are stale.

## Typography (app.css)

- Stack (app.css `.dlg code`, monospace contexts): `ui-monospace, 'Cascadia Code', Consolas, monospace`; body uses the Tauri default (system sans).
- Base body `13px` (app.css:32); secondary `12px`; dialog title `15-16px`; section heading in modals `14-18px`; oversized only in the sidebar wordmark.
- Weights: 400 body, 600 for titles/labels/strong, 600 monospace pills.

## Scale (app.css)

- Radius: `8px` (buttons, inputs, code wells — most common), `10px`, `6px` (small chips),
  `12px` (cards), `14px` (dialog card), `999px` (pills/badges), `50%` (dots).
- Spacing rhythm: 4/6/8/14/16/18/24px; dialog padding `18px`, section padding `14-24px`.
- Row height: `--row-h` 34px for list rows and buttons.
- Focus: `outline: 2px solid var(--text); outline-offset: 2px` (artboard precedent) or accent border on inputs.

## Component rules (app.css selectors)

- `.btn` — border `1px solid var(--border)`, bg `var(--surface2)`, color `var(--text)`,
  radius `8px`, padding `6px 14px`; hover border `var(--accent)`; disabled opacity .55.
  Primary: `border-color: var(--accent)`; modal primary (whats-new): bg `var(--accent)`,
  color `var(--bg)`, weight 600. Danger `.btn.d`: bg/border `var(--danger)`, color `var(--bg)`.
- `.dlg` — fixed card, centered, `min(440px, calc(100vw - 32px))` (settings: 680px two-pane
  rail; whats-new: 560px), bg `var(--surface)`, border, radius `14px`, padding `18px`,
  shadow `var(--shadow)`, `z-index: 41` over `.ovl rgba(0,0,0,.5)`.
- `.dlg .dt` — dialog title, `15px/600`, margin `0 0 6px`.
- `.dlg label` — `12px/600` `var(--muted)`, margin `14px 0 6px`.
- `.dlg input` — full width, border `var(--accent)`, radius `8px`, padding `7px 10px`, bg `var(--bg)`.
- Rows/badges — `--row-h` rows; pills radius `999px` on `var(--tint)` with `var(--accent)`
  text (version pill precedent, `.dlg.whatsnew .ver`).
- Statuses: color never alone — every status pairs its colour with a glyph and a text label.
- Markdown notes: bullets drawn as accent dots (`li::before`), `strong` in `--text`, `em` faint,
  `hr` dashed `--border` (whats-new precedent).

## Self-check rule for artboards

List the colours and families an artboard uses; every one must be one of the tokens above
(its themed value at render time) or the system font stack. Fixed hex/rgb outside these
tokens is a deviation.
