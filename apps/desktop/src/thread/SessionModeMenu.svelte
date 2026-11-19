<script lang="ts">
  // The session mode popover (design/composer-dock S1, T3c; sheets A and B): the
  // list of the four modes the channel knows one row at a time, and the question
  // that replaces the list for the two that ask first.
  //
  // It is split out of `./SessionModeControl` because a `fixed` popover is its own
  // job: it measures the button it hangs from, holds the focus inside itself, and
  // closes on Escape or a click outside its own bounds. The control owns the mode
  // and the write; this only names them and hands the choice back.
  //
  // It is drawn only while the control's popover is open, so the confirmation
  // step is local state: every opening starts on the list, as it always has.
  import type { SessionModeView_Serialize as SessionModeView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { threadCopy } from './copy'
  import { MODE_CHOICES, modeIsDanger, modeLabel } from './session-mode'

  /** One of the four modes the page may ask for. */
  type Choice = (typeof MODE_CHOICES)[number]

  let {
    sessionMode,
    anchor = null,
    onsend,
    onclose,
  }: {
    /** What the chat's child reported (T3b): the mode the list marks. */
    sessionMode: SessionModeView
    /** The control's button: the popover hangs from it and excludes it from dismissal. */
    anchor?: HTMLButtonElement | null
    /** The reader chose a mode: quiet ones at once, danger ones after its button. */
    onsend: (choice: Choice) => void
    /** Closes the popover, leaving the focus on the control when asked to. */
    onclose: (refocus: boolean) => void
  } = $props()

  /** The line's own word in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))

  /** Whether the popover shows the confirmation step, and for which mode. */
  let confirming = $state<Choice | null>(null)

  /** Where the popover is pinned: it is `fixed`, so the thread never clips it. */
  let spot = $state({ left: 0, top: 0 })

  let pop = $state<HTMLElement | null>(null)
  let cancelButton = $state<HTMLButtonElement | null>(null)

  /** The menu's four rows, in the design's order, with the one hairline. */
  const options = $derived(
    MODE_CHOICES.map((mode) => ({
      mode,
      label: modeLabel(mode),
      danger: modeIsDanger(mode),
      // Only the two widest have a source for a description: Normal and Auto
      // are drawn with their name alone.
      why:
        mode === 'yolo' ? copy.modeYoloWhy : mode === 'super-yolo' ? copy.modeSuperWhy : null,
      // The hairline splits the two quiet modes from the two that ask first.
      sep: mode === 'yolo',
    })),
  )

  // The popover is pinned above or below the control, whichever side has room,
  // and never wider than the window.
  $effect(() => {
    if (pop === null || anchor === null) return
    const rect = anchor.getBoundingClientRect()
    const width = pop.offsetWidth || 292
    const height = pop.offsetHeight
    const gap = 4
    const below = window.innerHeight - rect.bottom
    // Below when it fits, above otherwise; the fallback keeps it on screen when
    // neither side has room.
    const fitsBelow = below >= height + gap || rect.top < height + gap
    spot = {
      left: Math.max(8, Math.min(rect.right - width, window.innerWidth - width - 8)),
      top: fitsBelow ? rect.bottom + gap : rect.top - height - gap,
    }
  })

  // The focus follows the popover being bound and which step it shows: the mode
  // in effect (or the first one) for the list, Cancel for the question.
  $effect(() => {
    if (confirming !== null) {
      cancelButton?.focus()
      return
    }
    const current = pop?.querySelector<HTMLElement>('[aria-checked="true"]')
    const first = pop?.querySelector<HTMLElement>('[role="menuitemradio"]')
    ;(current ?? first)?.focus()
  })

  // A click outside the control and its popover closes it, the way every
  // popover of this app behaves.
  $effect(() => {
    const onDown = (event: PointerEvent) => {
      const target = event.target
      if (!(target instanceof Node)) return
      if (pop?.contains(target) === true || anchor?.contains(target) === true) return
      onclose(false)
    }
    document.addEventListener('pointerdown', onDown)
    return () => document.removeEventListener('pointerdown', onDown)
  })

  function choose(choice: Choice): void {
    if (modeIsDanger(choice)) {
      confirming = choice
      return
    }
    onsend(choice)
  }

  function confirm(): void {
    const choice = confirming
    if (choice === null) return
    onsend(choice)
  }

  function cancel(): void {
    confirming = null
  }

  function popoverKeyed(event: KeyboardEvent): void {
    if (event.key !== 'Escape') return
    event.preventDefault()
    if (confirming !== null) {
      cancel()
      return
    }
    onclose(true)
  }
</script>

{#if confirming === null}
  <div
    class="mode-pop"
    role="menu"
    aria-label={copy.modeMenu}
    tabindex="-1"
    style:left="{spot.left}px"
    style:top="{spot.top}px"
    bind:this={pop}
    onkeydown={popoverKeyed}
  >
    <p class="cap">{copy.modeCaption}</p>
    {#each options as option (option.mode)}
      {#if option.sep}
        <div class="mode-sep" role="separator"></div>
      {/if}
      <button
        type="button"
        role="menuitemradio"
        aria-checked={option.mode === sessionMode.mode}
        class="mode-opt"
        class:hot={option.danger}
        onclick={() => choose(option.mode)}
      >
        <span class="ck" aria-hidden="true"
          >{option.mode === sessionMode.mode ? '\u2713' : ''}</span
        >
        <span class="nm">
          {option.label}
          {#if option.danger}<span class="warn">{copy.modeConfirmTag}</span>{/if}
        </span>
        {#if option.why !== null}
          <span class="ds">{option.why}</span>
        {/if}
      </button>
    {/each}
  </div>
{:else}
  <!-- The confirmation replaces the list inside the same popover (sheet B):
       Cancel holds the first focus, Esc cancels, and only the danger button
       sends. -->
  <div
    class="mode-pop"
    role="dialog"
    aria-label={copy.modeConfirm(modeLabel(confirming))}
    tabindex="-1"
    style:left="{spot.left}px"
    style:top="{spot.top}px"
    bind:this={pop}
    onkeydown={popoverKeyed}
  >
    <div class="mode-conf">
      <p class="q">{copy.modeAsk(modeLabel(confirming))}</p>
      <p class="why">
        <strong>{copy.modeEffect}</strong>{confirming === 'yolo'
          ? copy.modeEffectYolo
          : copy.modeEffectSuper}
      </p>
      <div class="acts">
        <button type="button" class="mode-btn" bind:this={cancelButton} onclick={cancel}>
          {copy.cancel}
        </button>
        <button type="button" class="mode-btn go" onclick={confirm}>
          {copy.modeGo(modeLabel(confirming))}
        </button>
      </div>
    </div>
  </div>
{/if}
