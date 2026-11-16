<script lang="ts">
  // One workspace header row: the open or closed folder icon (no chevron, D6),
  // the label, `aria-expanded` for the group, the circle-plus new-chat icon (D3)
  // and, since S8, the row's own "..." menu. Both icons show while the workspace
  // section is hovered, or while either button holds keyboard focus, and they go
  // away otherwise: a click on one of them used to pin it open, because any focus
  // inside the row kept it drawn. A folder that is gone keeps it `aria-disabled`
  // with the reason as its description (states board, "a workspace whose folder is
  // gone"), and it loses nothing else.
  //
  // D5 kept this row menu-free until the removal feature: S8 is the maintainer's
  // decision to add the one menu, and its only item is the destructive one that
  // offers the workspace's removal. The row does not own that dialog - the sidebar
  // does, with the offer, the words and the command - so the item asks the
  // sidebar's removal host through the context this tree carries down.
  import { getContext } from 'svelte'
  import { DropdownMenu } from 'bits-ui'
  import Icon from '../icons/Icon.svelte'
  import type { WorkspaceRow } from '../sessions/rows'
  import { language } from '../i18n/language.svelte'
  import { sidebarCopy } from './copy'
  import { REMOVAL_HOST, type RemovalHost } from './removal'

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

  const reasonId = $derived(`ws-reason-${row.key.replace(/[^a-zA-Z0-9]+/g, '-')}`)
  const reason = $derived(row.available ? null : row.reason)
  /**
   * The icons' own words, in the language this run is set to (0.2.0, S3/D4). The
   * workspace label inside the names is Rust's and travels through untouched (D7).
   */
  const copy = $derived(sidebarCopy(language.current))
  const actions = $derived(copy.workspaceActions(row.label))
  /** The sidebar's own removal host, or nothing when this row is drawn alone. */
  const removal = getContext<RemovalHost | undefined>(REMOVAL_HOST)
  /** Whether the row's menu is open, so it keeps the slot it opened from. */
  let menuOpen = $state(false)

  function newChat(): void {
    // A folder that is not there cannot host a child: the icon says why instead.
    if (row.available) onNewChat(row.path)
  }
</script>

<div class="frow" class:menu-open={menuOpen}>
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
  <DropdownMenu.Root bind:open={menuOpen}>
    <DropdownMenu.Trigger
      class="ic nm tip-r"
      aria-label={actions}
      data-tip={copy.workspaceActionsTip}
      data-visible={revealed ? 'true' : 'false'}
      tabindex={active ? 0 : -1}
    >
      <Icon name="more" size={16} />
    </DropdownMenu.Trigger>
    <DropdownMenu.Portal>
      <!-- The card is `.menu`'s, shared with the chat row's menu: right-aligned
           under its button, inside the sidebar, away from its edge. -->
      <DropdownMenu.Content
        class="menu ws-menu"
        aria-label={actions}
        side="bottom"
        align="end"
        alignOffset={-10}
        sideOffset={6}
        collisionPadding={12}
      >
        <DropdownMenu.Item class="d" onSelect={() => removal?.askRemove(row.path)}>
          {copy.removeWorkspace}
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Portal>
  </DropdownMenu.Root>
  <button
    class="ic nc tip-r"
    type="button"
    aria-label={copy.newChatIn(row.label)}
    aria-disabled={row.available ? undefined : 'true'}
    aria-describedby={row.available ? undefined : reasonId}
    data-tip={copy.newChatTip}
    data-visible={revealed ? 'true' : 'false'}
    tabindex={active ? 0 : -1}
    onclick={newChat}
  >
    <Icon name="circle-plus" />
  </button>
</div>
