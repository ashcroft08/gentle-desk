<script lang="ts">
  // One workspace header row: the open or closed folder icon (no chevron, D6),
  // the label, `aria-expanded` for the group, and the circle-plus new-chat icon
  // (D3). The icon shows while any row of this workspace is hovered or while
  // the row holds focus, and a folder that is gone keeps it `aria-disabled` with
  // the reason as its description (states board, "a workspace whose folder is
  // gone"). There is no "..." menu on the row (D5).
  import Icon from '../icons/Icon.svelte'
  import type { WorkspaceRow } from '../sessions/rows'

  let {
    row,
    index,
    active = false,
    revealed = false,
    onToggle,
    onNewChat,
    onHover,
    onKeydown,
  }: {
    row: WorkspaceRow
    /** The row's place in the drawn page, the key the tree uses to activate it. */
    index: number
    /** Whether this row holds the tree's one roving tab stop. */
    active?: boolean
    /** Whether the workspace section is hovered at the moment. */
    revealed?: boolean
    onToggle: (row: WorkspaceRow) => void
    onNewChat: (path: string) => void
    onHover: (key: string) => void
    /** The tree's own key handler: one roving tab stop, arrow keys, Enter, Space. */
    onKeydown: (event: KeyboardEvent) => void
  } = $props()

  let focused = $state(false)

  const reasonId = $derived(`ws-reason-${row.key.replace(/[^a-zA-Z0-9]+/g, '-')}`)
  const visible = $derived(revealed || focused)
  const reason = $derived(row.available ? null : row.reason)

  function newChat(): void {
    // A folder that is not there cannot host a child: the icon says why instead.
    if (row.available) onNewChat(row.path)
  }
</script>

<div class="frow" onfocusin={() => (focused = true)} onfocusout={() => (focused = false)}>
  <div
    class="row folder"
    role="treeitem"
    aria-level="1"
    aria-selected="false"
    aria-expanded={!row.collapsed}
    tabindex={active ? 0 : -1}
    data-row={index}
    onmouseenter={() => onHover(row.key)}
    onclick={() => onToggle(row)}
    onkeydown={onKeydown}
  >
    <Icon name={row.collapsed ? 'folder' : 'folder-open'} size={16} />
    <span class="t">
      {row.label}
      {#if reason !== null}
        <span class="sub" id={reasonId}>{reason}</span>
      {/if}
    </span>
  </div>
  <button
    class="ic nc tip-r"
    type="button"
    aria-label={`New chat in ${row.label}`}
    aria-disabled={row.available ? undefined : 'true'}
    aria-describedby={row.available ? undefined : reasonId}
    data-tip="New chat"
    data-visible={visible ? 'true' : 'false'}
    tabindex={active ? 0 : -1}
    onclick={newChat}
  >
    <Icon name="circle-plus" />
  </button>
</div>
