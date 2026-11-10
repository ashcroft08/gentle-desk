<script lang="ts">
  // A user message is PLAIN text (crates/ui/src/view/thread.rs draws it with a
  // `text` widget): the markdown a person typed is shown as typed, so what the
  // model receives is what the person sees. CSS keeps the line breaks.
  //
  // The images the message was sent with are drawn above the text, as the same
  // thumbnails the composer showed: the shell serves them from the dropped file,
  // so the thread holds a URL and never a byte.
  import type { ImageView } from '../contract/bindings'

  let { text = '', images = [] }: { text?: string; images?: ImageView[] } = $props()
</script>

{#if images.length > 0}
  <ul class="shots">
    {#each images as image (image.id)}
      <li class="shot">
        <img class="thumb" src={image.preview_url} alt={image.name} />
      </li>
    {/each}
  </ul>
{/if}
<p class="ut">{text}</p>
