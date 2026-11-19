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
  import type {
    ChatActivityMark,
    ContextUsageView_Serialize as ContextUsageView,
    FolderView_Serialize as FolderView,
    ImageView,
    ModelChipView,
  } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'
  import { language } from '../i18n/language.svelte'
  import ContextRail from './ContextRail.svelte'
  import FolderChip from './FolderChip.svelte'
  import ModelChip from './ModelChip.svelte'
  import { SEND_WHY_ID, composerCopy } from './copy'
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
    contextUsage = null,
    folder = null,
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
    /** How full this chat's context is (0.1.12, S5), as Rust pushes it. */
    contextUsage?: ContextUsageView | null
    /** The folder this chat runs in (design/composer-dock S2, R4): the last
     * segment of its working directory and the whole path, or `null` for a chat
     * whose session is not listed yet - absent, never a placeholder. */
    folder?: FolderView | null
  } = $props()

  /** The composer's own words in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(composerCopy(language.current))

  /** The drafts, one per chat id, kept for as long as the box is on screen. */
  let drafts = $state<Record<string, string>>({})
  /** The textarea itself, for the focus and the height measurement. */
  let box = $state<HTMLTextAreaElement | null>(null)
  /** The chat whose draft the box holds, so a switch loads the new one once. */
  let shown: string | null = null
  /** Whether files are being dragged over the window: the composer says so. */
  let dropping = $state(false)

  const rule = $derived(composerStateOf(mark, copy))
  const text = $derived(chat === null ? '' : (drafts[chat] ?? ''))

  /**
   * Whether a send would do anything: a chat, a box that is actually Send (not
   * Stop), an unlocked composer, real text. A working or queued chat shows Stop
   * because it takes one turn at a time, so an Enter there sends nothing and the
   * draft stays where it is for the turn after (the reducer would drop it).
   */
  const sendable = $derived(
    chat !== null && rule.mode === 'send' && !rule.disabled && text.trim() !== '',
  )

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
    // A browser reports scrollHeight as at least the current height, so the
    // old size must be dropped first: without this reset a box that grew with
    // a long draft keeps measuring itself as grown after the send (L9).
    box.style.height = 'auto'
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
  {#if dropping}
    <!-- Where the files go while they are still in the air. -->
    <p class="drop">{copy.dropHint}</p>
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
            aria-label={`${copy.removeImage} ${image.name}`}
            onclick={() => take(image.id)}
          >
            <Icon name="x" size={12} />
          </button>
        </li>
      {/each}
    </ul>
  {/if}
  <!-- The context rail ABOVE the box and outside its border (design/composer-dock
       S1, S5): it replaced the chip that used to share the row with the pills. -->
  <ContextRail usage={contextUsage} />
  <form class="composer" class:over={dropping} onsubmit={submitted}>
    <textarea
      id="msg"
      rows="1"
      aria-label={copy.messageLabel}
      placeholder={copy.messagePlaceholder}
      bind:this={box}
      value={text}
      oninput={typed}
      onkeydown={keyed}
    ></textarea>
    <!-- The row, regrouped (design/composer-dock): what the message runs ON
         (model, effort), a hairline, then where it runs (the folder - the branch
         is a later slice, so nothing sits in its place), the flexible space, and
         Send. Every part is text: no box, no chip border. -->
    <div class="row">
      <ModelChip {chip} />
      <FolderChip {folder} />
      <span class="grow"></span>
      {#if rule.mode === 'stop'}
        <StopButton onStop={stop} />
      {:else}
        <SendButton disabled={rule.disabled} describedBy={SEND_WHY_ID} />
      {/if}
    </div>
  </form>
  {#if rule.reason !== null}
    <!-- The visible reason for a disabled Send, which Send is described by. -->
    <p class="why" id={SEND_WHY_ID}>{rule.reason}</p>
  {/if}
  {#if imagesNotice !== null}
    <!-- Why the last drop attached nothing, in the words Rust refused it with. -->
    <p class="refused" role="status">{imagesNotice}</p>
  {/if}
  <!-- The keys BELOW the box's border, left-aligned, in the app's own words (S7):
       Stop mode is Working or Queued, where Enter sends nothing, so the hint stops
       promising a send the composer will not do. -->
  <p class="keys">{rule.mode === 'stop' ? copy.keysHintWorking : copy.keysHint}</p>
</div>
