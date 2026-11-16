<script lang="ts">
  // The first run, as the states board draws it: no workspace yet, so the way
  // forward is New workspace. The prompt opens the header's own menu, so there
  // is still one menu, on the icon; the reason line is the one New session is
  // described by, so the disabled button and this block say the same thing once.
  import { language } from '../i18n/language.svelte'
  import { sidebarCopy } from './copy'

  let {
    title = '',
    reason = '',
    reasonId = 'workspace-empty-why',
    onNewWorkspace,
  }: {
    title?: string
    reason?: string
    reasonId?: string
    /** Opens the New workspace menu of the header. */
    onNewWorkspace: () => void
  } = $props()

  /** The block's own way forward, in the language this run is set to (D4). */
  const copy = $derived(sidebarCopy(language.current))
</script>

<div class="empty">
  <p class="et">{title}</p>
  <p class="why" id={reasonId}>{reason}</p>
  <button class="btn" type="button" onclick={onNewWorkspace}>{copy.newWorkspace}</button>
</div>
