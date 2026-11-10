<script lang="ts">
  // One item of a helper's thread, drawn the way iced draws it
  // (`crates/ui/src/view/helper_thread.rs`):
  //
  // - Said: the helper's own words, as markdown through the thread's one
  //   renderer, so a link opens through `open_link` and a hostile payload is
  //   sanitized before it reaches the document;
  // - Thought and Note: plain muted text, the note never markdown;
  // - Ran: a tool call, with its name, the word of its status (the text carries
  //   the state, never the colour alone), its arguments and the first lines of
  //   its output in a monospaced block.
  //
  // Nothing here decides what to show: iced always shows the tools, and the
  // strings arrive already built by `gentle_runtime::helper_text`.
  import type { HelperThreadItemView as Item } from '../contract/bindings'
  import Markdown from '../thread/Markdown.svelte'
  import { toolStatusWord } from './copy'

  let {
    item,
    html = '',
  }: {
    item: Item
    /** The markdown html of a `Said` item; empty for every other kind. */
    html?: string
  } = $props()
</script>

{#if item.kind === 'Said'}
  <div class="said">
    <Markdown {html} />
  </div>
{:else if item.kind === 'Thought'}
  <p class="thought">{item.text}</p>
{:else if item.kind === 'Note'}
  <p class="note">{item.text}</p>
{:else}
  <div class="ran">
    <p class="head">
      <span class="tool">{item.name}</span>
      <span class="status" data-status={item.status}>{toolStatusWord(item.status)}</span>
    </p>
    <pre class="detail">{item.args_line}</pre>
    {#if item.output_preview !== ''}
      <pre class="detail out">{item.output_preview}</pre>
    {/if}
  </div>
{/if}

<style>
  p {
    margin: 0;
    padding: 0;
  }

  .thought,
  .note {
    color: var(--muted);
    font-size: 12px;
  }

  .thought {
    white-space: pre-wrap;
  }

  .ran {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 10px;
    border-radius: 8px;
    background: var(--surface2);
  }

  .head {
    display: flex;
    align-items: baseline;
    gap: 8px;
    font-size: 13px;
  }

  .tool {
    font-weight: 600;
  }

  .status {
    font-size: 11px;
  }

  .status[data-status='Running'] {
    color: var(--accent);
  }

  .status[data-status='Failed'] {
    color: var(--danger);
  }

  .status[data-status='Ok'] {
    color: var(--success);
  }

  .detail {
    margin: 0;
    color: var(--muted);
    font-family: ui-monospace, 'Cascadia Code', Consolas, monospace;
    font-size: 11px;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .out {
    color: var(--faint);
  }
</style>
