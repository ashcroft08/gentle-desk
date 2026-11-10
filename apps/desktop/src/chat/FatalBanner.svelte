<script lang="ts">
// The fatal bar: the one failure that ends the whole run instead of one chat,
// which is a launcher that cannot be resolved at all (crates/ui/src/view/
// banner.rs draws the same bar). It is not a notice: it does not dismiss, it
// offers no action, and it stays because every chat is impossible until the app
// is restarted with a launcher it can find.

import Icon from '../icons/Icon.svelte'

let { text }: { text: string } = $props()
</script>

<div class="fatal" role="alert">
  <Icon name="x" />
  <span class="m">{text}</span>
</div>
