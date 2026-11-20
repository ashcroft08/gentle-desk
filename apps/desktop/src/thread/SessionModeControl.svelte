<script lang="ts">
  // The session mode control (design/composer-dock S1, T3c; sheet D of
  // `02-mode-picker.html`): a quiet text button with a caret at the right edge of
  // the status row, opening the four modes the channel knows.
  //
  // The row is PERMANENT since sheet D: it is drawn whether the chat is working
  // or idle, and the control sits at its right edge in both states, so the mode
  // can be changed at any time. Only the row's left side belongs to the turn - the
  // clock `./WorkingLine` hands in as this component's content, drawn while the
  // mark is `Working` and absent otherwise.
  //
  // Normal and Auto change at once; YOLO and Super YOLO ask first in the same
  // popover (`./SessionModeMenu`, sheet B), and only its danger button sends.
  //
  // The shown mode NEVER changes optimistically: a write keeps the previous text
  // beside a spinner until the pushed `attempt` moves, which is the shell's own
  // count of answered writes, and a wait nothing answers drops silently after
  // `PENDING_MS`. A refused answer says its code, in the line under this one, and
  // leaves the mode exactly as it was (sheet C). A chat the shell answered
  // nothing for draws NO control: no button, no gap. It still draws the row - it
  // may simply be empty, and that is what an unanswered read looks like.
  //
  // This component draws the status row's two wrappers because its own parts live
  // on both sides of it: the button inside `.work-line`, the refusal line beside
  // it under the row (`.work` is the column that spaces them, and app.css owns
  // both). The clock the row reports is the caller's, handed in as its content.
  import type { Snippet } from 'svelte'
  import type { SessionModeView_Serialize as SessionModeView } from '../contract/bindings'
  import { dispatchIntent } from '../bridge'
  import Icon from '../icons/Icon.svelte'
  import { language } from '../i18n/language.svelte'
  import { threadCopy } from './copy'
  import { MODE_CHOICES, modeIsDanger, modeLabel } from './session-mode'
  import SessionModeMenu from './SessionModeMenu.svelte'

  let {
    chatId = null,
    sessionMode = null,
    children,
  }: {
    /** The foreground chat the write names, or `null` while none is open. */
    chatId?: string | null
    /** What that chat's child reported (T3b), or `null` when nothing answered. */
    sessionMode?: SessionModeView | null
    /** The clock the line draws before the control (this row is its content). */
    children?: Snippet
  } = $props()

  /** How long a sent change waits for an answer before the spinner drops. */
  const PENDING_MS = 8000

  /** The line's own word in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))

  /** The mode in effect, as the row and the menu name it; empty with no mode. */
  const label = $derived(sessionMode === null ? '' : modeLabel(sessionMode.mode))

  /** Whether the popover is open (the menu it draws owns its own two steps). */
  let open = $state(false)

  /** A change was sent and no answer has moved the attempt yet. */
  let pending = $state(false)
  /** The attempt the pending write started from; a different one ends the wait. */
  let pendingAttempt: number | null = null
  let pendingTimer: ReturnType<typeof setTimeout> | null = null

  /** The last refusal the pushed state brought, shown under the line until read. */
  let failure = $state<{ code: string; label: string } | null>(null)
  /** The attempt `failure` was read from; only a moved one replaces it. */
  let seenAttempt: number | null = null

  let button = $state<HTMLButtonElement | null>(null)

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
    open = true
  }

  /** Sends one change: the only place a `SetSessionMode` ever leaves. */
  function send(choice: (typeof MODE_CHOICES)[number]): void {
    if (chatId === null || pending) return
    beginPending()
    close(true)
    void dispatchIntent({ type: 'SetSessionMode', data: { chat_id: chatId, mode: choice } })
  }
</script>

<div class="work">
  <p class="work-line">
    {@render children?.()}
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
    <SessionModeMenu {sessionMode} anchor={button} onsend={send} onclose={close} />
  {/if}
</div>
