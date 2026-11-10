<script lang="ts">
  // The chat header: the foreground chat's title and the Chat | Helpers
  // tablist (01b). The title is drawn whole and cut by CSS for the width it
  // has, so whoever asks for it - a hover, a screen reader - still reads the
  // session's full name. The artboard's "..." menu (Rename, Delete) is NOT
  // drawn: its actions are not built.
  import ViewTabs from './ViewTabs.svelte'
  import type { ViewTab } from './tabs'

  let {
    title,
    tab = 'chat',
    running = 0,
    onSelect = () => {},
  }: {
    title: string
    /** The active view, as the shell keeps it. */
    tab?: ViewTab
    /** How many helpers run right now, for the tab's badge. */
    running?: number
    /** The view the user asked for. */
    onSelect?: (tab: ViewTab) => void
  } = $props()
</script>

<header class="mh">
  <h1 title={title}>{title}</h1>
  <span class="sp"></span>
  <ViewTabs {tab} {running} {onSelect} />
</header>
