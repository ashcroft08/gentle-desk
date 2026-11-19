<script lang="ts">
  // The one action of the thread: copy the finished message's raw markdown. The
  // intent names the message id; the clipboard write itself is the shell's
  // (T5a), never the view's.
  //
  // It is an ICON that only exists while the pointer is over the message (or the
  // button itself has focus): a word on every finished reply is furniture, and the
  // thread is where the reader looks. The word survives as the accessible name and
  // the tooltip, so nothing is lost to a keyboard or a screen reader.
  import { copyMessage } from '../bridge'
  import { language } from '../i18n/language.svelte'
  import { threadCopy } from './copy'

  let { messageId }: { messageId: string } = $props()

  /** Copy's word in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))
</script>

<button class="copy" type="button" aria-label={copy.copy} title={copy.copy} onclick={() => void copyMessage(messageId)}>
  <svg viewBox="0 0 24 24" aria-hidden="true"><rect x="9" y="9" width="11" height="11" rx="2" /><path d="M15 5.5H6.5A1.5 1.5 0 005 7v8.5" /></svg>
</button>
