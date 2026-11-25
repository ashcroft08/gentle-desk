<script lang="ts">
  // The sidebar collapse toggle (responsive-sidebar S1): one button, drawn in
  // both headers, that names what it does and reports whether the sidebar is
  // expanded. The collapse rule lives in `../sidebar/collapse`; this control
  // only passes the user's toggle through. Its word is the composer's own (the
  // toggle sits in the composer's dock area too), so it reads the same dictionary
  // in the language the app holds (0.2.0, S3/D4).
  import Icon from '../icons/Icon.svelte'
  import { language } from '../i18n/language.svelte'
  import { composerCopy } from '../composer/copy'

  let {
    collapsed = false,
    onToggle = () => {},
  }: {
    /** Whether the sidebar is collapsed right now, for `aria-expanded`. */
    collapsed?: boolean
    onToggle?: () => void
  } = $props()

  /** The toggle's own word in the language the app holds (0.2.0, D4). */
  const copy = $derived(composerCopy(language.current))
</script>

<button
  type="button"
  class="side-toggle ic"
  aria-label={copy.sidebarToggle}
  aria-expanded={!collapsed}
  onclick={onToggle}
>
  <Icon name="panel-left" size={18} />
</button>
