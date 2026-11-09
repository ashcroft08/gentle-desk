<script lang="ts">
  // One chat row of a workspace: its title, how long ago it was last written,
  // and the mark its activity calls for (reusing `Mark`, so the state is always
  // a shape and a name, never colour alone). Selected means foreground.
  import Mark from '../icons/Mark.svelte'
  import type { MarkKind } from '../icons/shapes'
  import type { ChatRow } from '../sessions/rows'

  let {
    row,
    index,
    workspaceKey = null,
    active = false,
    selected = false,
    mark = null,
    onOpen,
    onHover,
    onKeydown,
  }: {
    row: ChatRow
    /** The row's place in the drawn page, the key the tree uses to activate it. */
    index: number
    /** The workspace this chat sits under, so hovering it reveals that icon. */
    workspaceKey?: string | null
    /** Whether this row holds the tree's one roving tab stop. */
    active?: boolean
    selected?: boolean
    mark?: MarkKind | null
    onOpen: (row: ChatRow) => void
    onHover: (key: string | null) => void
    /** The tree's own key handler: one roving tab stop, arrow keys, Enter, Space. */
    onKeydown: (event: KeyboardEvent) => void
  } = $props()
</script>

<div
  class="row chat"
  role="treeitem"
  aria-level="2"
  aria-selected={selected}
  tabindex={active ? 0 : -1}
  data-row={index}
  onmouseenter={() => onHover(workspaceKey)}
  onclick={() => onOpen(row)}
  onkeydown={onKeydown}
>
  <span class="t">{row.title}</span>
  <span class="ago">{row.modified_ago}</span>
  {#if mark !== null}
    <Mark kind={mark} />
  {/if}
</div>
