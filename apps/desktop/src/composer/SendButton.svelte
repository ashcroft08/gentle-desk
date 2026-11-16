<script lang="ts">
  // Send: the composer's one filled button (01a, `#send`). Disabled with
  // `aria-disabled`, never taken away, so a keyboard or screen-reader user still
  // reaches the line that says why (the states board's rule for a disabled
  // control: the faded look is never the only signal).
  import Icon from '../icons/Icon.svelte'
  import { language } from '../i18n/language.svelte'
  import { composerCopy } from './copy'

  let { disabled = false, describedBy = null }: {
    disabled?: boolean
    /** The element this button is described by while it is disabled. */
    describedBy?: string | null
  } = $props()

  /** The word this button draws, in the language the app holds (0.2.0, D4). */
  const copy = $derived(composerCopy(language.current))
</script>

<button
  class="ic fill tip-r"
  type="submit"
  data-tip={copy.send}
  aria-label={copy.send}
  aria-disabled={disabled ? 'true' : undefined}
  aria-describedby={disabled ? (describedBy ?? undefined) : undefined}
>
  <Icon name="send" />
</button>
