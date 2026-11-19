<script lang="ts">
  // The thumbnails of the images the chat holds before the send, each with its
  // own way to take it away (the composer's own anatomy, unchanged by T4).
  import type { ImageView } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'

  let {
    images,
    copy,
    onTake,
  }: {
    /** What Rust pushes: a name, a preview URL and the id the remove names back. */
    images: ImageView[]
    /** The composer's words, for the remove button's label. */
    copy: { removeImage: string }
    /** Takes one image away before the send. */
    onTake: (id: string) => void
  } = $props()
</script>

<ul class="shots">
  {#each images as image (image.id)}
    <li class="shot">
      <img class="thumb" src={image.preview_url} alt={image.name} title={image.name} />
      <button
        class="take"
        type="button"
        aria-label={`${copy.removeImage} ${image.name}`}
        onclick={() => onTake(image.id)}
      >
        <Icon name="x" size={12} />
      </button>
    </li>
  {/each}
</ul>
