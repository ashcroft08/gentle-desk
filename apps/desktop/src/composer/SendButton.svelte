<script lang="ts">
  // Send: the composer's one filled button (01a, `#send`). Disabled with
  // `aria-disabled`, never taken away, so a keyboard or screen-reader user still
  // reaches the line that says why (the states board's rule for a disabled
  // control: the faded look is never the only signal).
  import Icon from '../icons/Icon.svelte'
  import { SEND } from './copy'

  let { disabled = false, describedBy = null }: {
    disabled?: boolean
    /** The element this button is described by while it is disabled. */
    describedBy?: string | null
  } = $props()
</script>

<button
  class="ic fill tip-r"
  type="submit"
  data-tip={SEND}
  aria-label={SEND}
  aria-disabled={disabled ? 'true' : undefined}
  aria-describedby={disabled ? (describedBy ?? undefined) : undefined}
>
  <Icon name="send" />
</button>
