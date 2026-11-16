<script lang="ts">
  // "New session" (D2): one new chat in the current workspace. With none it is
  // disabled the way the states board draws it, with `aria-disabled` and a
  // visible reason the button is described by, so a keyboard or screen-reader
  // user still reaches the reason (D4). The click handler stays, because the
  // button keeps its focus; it just does nothing while disabled.
  import Icon from '../icons/Icon.svelte'
  import { language } from '../i18n/language.svelte'
  import { NEW_SESSION_WHY_ID, sidebarCopy } from './copy'

  let {
    enabled = false,
    reason = null,
    describedBy = null,
    onNewSession,
  }: {
    enabled?: boolean
    /** The reason line drawn under the button; `null` when another element says it. */
    reason?: string | null
    /** The element the button is described by while it is disabled. */
    describedBy?: string | null
    onNewSession: () => void
  } = $props()

  /** The button's own word, in the language this run is set to (0.2.0, S3/D4). */
  const copy = $derived(sidebarCopy(language.current))

  function activate(): void {
    if (enabled) onNewSession()
  }
</script>

<button
  class="new"
  type="button"
  aria-disabled={enabled ? undefined : 'true'}
  aria-describedby={enabled ? undefined : (describedBy ?? undefined)}
  onclick={activate}
>
  <Icon name="plus" />
  {copy.newSession}
</button>

{#if reason !== null}
  <p class="why" id={NEW_SESSION_WHY_ID}>{reason}</p>
{/if}
