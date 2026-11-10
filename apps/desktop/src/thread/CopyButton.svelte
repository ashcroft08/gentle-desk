<script lang="ts">
  // The one action of the thread: copy the finished message's raw markdown. The
  // intent names the message id; the clipboard write itself is the shell's
  // (T5a), never the view's.
  import { copyMessage } from '../bridge'
  import { COPY } from './copy'

  let { messageId }: { messageId: string } = $props()
</script>

<button class="copy" type="button" onclick={() => void copyMessage(messageId)}>{COPY}</button>
