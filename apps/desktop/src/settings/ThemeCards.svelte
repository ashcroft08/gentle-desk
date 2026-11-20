<script lang="ts">
  // The Appearance section's theme choice (the reference's theme cards,
  // crates/app/src/features/settings/theme.rs:463-533): one card per theme, each
  // drawn as a miniature terminal over THAT theme's own colours - a prompt
  // chevron in its accent, a letter pair in its foreground, an "ok" in its own
  // green and a path in its muted grey over the theme's background, then the
  // palette dots it can spare - with the drawn name under it, the selected card
  // outlined in the accent and checked.
  //
  // Nothing is instantiated to preview a theme. The miniature carries a
  // `data-theme` attribute, and `theme.css` already scopes every token to that
  // attribute, so the preview reads the same variables the window would and this
  // component spells no colour of its own: a theme that gains a token gains a
  // preview, and no second palette can drift from the first. `system` has no
  // block of its own beyond the light/dark pair it follows, so its card shows the
  // window's colours - which is what choosing it means.
  //
  // This replaces the theme picker the Appearance section held (the listbox the
  // footer used to open, retired with these cards): same choice, same intent,
  // drawn as cards. A card is a real button, so Enter and Space activate it and
  // every card is a tab stop; its state is `aria-pressed`, since it is one answer
  // of a small set and not a row of a list the arrows walk.
  import { dispatchIntent } from '../bridge'
  import type { ThemeView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { THEMES, themeLabelsCopy } from '../theme/themes'
  import { settingsCopy } from './copy'

  let { theme }: { theme: ThemeView } = $props()

  /** The six drawn names in the language the app's own words are written in. */
  const labels = $derived(themeLabelsCopy(language.current))
  /** The grid's own accessible name, in the same language (0.2.0, D4). */
  const copy = $derived(settingsCopy(language.current))

  /**
   * The palette dots, in the reference's order - red, yellow, green, then the
   * rest of what the theme has - named as the tokens `theme.css` declares. Only
   * tokens are named here, never a colour: a theme that stops declaring one is a
   * theme whose dot would otherwise fall back to the window's, which the tests
   * pin against the stylesheet.
   */
  const PALETTE: readonly string[] = ['danger', 'warning', 'success', 'accent', 'muted', 'faint', 'text']

  /** One choice, one intent: the theme already chosen is not a change. */
  function choose(next: ThemeView): void {
    if (next === theme) return
    void dispatchIntent({ type: 'SetTheme', data: { theme: next } })
  }
</script>

<div class="tc-grid" role="group" aria-label={copy.themeGroup}>
  {#each THEMES as option (option)}
    <button
      type="button"
      class="tc"
      aria-pressed={option === theme}
      onclick={() => choose(option)}
    >
      <!-- The miniature is decoration: the card's name is its word. -->
      <span class="tc-shot" data-theme={option} aria-hidden="true">
        <span class="tc-line">
          <span class="tc-prompt">❯</span>
          <span class="tc-text">Aa</span>
          <span class="tc-ok">ok</span>
          <span class="tc-path">~/src</span>
        </span>
        <span class="tc-dots">
          {#each PALETTE as token (token)}
            <span class="tc-dot" style:--dot={`var(--${token})`}></span>
          {/each}
        </span>
      </span>
      <span class="tc-meta">
        <span class="tc-name">{labels[option]}</span>
        {#if option === theme}
          <!-- The mark of the chosen card, beside its name: Lucide's check, the
               one shape this control draws that the shared set does not hold. -->
          <svg class="tc-check" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
            <path d="M20 6 9 17l-5-5" />
          </svg>
        {/if}
      </span>
    </button>
  {/each}
</div>
