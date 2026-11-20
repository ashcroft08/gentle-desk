<script lang="ts">
  // The reasoning row (chat-transcript T2b, S3): NOT a card. One quiet line with a
  // spark glyph, the word for "thought", the block's duration chip and a chevron;
  // the text opens behind a hairline. It is text, not a surface, so a thought never
  // competes with the tool call beside it - the tradeoff the artboard states.
  //
  // The block is open while it still streams, because that text is still arriving,
  // and folded once the child closes it. A block whose clock never saw it carries
  // the word alone: no chip, no invented number.
  import type { ThinkingItemView_Serialize as Thinking } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { threadCopy } from './copy'
  import { formatDuration } from './card'

  let { item }: { item: Thinking } = $props()

  /** The row's own words in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))
  const elapsed = $derived(formatDuration(item.duration_ms))

  /**
   * Whether the text is drawn. The row follows its block: open while the block
   * streams, because that text is still arriving, and folded the moment it closes.
   * The reader's own toggle holds in between - it is spent only when the block's
   * state changes, which is the one moment the row has something new to say.
   */
  let open = $state(false)
  const streaming = $derived(item.streaming)
  $effect(() => {
    open = streaming
  })

  const textId = $derived(`thought-${item.id}`)
</script>

<div class="think-row">
  <button
    class="think"
    type="button"
    aria-expanded={open}
    aria-controls={textId}
    onclick={() => (open = !open)}
  >
    <span class="spark" aria-hidden="true">✳</span>
    <span>{copy.thought}</span>
    {#if elapsed !== null}
      <span class="dur">{elapsed}</span>
    {/if}
    <svg class="chev" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M9 6l6 6-6 6" />
    </svg>
  </button>
  <p class="think-text" id={textId} hidden={!open}>{item.text}</p>
</div>
