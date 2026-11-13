<script lang="ts">
  // One pill dropdown of the model chip (S7): a button that opens a themed
  // popover listbox the app's own CSS draws — never a native select popup.
  // The list may be filtered by typing; arrows move, Enter chooses, Escape
  // closes, a click outside closes. The choice leaves through `onchange`, the
  // same intent the old select dispatched.
  import { untrack } from 'svelte'
  import Icon from '../icons/Icon.svelte'
  import { NOTHING_MATCHES, SEARCH_PLACEHOLDER } from './copy'

  export type PillOption = { value: string; label: string; hint?: string }

  let {
    label,
    value = null,
    options,
    filterable = false,
    faint = false,
    onchange,
  }: {
    /** The accessible name of the pill and of its listbox. */
    label: string
    /** The chosen option's value, or `null` when the child said nothing yet. */
    value?: string | null
    options: PillOption[]
    /** Whether the popover shows the filter box (the model list is long). */
    filterable?: boolean
    /** The level pill reads fainter than the model pill. */
    faint?: boolean
    onchange: (value: string) => void
  } = $props()

  let open = $state(false)
  let filter = $state('')
  let active = $state(0)
  let root = $state<HTMLDivElement | null>(null)
  let button = $state<HTMLButtonElement | null>(null)
  let filterBox = $state<HTMLInputElement | null>(null)
  let listbox = $state<HTMLDivElement | null>(null)

  const selected = $derived(options.find((option) => option.value === value) ?? null)
  const visible = $derived.by(() => {
    const needle = filter.trim().toLowerCase()
    if (needle === '') return options
    return options.filter((option) => option.label.toLowerCase().includes(needle))
  })

  // Opening resets the filter and puts the cursor on the chosen option, so a
  // second open does not inherit the last search. Only the open flag may re-run
  // it (review R3-1): `options` and `value` are read untracked, because the chip
  // rebuilds the options array on every state push and a chat that is answering
  // pushes constantly, so a tracked read wiped the filter while the user typed.
  $effect(() => {
    if (!open) return
    untrack(() => {
      filter = ''
      const chosen = options.findIndex((option) => option.value === value)
      active = chosen < 0 ? 0 : chosen
    })
  })

  // The focus follows the popover's elements being bound, never the options.
  $effect(() => {
    if (!open) return
    if (filterable) {
      filterBox?.focus()
    } else {
      listbox?.focus()
    }
  })

  // A click outside the pill closes it, the way every popover behaves.
  $effect(() => {
    if (!open) return
    const onDown = (event: PointerEvent) => {
      if (root !== null && event.target instanceof Node && !root.contains(event.target)) {
        open = false
      }
    }
    document.addEventListener('pointerdown', onDown)
    return () => document.removeEventListener('pointerdown', onDown)
  })

  function close(): void {
    open = false
    button?.focus()
  }

  function choose(option: PillOption): void {
    onchange(option.value)
    close()
  }

  function buttonKeyed(event: KeyboardEvent): void {
    if (open) return
    if (event.key === 'Enter' || event.key === ' ' || event.key === 'ArrowDown') {
      event.preventDefault()
      open = true
    }
  }

  function popoverKeyed(event: KeyboardEvent): void {
    if (event.key === 'Escape') {
      event.preventDefault()
      close()
      return
    }
    if (event.key === 'ArrowDown') {
      event.preventDefault()
      active = Math.min(active + 1, visible.length - 1)
      return
    }
    if (event.key === 'ArrowUp') {
      event.preventDefault()
      active = Math.max(active - 1, 0)
      return
    }
    if (event.key === 'Enter') {
      event.preventDefault()
      const choice = visible[active]
      if (choice !== undefined) choose(choice)
    }
  }
</script>

<div class="pill-select" class:faint bind:this={root}>
  <button
    type="button"
    class="pill"
    bind:this={button}
    aria-haspopup="listbox"
    aria-expanded={open}
    aria-label={label}
    onclick={() => (open = !open)}
    onkeydown={buttonKeyed}
  >
    <span class="pill-value">{selected?.label ?? label}</span>
    <Icon name="chevronDown" size={12} />
  </button>
  {#if open}
    <div
      class="popover"
      role="listbox"
      aria-label={label}
      tabindex="-1"
      bind:this={listbox}
      onkeydown={popoverKeyed}
    >
      {#if filterable}
        <input
          class="filter"
          type="text"
          placeholder={SEARCH_PLACEHOLDER}
          bind:this={filterBox}
          bind:value={filter}
          oninput={() => (active = 0)}
        />
      {/if}
      <ul class="options">
        {#each visible as option, index (option.value)}
          <li>
            <button
              type="button"
              role="option"
              class="option"
              class:active={index === active}
              aria-selected={option.value === value}
              onmouseenter={() => (active = index)}
              onclick={() => choose(option)}
            >
              <span class="option-label">{option.label}</span>
              {#if option.hint !== undefined}
                <span class="hint">{option.hint}</span>
              {/if}
            </button>
          </li>
        {/each}
        {#if visible.length === 0}
          <li class="none">{NOTHING_MATCHES}</li>
        {/if}
      </ul>
    </div>
  {/if}
</div>
