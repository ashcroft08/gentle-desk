<script lang="ts">
  // The prompt box (01a, `.composer`): one textarea that grows with what is
  // typed, and one button that is Send or Stop depending on the chat's state.
  //
  // Four things live here and nowhere else, because they are the view's own
  // business (S2: Rust owns the state, not the text being typed):
  //
  // - one draft per chat, so switching chats and coming back finds the text;
  // - the keyboard: Enter sends, Shift+Enter is a newline, and an Enter that
  //   commits an IME composition belongs to the candidate list;
  // - the focus: the box takes it when a chat opens and keeps it after a send,
  //   and Escape never touches the draft;
  // - the drop: files dragged over the window land here, and the composer hands
  //   Rust their PATHS so it can read them (`./drop`). Attaching never sends, and
  //   a thumbnail is drawn from the URL Rust answers with, never from a byte the
  //   page holds.
  import { attachImages, dispatchIntent, removeImage } from '../bridge'
  import type { ChatActivityMark, ImageView, ModelChipView } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'
  import ModelChip from './ModelChip.svelte'
  import {
    DROP_HINT,
    KEYS_HINT,
    MESSAGE_LABEL,
    MESSAGE_PLACEHOLDER,
    REMOVE_IMAGE,
    SEND_WHY_ID,
  } from './copy'
  import { watchDrops } from './drop'
  import { fittedHeight, shouldSend } from './keys'
  import SendButton from './SendButton.svelte'
  import { composerStateOf } from './state'
  import StopButton from './StopButton.svelte'

  let {
    chat,
    mark = null,
    images = [],
    imagesNotice = null,
    chip = null,
  }: {
    /** The foreground chat: which draft is shown, and where a send goes. */
    chat: string | null
    /** Its mark, `null` when it is idle; it decides Send, Stop and the reason. */
    mark?: ChatActivityMark | null
    /** The images the chat holds, as Rust pushes them: a name, a preview URL and
     * the id the remove names back. The page never holds a byte of an image. */
    images?: ImageView[]
    /** Why the last drop was refused, as one plain line, or `null`. */
    imagesNotice?: string | null
    /** What the model chip shows, as Rust pushes it, or `null`. */
    chip?: ModelChipView | null
  } = $props()

  /** The drafts, one per chat id, kept for as long as the box is on screen. */
  let drafts = $state<Record<string, string>>({})
  /** The textarea itself, for the focus and the height measurement. */
  let box = $state<HTMLTextAreaElement | null>(null)
  /** The chat whose draft the box holds, so a switch loads the new one once. */
  let shown: string | null = null
  /** Whether files are being dragged over the window: the composer says so. */
  let dropping = $state(false)

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

  // One drop listener for as long as the composer is on screen: Tauri's own
  // event, because a dropped `File` has no path to give. What arrives is a
  // decision (`dropAction`), and a drop of paths asks Rust to read them.
  $effect(() => {
    let stop: (() => void) | null = null
    let live = true
    void watchDrops((action) => {
      dropping = action.kind === 'over'
      if (action.kind === 'drop' && action.paths.length > 0) void attachImages(action.paths)
    }).then((off) => {
      if (live) stop = off
      else off()
    })
    return () => {
      live = false
      stop?.()
    }
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

  /** Takes one attached image away. Nothing is sent; Rust answers the slice. */
  function take(id: string): void {
    void removeImage(id)
  }
</script>

<div class="composer-dock">
  <ModelChip {chip} />
  {#if dropping}
    <!-- Where the files go while they are still in the air. -->
    <p class="drop">{DROP_HINT}</p>
  {/if}
  {#if images.length > 0}
    <!-- What the chat holds before the send: the same thumbnails the sent
         message draws, each one with its own way to take it away. -->
    <ul class="shots">
      {#each images as image (image.id)}
        <li class="shot">
          <img class="thumb" src={image.preview_url} alt={image.name} title={image.name} />
          <button
            class="take"
            type="button"
            aria-label={`${REMOVE_IMAGE} ${image.name}`}
            onclick={() => take(image.id)}
          >
            <Icon name="x" size={12} />
          </button>
        </li>
      {/each}
    </ul>
  {/if}
  <form class="composer" class:over={dropping} onsubmit={submitted}>
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
  {#if imagesNotice !== null}
    <!-- Why the last drop attached nothing, in the words Rust refused it with. -->
    <p class="refused" role="status">{imagesNotice}</p>
  {/if}
  <p class="keys">{KEYS_HINT}</p>
</div>
