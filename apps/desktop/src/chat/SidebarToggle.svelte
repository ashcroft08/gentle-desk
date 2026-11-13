<script lang="ts">
  // The sidebar collapse toggle (responsive-sidebar S1): one button, drawn in
  // both headers, that names what it does and reports whether the sidebar is
  // expanded. The collapse rule lives in `../sidebar/collapse`; this control
  // only passes the user's toggle through.
  import Icon from '../icons/Icon.svelte'
  import { SIDEBAR_TOGGLE } from '../composer/copy'

  let {
    collapsed = false,
    onToggle = () => {},
  }: {
    /** Whether the sidebar is collapsed right now, for `aria-expanded`. */
    collapsed?: boolean
    onToggle?: () => void
  } = $props()
</script>

<button
  type="button"
  class="side-toggle ic"
  aria-label={SIDEBAR_TOGGLE}
  aria-expanded={!collapsed}
  onclick={onToggle}
>
  <Icon name="menu" size={18} />
</button>
