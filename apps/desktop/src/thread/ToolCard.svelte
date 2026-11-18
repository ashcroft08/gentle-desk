<script lang="ts">
  // ONE tool call, drawn as ONE quiet card (chat-transcript T2b, S3, L14): the
  // header (the kind's glyph, the tool's own name, a chip only when that name names
  // a language, and the chevron that folds the body away), the monospace subject,
  // the output folded to its tail behind an honest count, and the footer that
  // carries the STATE and the elapsed time - never a sentence.
  //
  // Three rules the card is the only thing that can keep:
  // - a failed call draws the first line of its output as the error line, so a
  //   folded body never swallows the reason it failed;
  // - the two payload losses are worded apart: the output was cut at the bound, or
  //   a field of it was dropped (`truncated` vs `dropped`, L10 R3-2);
  // - a call whose clock is unknown states its state and no number.
  //
  // Everything here is tokens from `design/system.md` and the house monospace stack,
  // so the six themes draw the card with no colour of its own. No part of the card
  // measures the DOM: the fold is a slice of the text, counted once per push, so a
  // chat of 400 calls stays a fixed amount of work per row.
  import type { ToolCallItemView_Serialize as ToolCall } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import ToolIcon from './ToolIcon.svelte'
  import { threadCopy } from './copy'
  import { FOLD_LINES, foldLines, formatDuration, kindChip, toolBody, toolKind, toolSubject } from './card'

  let { tool }: { tool: ToolCall } = $props()

  /** The card's own words in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))

  const kind = $derived(toolKind(tool.tool_name))
  const chip = $derived(kindChip(tool.tool_name))
  const subject = $derived(toolSubject(tool))
  const body = $derived(toolBody(tool))
  const elapsed = $derived(formatDuration(tool.duration_ms))
  const state = $derived(
    tool.running ? copy.toolRunning : tool.is_error ? copy.toolFailed : copy.toolDone,
  )

  /** Whether the header's chevron leaves the body drawn (the footer always is). */
  let shown = $state(true)
  /** Whether the reader asked for the whole body instead of its tail. */
  let open = $state(false)

  const fold = $derived(foldLines(body.lines, open))
  /** The body's own id, so the chevron can name what it folds. */
  const detailId = $derived(`tool-body-${tool.id}`)
</script>

<div
  class="tool-card"
  class:run={tool.running}
  class:bad={tool.is_error && !tool.running}
>
  <div class="tc-head">
    <span class="tc-tool">
      <ToolIcon {kind} />
      {tool.tool_name}
    </span>
    {#if chip !== null}
      <span class="tc-chip">{chip}</span>
    {/if}
    <span class="tc-grow"></span>
    {#if tool.truncated}
      <span class="tc-cut">{copy.cutShort}</span>
    {/if}
    {#if tool.dropped}
      <span class="tc-cut">{copy.droppedField}</span>
    {/if}
    <button
      class="tc-chev"
      type="button"
      aria-expanded={shown}
      aria-controls={detailId}
      aria-label={shown ? copy.collapse : copy.expand}
      onclick={() => (shown = !shown)}
    >
      <svg viewBox="0 0 24 24" aria-hidden="true">
        {#if shown}
          <path d="M6 9l6 6 6-6" />
        {:else}
          <path d="M9 6l6 6-6 6" />
        {/if}
      </svg>
    </button>
  </div>

  <div class="tc-detail" id={detailId} hidden={!shown}>
    {#if subject.text !== ''}
      <p class="tc-cmd" class:shell={subject.shell}>{subject.text}</p>
    {/if}
    {#if body.error !== null || body.lines.length > 0}
      <div class="tc-out">
        {#if body.error !== null}
          <!-- The reason a call failed is never folded away behind the count. -->
          <span class="tc-line tc-line-error">{body.error}</span>
        {/if}
        {#each fold.shown as line}
          <span class="tc-line">{line}</span>
        {/each}
      </div>
      {#if body.lines.length > FOLD_LINES}
        <button class="tc-more" type="button" onclick={() => (open = !open)}>
          {#if open}
            {copy.showLess}
          {:else}
            {copy.earlierLines(fold.hidden)}
          {/if}
        </button>
      {/if}
    {/if}
  </div>

  <div class="tc-foot" class:run={tool.running} class:bad={tool.is_error && !tool.running}>
    {#if tool.running}
      <span class="tc-spin" aria-hidden="true"></span>
    {:else if tool.is_error}
      <svg class="mark tc-mark" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M6 6l12 12" />
        <path d="M18 6L6 18" />
      </svg>
    {:else}
      <svg class="mark tc-mark" viewBox="0 0 24 24" aria-hidden="true">
        <path d="M4 12l6 6L20 6" />
      </svg>
    {/if}
    <span class="tc-state">
      {state}{#if elapsed !== null}<span class="tc-time">{` ${elapsed}`}</span>{/if}
    </span>
  </div>
</div>
