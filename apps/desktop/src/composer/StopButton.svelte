<script lang="ts">
  // Stop: what replaces Send while the chat is working (01a, `#h-stop`, which
  // the design also draws in the Helpers thread bar). It sends the Abort intent
  // through the bridge; the state, not the view, decides what a stop does with a
  // queued prompt.
  import Icon from '../icons/Icon.svelte'
  import { STOP } from './copy'

  let { onStop }: { onStop: () => void } = $props()
</script>

<button
  class="ic danger tip-r"
  type="button"
  data-tip={STOP}
  aria-label={STOP}
  onclick={onStop}
>
  <Icon name="stop" />
</button>
