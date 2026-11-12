<script lang="ts">
  // One chat row of a workspace: its title, how long ago it was last written,
  // and the mark its activity calls for (reusing `Mark`, so the state is always
  // a shape and a name, never colour alone). Selected means foreground.
  //
  // The row's own "..." menu (T2, S4) offers Rename and Delete. Rename turns the
  // title into an input, like the states board's inline rename: Enter commits,
  // Escape cancels, and the typed name goes to Rust as the rename command (the
  // title the state carries moves on the next push, when the session list reads
  // the appended name back). Delete opens the sidebar's confirm dialog, which
  // names the real file before the delete command runs (T3); the item stays
  // enabled always, because only the chat's WORKING state refuses a delete, and
  // that refusal is the command's to say.
  import { DropdownMenu } from 'bits-ui'
  import Icon from '../icons/Icon.svelte'
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
    onRename,
    onDelete,
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
    /** The name the user typed for this chat, once they pressed Enter. */
    onRename: (row: ChatRow, name: string) => void
    /** This chat's Delete: the sidebar opens the confirm dialog that names it. */
    onDelete: (row: ChatRow) => void
  } = $props()

  /** Whether the title is an input right now. */
  let renaming = $state(false)
  /** What the input holds; the row's title until the user types. */
  let draft = $state('')
  /** The input, so starting a rename can focus it. */
  let field = $state<HTMLInputElement | null>(null)
  /** Whether the actions menu is open, so its button keeps the slot it holds. */
  let menuOpen = $state(false)

  const actions = $derived(`Chat actions for ${row.title}`)

  $effect(() => {
    if (renaming && field !== null) {
      field.focus()
      field.select()
    }
  })

  function startRename(): void {
    draft = row.title
    renaming = true
  }

  function cancel(): void {
    renaming = false
    draft = ''
  }

  function commit(): void {
    const name = draft.trim()
    renaming = false
    // An unchanged name would only add a line the file does not need. Anything
    // else goes to Rust, a blank name included: the reason it cannot be used is
    // the backend's to say, and it reaches the user as the session notice.
    if (name !== row.title) onRename(row, name)
  }

  /**
   * The input owns the keyboard while it is open: the tree's handler would
   * otherwise read the arrows as row moves and Enter as "open this chat".
   */
  function typing(event: KeyboardEvent): void {
    event.stopPropagation()
    if (event.key === 'Enter') {
      event.preventDefault()
      commit()
    } else if (event.key === 'Escape') {
      event.preventDefault()
      cancel()
    }
  }
</script>

<div class="frow chat-row" class:menu-open={menuOpen}>
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
    {#if renaming}
      <input
        class="rn"
        type="text"
        aria-label="Chat name"
        bind:value={draft}
        bind:this={field}
        onkeydown={typing}
        onclick={(event) => event.stopPropagation()}
        onblur={cancel}
      />
    {:else}
      <span class="t">{row.title}</span>
    {/if}
    <span class="ago">{row.modified_ago}</span>
    {#if mark !== null}
      <Mark kind={mark} />
    {/if}
  </div>
  <DropdownMenu.Root bind:open={menuOpen}>
    <DropdownMenu.Trigger
      class="ic nc tip-r"
      aria-label={actions}
      data-tip="Chat actions"
      tabindex={active ? 0 : -1}
    >
      <Icon name="more" size={16} />
    </DropdownMenu.Trigger>
    <DropdownMenu.Portal>
      <!-- The menu opens inside the sidebar and away from its edge: right-aligned
           to this button, pulled a little further in, and kept off the window by
           the collision padding. The card (padding, radius, shadow, item hit
           areas) is `.menu`'s, shared with the New workspace menu. -->
      <DropdownMenu.Content
        class="menu chat-menu"
        aria-label={actions}
        side="bottom"
        align="end"
        alignOffset={-10}
        sideOffset={6}
        collisionPadding={12}
      >
        <DropdownMenu.Item onSelect={startRename}>Rename</DropdownMenu.Item>
        <DropdownMenu.Item class="d" onSelect={() => onDelete(row)}>Delete</DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Portal>
  </DropdownMenu.Root>
</div>
