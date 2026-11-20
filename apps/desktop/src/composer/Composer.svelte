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
  // - the attachments: the clip and the drop name PATHS, Rust says what each one
  //   is, an image keeps the road it already had (a thumbnail served from Rust)
  //   and every other file becomes a chip whose path the send appends below the
  //   text (`./attach`, `./drop`). Attaching never sends, and a file outside the
  //   chat's folder attaches anyway and says so (R5).
  import { dispatchIntent, removeImage } from '../bridge'
  import { language } from '../i18n/language.svelte'
  import { promptWithPaths } from './attach'
  import { emptyDraft, attachPaths, withoutPath, type AttachmentDraft } from './attachment-draft'
  import type { ComposerProps } from './props'
  import AttachButton from './AttachButton.svelte'
  import ImageShots from './ImageShots.svelte'
  import Attachments from './Attachments.svelte'
  import BranchChip from './BranchChip.svelte'
  import ContextRail from './ContextRail.svelte'
  import FolderChip from './FolderChip.svelte'
  import ModelChip from './ModelChip.svelte'
  import { SEND_WHY_ID, composerCopy } from './copy'
  import { watchDrops } from './drop'
  import { fittedHeight, shouldSend } from './keys'
  import SendButton from './SendButton.svelte'
  import { composerStateOf } from './state'
  import StopButton from './StopButton.svelte'

  let { chat, mark = null, images = [], imagesNotice = null, chip = null, contextUsage = null, folder = null, branch = null }: ComposerProps = $props()

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
  /** The chips of the message being written, and why the last attach refused
   * anything, as one draft (`./attachment-draft`). */
  let draft = $state<AttachmentDraft>(emptyDraft())
  /** The chat the chips belong to, so a switch clears them once. */
  let attachedFor: string | null = null

  const rule = $derived(composerStateOf(mark, copy))
  const text = $derived(chat === null ? '' : (drafts[chat] ?? ''))

  /**
   * Whether a send would do anything: a chat, a box that is actually Send (not
   * Stop), an unlocked composer, real text. A working or queued chat shows Stop
   * because it takes one turn at a time, so an Enter there sends nothing and the
   * draft stays where it is for the turn after (the reducer would drop it).
   */
  const sendable = $derived(
    chat !== null &&
      rule.mode === 'send' &&
      !rule.disabled &&
      (text.trim() !== '' || draft.attached.length > 0),
  )

  // A chat change drops the chips: they belong to the message being written in
  // THIS chat, exactly as the images Rust holds do. It runs only on the change,
  // never on an attach, so a chip is never lost by anything but its own chat.
  $effect(() => {
    const open = chat
    if (open === attachedFor) return
    attachedFor = open
    draft = emptyDraft()
  })

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
      if (action.kind === 'drop' && action.paths.length > 0) void attach(action.paths)
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
      void dispatchIntent({ type: 'Abort' })
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
  /** The clip and the drop both land here (T4, S3, R6): Rust says what each
   * path is, an image keeps the road it already had, every other file becomes a
   * chip, and a refused path draws ONE line instead. A picker or drop that
   * arrives after the chat changed attaches nothing: the chips belong to the
   * message of the chat they were chosen for. */
  async function attach(paths: string[]): Promise<void> {
    if (chat === null || paths.length === 0) return
    const chosenFor = chat
    const next = await attachPaths(draft, chat, paths, folder?.path ?? null, copy.attachFailed)
    if (chat !== chosenFor || next === null) return
    draft = next
  }

  /** Takes one chip away before the send. Nothing is sent and nothing is read;
   * a file outside the chat's folder is attached like any other (R5). */
  function remove(path: string): void {
    draft = withoutPath(draft, path)
  }

  /**
   * Sends the trimmed draft with the chips' whole paths below it, one per line
   * (T4, S3, R6), and clears both. A message with no text and only files is a
   * send like any other; the chips are cleared once it is dispatched, never on
   * an attach.
   */
  function send(): void {
    if (!sendable || chat === null) return
    const value = promptWithPaths(
      text,
      draft.attached.map((view) => view.path),
    )
    if (value === '') return
    drafts = { ...drafts, [chat]: '' }
    draft = emptyDraft()
    void dispatchIntent({ type: 'SendPrompt', data: { text: value } })
    box?.focus()
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
    <ImageShots {images} {copy} onTake={take} />
  {/if}
  <!-- The context rail ABOVE the box and outside its border (design/composer-dock
       S1, S5): it replaced the chip that used to share the row with the pills. -->
  <ContextRail usage={contextUsage} />
  <form class="composer" class:over={dropping} onsubmit={submitted}>
    <!-- The attached FILES are content, not chrome: their own row above the
         text, with the one line a refused path and the one line an outside file
         draw (design/composer-dock/03-attachments.html, sheets A, B, D). -->
    <Attachments attachments={draft.attached} onRemove={remove} />
    {#if draft.attachNotice !== null}
      <p class="attach-failed" role="alert">{draft.attachNotice}</p>
    {/if}
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
    <!-- The row, regrouped in THREE groups the eye can tell apart (maintainer's
         test note, 2026-11-19): the clip is its own act, the model and its
         effort are what the message runs ON, a hairline, then where it runs
         (the folder, and the branch that folder is on - absent when there is no
         repository to name), the flexible space and Send. The wider gap between
         the groups is what keeps them from reading as one control. -->
    <div class="row">
      <span class="grp"><AttachButton label={copy.attachFiles} disabled={chat === null} onPick={attach} /></span>
      <span class="grp"><ModelChip {chip} /></span>
      <span class="vsep" role="presentation"></span>
      <span class="grp"><FolderChip {folder} /><BranchChip {branch} /></span>
      <span class="grow"></span>
      {#if rule.mode === 'stop'}
        <StopButton
          onStop={() => dispatchIntent({ type: 'Abort' })}
        />
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
