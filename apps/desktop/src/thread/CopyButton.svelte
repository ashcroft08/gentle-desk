<script lang="ts">
  // The one action of the thread: copy the finished message's raw markdown. The
  // intent names the message id; the clipboard write itself is the shell's
  // (T5a), never the view's.
  import { copyMessage } from '../bridge'
  import { language } from '../i18n/language.svelte'
  import { threadCopy } from './copy'

  let { messageId }: { messageId: string } = $props()

  /** Copy's word in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))
</script>

<button class="copy" type="button" onclick={() => void copyMessage(messageId)}>{copy.copy}</button>
