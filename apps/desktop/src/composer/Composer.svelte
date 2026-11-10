<script lang="ts">
  // The prompt box (01a, `.composer`): one textarea that grows with what is
  // typed, and one button that is Send or Stop depending on the chat's state.
  //
  // Three things live here and nowhere else, because they are the view's own
  // business (S2: Rust owns the state, not the text being typed):
  //
  // - one draft per chat, so switching chats and coming back finds the text;
  // - the keyboard: Enter sends, Shift+Enter is a newline, and an Enter that
  //   commits an IME composition belongs to the candidate list;
  // - the focus: the box takes it when a chat opens and keeps it after a send,
  //   and Escape never touches the draft.
  import { dispatchIntent } from '../bridge'
  import type { ChatActivityMark } from '../contract/bindings'
  import { KEYS_HINT, MESSAGE_LABEL, MESSAGE_PLACEHOLDER, SEND_WHY_ID } from './copy'
  import { fittedHeight, shouldSend } from './keys'
  import SendButton from './SendButton.svelte'
  import { composerStateOf } from './state'
  import StopButton from './StopButton.svelte'

  let {
    chat,
    mark = null,
  }: {
    /** The foreground chat: which draft is shown, and where a send goes. */
    chat: string | null
    /** Its mark, `null` when it is idle; it decides Send, Stop and the reason. */
    mark?: ChatActivityMark | null
  } = $props()

  /** The drafts, one per chat id, kept for as long as the box is on screen. */
  let drafts = $state<Record<string, string>>({})
  /** The textarea itself, for the focus and the height measurement. */
  let box = $state<HTMLTextAreaElement | null>(null)
  /** The chat whose draft the box holds, so a switch loads the new one once. */
  let shown: string | null = null

  const rule = $derived(composerStateOf(mark))
  const text = $derived(chat === null ? '' : (drafts[chat] ?? ''))

  /** Whether a send would do anything: a chat, an unlocked composer, real text. */
  const sendable = $derived(chat !== null && !rule.disabled && text.trim() !== '')

  // Opening a chat, or starting one, puts the cursor where the user will type.
  // It runs only when the chat changes, never on every keystroke.
  $effect(() => {
    const open = chat
    if (open === null || open === shown) return
    shown = open
    box?.focus()
  })

  // The box grows with what is typed, up to the artboard's maximum.
  $effect(() => {
    // Reading the draft is what makes this follow a push and the typing alike.
    void text
    if (box === null) return
    const height = fittedHeight(box.scrollHeight)
    box.style.height = height === 0 ? 'auto' : `${height}px`
  })

  function typed(event: Event): void {
    if (chat === null) return
    const value = (event.currentTarget as HTMLTextAreaElement).value
    drafts = { ...drafts, [chat]: value }
  }

  function keyed(event: KeyboardEvent): void {
    // Escape stops a running turn, the same Abort the Stop button sends; with
    // nothing running it keeps the meaning the box gives it.
    if (event.key === 'Escape' && rule.mode === 'stop') {
      event.preventDefault()
      stop()
      return
    }
    // Only a send takes the Enter: with nothing to send, or a chat that cannot
    // take a prompt, the key keeps the meaning the box gives it.
    if (!shouldSend(event) || !sendable) return
    event.preventDefault()
    send()
  }

  function submitted(event: SubmitEvent): void {
    event.preventDefault()
    send()
  }

  /** Sends the trimmed draft and keeps the cursor in the box; nothing else happens. */
  function send(): void {
    if (!sendable || chat === null) return
    const value = text.trim()
    drafts = { ...drafts, [chat]: '' }
    void dispatchIntent({ type: 'SendPrompt', data: { text: value } })
    box?.focus()
  }

  function stop(): void {
    void dispatchIntent({ type: 'Abort' })
  }
</script>

<div class="composer-dock">
  <form class="composer" onsubmit={submitted}>
    <textarea
      id="msg"
      rows="1"
      aria-label={MESSAGE_LABEL}
      placeholder={MESSAGE_PLACEHOLDER}
      bind:this={box}
      value={text}
      oninput={typed}
      onkeydown={keyed}
    ></textarea>
    {#if rule.mode === 'stop'}
      <StopButton onStop={stop} />
    {:else}
      <SendButton disabled={rule.disabled} describedBy={SEND_WHY_ID} />
    {/if}
  </form>
  {#if rule.reason !== null}
    <!-- The visible reason for a disabled Send, which Send is described by. -->
    <p class="why" id={SEND_WHY_ID}>{rule.reason}</p>
  {/if}
  <p class="keys">{KEYS_HINT}</p>
</div>
