<script lang="ts">
  // A model error replaces the reply (crates/ui draws a danger box there too):
  // the title, then the provider's own text. No glyph is drawn: the iced box
  // leads with a cross mark, and this thread draws no icon characters.
  import { language } from '../i18n/language.svelte'
  import { threadCopy } from './copy'

  let { error }: { error: string } = $props()

  /** The box's title in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))
</script>

<div class="err">
  <p class="et">{copy.modelErrorTitle}</p>
  <p class="em">{error}</p>
</div>
