<script lang="ts">
  // Stop: what replaces Send while the chat is working (01a, `#h-stop`, which
  // the design also draws in the Helpers thread bar). It sends the Abort intent
  // through the bridge; the state, not the view, decides what a stop does with a
  // queued prompt.
  import Icon from '../icons/Icon.svelte'
  import { language } from '../i18n/language.svelte'
  import { composerCopy } from './copy'

  let { onStop }: { onStop: () => void } = $props()

  /** The word this button draws, in the language the app holds (0.2.0, D4). */
  const copy = $derived(composerCopy(language.current))
</script>

<button
  class="ic danger tip-r"
  type="button"
  data-tip={copy.stop}
  aria-label={copy.stop}
  onclick={onStop}
>
  <Icon name="stop" />
</button>
