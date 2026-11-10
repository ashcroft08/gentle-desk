<script lang="ts">
  // An assistant message: its markdown, and Copy once it is finished. The
  // streaming case draws the body alone (crates/ui/src/view/thread.rs: no Copy
  // before `message_end`), and an errored message never reaches this component.
  import CopyButton from './CopyButton.svelte'
  import Markdown from './Markdown.svelte'

  let {
    id,
    html = '',
    copyable = false,
  }: { id: string; html?: string; copyable?: boolean } = $props()
</script>

<Markdown {html} />
{#if copyable}
  <CopyButton messageId={id} />
{/if}
