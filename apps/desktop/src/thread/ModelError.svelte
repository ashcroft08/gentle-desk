<script lang="ts">
  // A model error replaces the reply (crates/ui draws a danger box there too):
  // the title, then the provider's own text. No glyph is drawn: the iced box
  // leads with a cross mark, and this thread draws no icon characters.
  import { MODEL_ERROR_TITLE } from './copy'

  let { error }: { error: string } = $props()
</script>

<div class="err">
  <p class="et">{MODEL_ERROR_TITLE}</p>
  <p class="em">{error}</p>
</div>
