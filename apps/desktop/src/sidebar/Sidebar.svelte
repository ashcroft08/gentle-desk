<script lang="ts">
  // The sidebar: New session (D2), the New workspace icon (D7), the two plain
  // notice lines, the reading line of the first pass, and the workspace tree.
  // It owns the only view-local selection this slice needs (P5): the workspace
  // row the user clicked last, and the chat row clicked while Rust catches up.
  import { addWorkspace, dispatchIntent } from '../bridge'
  import { workspacePathOfChat, type ChatRow, type WorkspaceRow } from '../sessions/rows'
  import { createSessionsView } from '../sessions/view'
  import type { Viewport } from '../sessions/window'
  import EmptyState from './EmptyState.svelte'
  import NewSessionButton from './NewSessionButton.svelte'
  import SidebarHeader from './SidebarHeader.svelte'
  import SidebarNotices from './SidebarNotices.svelte'
  import WorkspaceTree from './WorkspaceTree.svelte'
  import {
    EMPTY_REASON,
    EMPTY_TITLE,
    EMPTY_WHY_ID,
    NEW_SESSION_WHY_ID,
    NO_WORKSPACE_REASON,
    readingLine,
  } from './copy'

  const sessions = createSessionsView()

  /** The workspace row the user clicked last. */
  let clickedPath = $state<string | null>(null)
  /** The chat row the user clicked, until a push names the foreground chat. */
  let clickedChat = $state<string | null>(null)

  // `model`, not `state`: a binding named `state` would shadow the `$state` rune.
  const model = $derived($sessions)
  const foreground = $derived(model.view?.foreground_chat ?? null)
  const selectedId = $derived(foreground ?? clickedChat)
  // P5: the workspace of the foreground chat when this page carries it, else the
  // workspace row the user clicked last.
  const currentPath = $derived(
    foreground === null ? clickedPath : (workspacePathOfChat(model.rows, foreground) ?? clickedPath),
  )
  // Only a settled list with no row at all may call itself empty: a workspace
  // with no chat counts its header row (S3), and a page still being read never
  // reaches this line.
  const empty = $derived(
    model.view?.list_status.type === 'Complete' && !model.loading && model.total === 0,
  )
  const status = $derived(model.view?.list_status ?? null)
  const reading = $derived(
    status?.type === 'Loading' ? readingLine(status.data.done, status.data.total) : null,
  )
  const reason = $derived(currentPath !== null || empty ? null : NO_WORKSPACE_REASON)
  const describedBy = $derived(
    currentPath !== null ? null : empty ? EMPTY_WHY_ID : NEW_SESSION_WHY_ID,
  )

  // One view, one store: closing the sidebar ends the event listener and drops
  // every cached page.
  $effect(() => () => sessions.dispose())

  function showPage(viewport: Viewport): void {
    sessions.show(viewport)
  }

  function toggle(row: WorkspaceRow): void {
    clickedPath = row.path
    void dispatchIntent({
      type: 'SetWorkspaceCollapsed',
      data: { path: row.path, collapsed: !row.collapsed },
    })
  }

  function open(row: ChatRow): void {
    clickedChat = row.id
    clickedPath = workspacePathOfChat(model.rows, row.id) ?? clickedPath
    void dispatchIntent({ type: 'OpenChat', data: { chat_id: row.id } })
  }

  function newChatAt(path: string): void {
    clickedPath = path
    void dispatchIntent({ type: 'NewChatInWorkspace', data: { path } })
  }

  function newSession(): void {
    if (currentPath !== null) newChatAt(currentPath)
  }
</script>

<aside class="side" aria-label="Workspaces">
  <NewSessionButton
    enabled={currentPath !== null}
    reason={reason}
    describedBy={describedBy}
    onNewSession={newSession}
  />
  <SidebarHeader onNewWorkspace={() => void addWorkspace()} />
  <SidebarNotices
    configNotice={model.view?.config_notice ?? null}
    workspaceNotice={model.view?.workspace_notice ?? null}
  />
  {#if reading !== null}
    <p class="gh" role="status">{reading}</p>
  {/if}
  {#if empty}
    <EmptyState title={EMPTY_TITLE} reason={EMPTY_REASON} reasonId={EMPTY_WHY_ID} />
  {:else}
    <WorkspaceTree
      rows={model.rows}
      total={model.total}
      first={model.first}
      selectedId={selectedId}
      activity={model.view?.activity ?? []}
      onViewport={showPage}
      onToggle={toggle}
      onNewChat={newChatAt}
      onOpen={open}
    />
  {/if}
</aside>
