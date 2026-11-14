<script lang="ts">
  // The workspace tree: one virtualized list over the flattened rows Rust serves
  // (headers and chats share `ROW_HEIGHT`, so the tree is one list, S3). Only the
  // served page is drawn; the padding above and below it measures the rest of
  // the tree so the scrollbar stays honest with 50 000 chats.
  //
  // The keyboard is the tree pattern: one roving tab stop, arrow keys move it,
  // Home and End jump to the ends, Enter and Space activate the focused row.
  // The rows themselves carry no key handler, so the roles and markup stay the
  // artboard's (01a).
  import type { ChatActivityView } from '../contract/bindings'
  import {
    isWorkspaceRow,
    markKindOf,
    rowWorkspaceKeys,
    ROW_HEIGHT,
    type ChatRow as ChatRowData,
    type TreeRow,
    type WorkspaceRow,
  } from '../sessions/rows'
  import type { Viewport } from '../sessions/window'
  import ChatRow from './ChatRow.svelte'
  import WorkspaceHeaderRow from './WorkspaceHeaderRow.svelte'

  let {
    rows = [],
    total = 0,
    first = 0,
    selectedId = null,
    activity = [],
    onViewport,
    onToggle,
    onNewChat,
    onOpen,
    onRename,
    onClose,
    onDelete,
  }: {
    rows?: TreeRow[]
    /** How many rows the whole tree has, headers included. */
    total?: number
    /** Index of the first row of the served page. */
    first?: number
    /** The foreground chat, drawn selected. */
    selectedId?: string | null
    activity?: ChatActivityView[]
    onViewport: (viewport: Viewport) => void
    onToggle: (row: WorkspaceRow) => void
    onNewChat: (path: string) => void
    onOpen: (row: ChatRowData) => void
    /** The name a chat row's inline input committed, to send as the rename. */
    onRename: (row: ChatRowData, name: string) => void
    /** A chat row's Close chat: stop its child, leave its row and its file. */
    onClose: (row: ChatRowData) => void
    /** A chat row's Delete: the sidebar opens the confirm dialog that names it. */
    onDelete: (row: ChatRowData) => void
  } = $props()

  /** The scroll container: it is what the viewport is measured from. */
  let nav: HTMLElement | null = $state(null)
  /** The tree itself: the element the rows are read from and keys are handled on. */
  let tree: HTMLElement | null = $state(null)
  /** The row that holds the one tab stop. */
  let activeIndex = $state(0)
  /** The workspace section the pointer is over, so its icon can show (D3). */
  let hoveredKey = $state<string | null>(null)

  const keys = $derived(rowWorkspaceKeys(rows))
  const marks = $derived(
    new Map(activity.map((entry) => [entry.chat_id, markKindOf(entry.activity)])),
  )
  const active = $derived(rows.length === 0 ? 0 : Math.min(activeIndex, rows.length - 1))
  const padTop = $derived(Math.max(0, first) * ROW_HEIGHT)
  const padBottom = $derived(Math.max(0, total - first - rows.length) * ROW_HEIGHT)

  function rowKey(row: TreeRow): string {
    return isWorkspaceRow(row) ? `w:${row.key}` : `c:${row.id}`
  }

  function viewportNow(): Viewport {
    const top = nav === null ? 0 : nav.scrollTop
    const height = nav === null ? 0 : nav.clientHeight
    return {
      first: Math.max(0, Math.floor(top / ROW_HEIGHT)),
      visible: Math.max(0, Math.ceil(height / ROW_HEIGHT)),
    }
  }

  // The page is read once the container exists and again on every scroll;
  // window.ts widens each request by its overscan.
  $effect(() => {
    if (nav !== null) onViewport(viewportNow())
  })

  function scrolled(): void {
    onViewport(viewportNow())
  }

  function items(): HTMLElement[] {
    if (tree === null) return []
    return [...tree.querySelectorAll<HTMLElement>('[role="treeitem"]')]
  }

  /** A row that takes focus, by click or by key, is the tree's tab stop. */
  function focusMoved(event: FocusEvent): void {
    const found = event.target instanceof Element ? event.target.closest('[role="treeitem"]') : null
    if (found instanceof HTMLElement && found.dataset.row !== undefined) {
      activeIndex = Number(found.dataset.row)
    }
  }

  function focusAt(target: HTMLElement): void {
    const place = items().indexOf(target)
    if (place >= 0) activeIndex = place
    target.focus()
  }

  function activate(target: HTMLElement): void {
    const row = rows[Number(target.dataset.row)]
    if (row === undefined) return
    if (isWorkspaceRow(row)) onToggle(row)
    else onOpen(row)
  }

  function keydown(event: KeyboardEvent): void {
    const found = event.target instanceof Element ? event.target.closest('[role="treeitem"]') : null
    if (!(found instanceof HTMLElement)) return
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      activate(found)
      return
    }
    const all = items()
    const place = all.indexOf(found)
    let next: HTMLElement | undefined
    if (event.key === 'ArrowDown') next = all[Math.min(place + 1, all.length - 1)]
    else if (event.key === 'ArrowUp') next = all[Math.max(place - 1, 0)]
    else if (event.key === 'Home') next = all[0]
    else if (event.key === 'End') next = all[all.length - 1]
    else return
    if (next === undefined) return
    event.preventDefault()
    focusAt(next)
  }
</script>

<nav class="tree" aria-label="Workspaces" bind:this={nav} onscroll={scrolled}>
  <div
    role="tree"
    aria-label="Workspaces"
    tabindex="-1"
    bind:this={tree}
    onfocusin={focusMoved}
    onmouseleave={() => (hoveredKey = null)}
    style:padding-top={`${padTop}px`}
    style:padding-bottom={`${padBottom}px`}
  >
    {#each rows as row, index (rowKey(row))}
      {#if isWorkspaceRow(row)}
        <WorkspaceHeaderRow
          row={row}
          index={index}
          active={index === active}
          revealed={hoveredKey === row.key}
          onToggle={onToggle}
          onNewChat={onNewChat}
          onHover={(key) => (hoveredKey = key)}
          onKeydown={keydown}
        />
      {:else}
        <ChatRow
          row={row}
          index={index}
          workspaceKey={keys[index] ?? null}
          active={index === active}
          selected={row.id === selectedId}
          mark={marks.get(row.id) ?? null}
          onOpen={onOpen}
          onHover={(key) => (hoveredKey = key)}
          onKeydown={keydown}
          onRename={onRename}
          onClose={onClose}
          onDelete={onDelete}
        />
      {/if}
    {/each}
  </div>
</nav>
