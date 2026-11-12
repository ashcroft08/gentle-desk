<script lang="ts">
  // The footer theme control (01a: `#theme-btn`, its `#theme-list` listbox and
  // the `.tp` panel above it). The list is a real list of THIS page, drawn with
  // the chosen theme's own colors, which is why 01a has no select: the native
  // popup ignored the page theme.
  //
  // The keyboard is the artboard's: arrows, Home and End move the focus, Enter
  // and Space choose, Escape closes back to the button, and the focus stays
  // inside the open list. The current theme is the button's NAME, because the
  // button only draws an icon and silence is not a value. That name, and each
  // option's text, is the theme's LABEL: the identifiers are lowercase in three
  // cases (`system`, `dark`, `light`) and stay that way in the config, the
  // intent and the `data-theme` attribute, so the drawn word is a display name
  // kept beside `THEMES` and never a renamed theme.
  import { tick } from 'svelte'
  import type { ThemeView } from '../contract/bindings'
  import { dispatchIntent } from '../bridge'
  import Icon from '../icons/Icon.svelte'
  import { THEMES, themeLabel, themeSwatch } from '../theme/themes'
  import { movedIndex, trappedIndex } from './theme-keys'

  let { theme }: { theme: ThemeView } = $props()

  /** Whether the list is open. */
  let open = $state(false)
  /** The button, so a closing list can give the focus back to it. */
  let button = $state<HTMLButtonElement | null>(null)
  /** The list itself, so the focus code reads the options it drew. */
  let list = $state<HTMLUListElement | null>(null)

  const name = $derived(`Theme: ${themeLabel(theme)}`)
  const selected = $derived(Math.max(THEMES.indexOf(theme), 0))

  /** The drawn options, in the order `THEMES` names them. */
  function options(): HTMLLIElement[] {
    return list === null ? [] : [...list.querySelectorAll('li')]
  }

  function focusOption(index: number): void {
    const drawn = options()
    drawn[Math.min(Math.max(index, 0), drawn.length - 1)]?.focus()
  }

  function openedIndex(): number {
    return options().indexOf(document.activeElement as HTMLLIElement)
  }

  async function openList(): Promise<void> {
    open = true
    // The options exist only after the update, so the focus moves after it.
    await tick()
    focusOption(selected)
  }

  function closeList(restoreFocus: boolean): void {
    open = false
    if (restoreFocus) button?.focus()
  }

  function toggle(): void {
    if (open) closeList(true)
    else void openList()
  }

  /** One choice, one intent: the same theme again changes nothing in Rust. */
  function choose(next: ThemeView): void {
    closeList(true)
    void dispatchIntent({ type: 'SetTheme', data: { theme: next } })
  }

  function onButtonKeydown(event: KeyboardEvent): void {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault()
      void openList()
    }
  }

  /** A click on an option, taken from the list so no option owns a handler. */
  function onListClick(event: MouseEvent): void {
    const option = (event.target as Element).closest('li')
    const next = option?.getAttribute('data-value') as ThemeView | undefined
    if (next !== undefined) choose(next)
  }

  function onListKeydown(event: KeyboardEvent): void {
    const index = openedIndex()
    const count = options().length
    if (event.key === 'Tab') {
      // Trapped: with the list open the focus moves through its options and
      // never reaches the page behind it.
      event.preventDefault()
      focusOption(trappedIndex(event.shiftKey, index, count) ?? 0)
      return
    }
    const moved = movedIndex(event.key, index, count)
    if (moved !== null) {
      event.preventDefault()
      focusOption(moved)
      return
    }
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      if (index >= 0) choose(THEMES[index])
      return
    }
    if (event.key === 'Escape') {
      event.preventDefault()
      closeList(true)
    }
  }

  // A click anywhere else closes the list, as the artboard's script does, and
  // leaves the focus where the user put it.
  $effect(() => {
    if (!open) return
    const onDocumentClick = (event: MouseEvent): void => {
      const target = event.target as Element | null
      if (target === null || button?.contains(target) || target.closest('.tpl') !== null) return
      closeList(false)
    }
    document.addEventListener('click', onDocumentClick)
    return () => document.removeEventListener('click', onDocumentClick)
  })
</script>

<div class="tp">
  <button
    class="ic tip-l"
    type="button"
    id="theme-btn"
    data-tip="Theme"
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label={name}
    bind:this={button}
    onclick={toggle}
    onkeydown={onButtonKeydown}
  >
    <Icon name="theme" />
  </button>
  {#if open}
    <ul
      class="tpl"
      id="theme-list"
      role="listbox"
      aria-label="Theme"
      tabindex="-1"
      bind:this={list}
      onclick={onListClick}
      onkeydown={onListKeydown}
    >
      {#each THEMES as option (option)}
        <li role="option" tabindex="-1" aria-selected={option === theme} data-value={option}>
          <!-- The theme's accent as a dot: decoration, so the option's name is
               still only its word. `system` draws the two halves it follows. -->
          <span class="swatch" style:--swatch={themeSwatch(option)} aria-hidden="true"></span>
          {themeLabel(option)}
        </li>
      {/each}
    </ul>
  {/if}
</div>
