<script lang="ts">
  // The status line (design/composer-dock S1): the last line of the thread,
  // above the composer, that says what the chat is doing. It shares the reasoning
  // row's anatomy instead of growing a third visual language - a thin spinner, the
  // word, the seconds - faint, 11.5px, no card, no glyph soup.
  //
  // The clock is the WEB's: it counts from the moment the chat's mark BECOMES
  // working and stops when the mark leaves, which is why the contract carries no
  // timestamp for it and the thread gains no field. The line disappears with the
  // turn; a mark that is not `Working` (opening, queued, ended) is another line's
  // business (`./Activity`).
  //
  // composer-dock T3c (L7 revokes R1, L8 approves the design) puts the SESSION
  // mode at the right edge of that line: a quiet text button with a caret that
  // opens the four modes the channel knows (`02-mode-picker.html`, sheet A).
  // Normal and Auto change at once; YOLO and Super YOLO replace the list with a
  // confirmation in the same popover (sheet B), and only its danger button sends.
  //
  // The shown mode NEVER changes optimistically: a write keeps the previous text
  // beside a spinner until the pushed `attempt` moves, which is the shell's own
  // count of answered writes, and a wait nothing answers drops silently after
  // `PENDING_MS`. A refused answer says its code, in the line under this one, and
  // leaves the mode exactly as it was (sheet C). A chat the shell answered
  // nothing for draws NO control: no button, no gap, no dash.
  import type {
    ChatActivityMark,
    SessionModeView_Serialize as SessionModeView,
  } from '../contract/bindings'
  import { dispatchIntent } from '../bridge'
  import Icon from '../icons/Icon.svelte'
  import { language } from '../i18n/language.svelte'
  import { threadCopy } from './copy'
  import { MODE_CHOICES, modeIsDanger, modeLabel } from './session-mode'

  let {
    mark = null,
    chatId = null,
    sessionMode = null,
  }: {
    mark?: ChatActivityMark | null
    /** The foreground chat the write names, or `null` while none is open. */
    chatId?: string | null
    /** What that chat's child reported (T3b), or `null` when nothing answered. */
    sessionMode?: SessionModeView | null
  } = $props()

  /** How long a sent change waits for an answer before the spinner drops. */
  const PENDING_MS = 8000

  /** The line's own word in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))

  /** The mode in effect, as the row and the menu name it; empty with no mode. */
  const label = $derived(sessionMode === null ? '' : modeLabel(sessionMode.mode))

  /** How long the chat has been working, in whole seconds. */
  let seconds = $state(0)

  /** Whether the popover is open, and whether it shows the confirmation step. */
  let open = $state(false)
  let confirming = $state<(typeof MODE_CHOICES)[number] | null>(null)

  /** A change was sent and no answer has moved the attempt yet. */
  let pending = $state(false)
  /** The attempt the pending write started from; a different one ends the wait. */
  let pendingAttempt: number | null = null
  let pendingTimer: ReturnType<typeof setTimeout> | null = null

  /** The last refusal the pushed state brought, shown under the line until read. */
  let failure = $state<{ code: string; label: string } | null>(null)
  /** The attempt `failure` was read from; only a moved one replaces it. */
  let seenAttempt: number | null = null

  /** Where the popover is pinned: it is `fixed`, so the thread never clips it. */
  let spot = $state({ left: 0, top: 0 })

  let button = $state<HTMLButtonElement | null>(null)
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

  // One clock per turn: it starts the moment the mark becomes `Working`, ticks
  // while the line is drawn, and is thrown away - never left running - the moment
  // the mark changes. A remount starts from zero, which is the turn's own zero.
  $effect(() => {
    if (mark !== 'Working') {
      seconds = 0
      return
    }
    const start = Date.now()
    seconds = 0
    const tick = setInterval(() => {
      seconds = Math.floor((Date.now() - start) / 1000)
    }, 1000)
    return () => clearInterval(tick)
  })

  // A pushed state whose attempt moved is an ANSWERED write: it ends the wait and
  // is what says whether the change happened (`attempt` only counts writes, so a
  // read never ends it). A refusal keeps the mode and leaves its own line.
  $effect(() => {
    const view = sessionMode
    if (view === null) return
    if (seenAttempt === null) {
      seenAttempt = view.attempt
      return
    }
    if (view.attempt === seenAttempt) return
    seenAttempt = view.attempt
    failure =
      view.refusal === null ? null : { code: view.refusal, label: modeLabel(view.mode) }
  })

  // The pending the pushed attempt ends.
  $effect(() => {
    if (!pending) return
    const attempt = sessionMode?.attempt ?? null
    if (attempt !== null && attempt !== pendingAttempt) endPending()
  })

  // A pending nobody answers drops silently, keeping the mode it had.
  $effect(() => () => clearPendingTimer())

  // The popover is pinned above or below the control, whichever side has room,
  // and never wider than the window.
  $effect(() => {
    if (!open || pop === null || button === null) return
    const rect = button.getBoundingClientRect()
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
    if (!open) return
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
    if (!open) return
    const onDown = (event: PointerEvent) => {
      const target = event.target
      if (!(target instanceof Node)) return
      if (pop?.contains(target) === true || button?.contains(target) === true) return
      close(false)
    }
    document.addEventListener('pointerdown', onDown)
    return () => document.removeEventListener('pointerdown', onDown)
  })

  function clearPendingTimer(): void {
    if (pendingTimer === null) return
    clearTimeout(pendingTimer)
    pendingTimer = null
  }

  /** Starts the wait: the row keeps its text until the attempt moves. */
  function beginPending(): void {
    pending = true
    pendingAttempt = sessionMode?.attempt ?? null
    clearPendingTimer()
    pendingTimer = setTimeout(endPending, PENDING_MS)
  }

  function endPending(): void {
    pending = false
    pendingAttempt = null
    clearPendingTimer()
  }

  /** Closes the popover, leaving the focus on the control when asked to. */
  function close(refocus: boolean): void {
    open = false
    confirming = null
    if (refocus) button?.focus()
  }

  function toggle(): void {
    // While the channel owes an answer the control neither opens nor sends a
    // second change: one write at a time.
    if (pending) return
    if (open) {
      close(true)
      return
    }
    // Opening is how the reader asks again, so the last refusal is dismissed.
    failure = null
    confirming = null
    open = true
  }

  /** Sends one change: the only place a `SetSessionMode` ever leaves. */
  function send(choice: (typeof MODE_CHOICES)[number]): void {
    if (chatId === null || pending) return
    beginPending()
    close(true)
    void dispatchIntent({ type: 'SetSessionMode', data: { chat_id: chatId, mode: choice } })
  }

  function choose(choice: (typeof MODE_CHOICES)[number]): void {
    if (modeIsDanger(choice)) {
      confirming = choice
      return
    }
    send(choice)
  }

  function confirm(): void {
    const choice = confirming
    if (choice === null) return
    send(choice)
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
    close(true)
  }
</script>

{#if mark === 'Working'}
  <div class="work">
    <p class="work-line">
      <span class="spin" aria-hidden="true"></span>
      <span class="t">{copy.working}</span>
      <span class="n" aria-hidden="true">{seconds}s</span>
      {#if sessionMode !== null}
        <button
          type="button"
          class="mode"
          class:hot={modeIsDanger(sessionMode.mode)}
          class:wait={pending}
          aria-haspopup="menu"
          aria-expanded={open}
          aria-busy={pending}
          aria-label={pending ? copy.modeChanging(label) : copy.modeLabel(label)}
          bind:this={button}
          onclick={toggle}
        >
          <span class="mv">{label}</span>
          {#if pending}
            <span class="dots" aria-hidden="true"></span>
          {:else}
            <Icon name="chevronDown" size={9} />
          {/if}
        </button>
      {/if}
    </p>
    {#if failure !== null}
      <!-- The refusal of sheet C: the code, and the mode that did not change. -->
      <p class="mode-fail" role="alert">
        <span>{copy.modeRefused}</span>
        <code>{failure.code}</code>
        <span class="keep">{copy.modeKept(failure.label)}</span>
      </p>
    {/if}
    {#if open && sessionMode !== null}
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
             Cancel holds the first focus, Esc cancels, and only the danger
             button sends. -->
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
    {/if}
  </div>
{/if}
