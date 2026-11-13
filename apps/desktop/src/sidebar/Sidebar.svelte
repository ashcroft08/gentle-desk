<script lang="ts">
  // The sidebar: New session (D2), the New workspace control (D7), the two plain
  // notice lines, the reading line of the first pass, the workspace tree, and the
  // footer with the Preview chip and the gear that opens Settings
  // (settings-dialog S1: the theme picker and the home setting live there now).
  // It owns the only view-local selection this slice needs (P5): the workspace
  // row the user clicked last, and the chat row clicked while Rust catches up.
  import { dispatchIntent, renameSession, deleteSession } from '../bridge'
  import { workspacePathOfChat, type ChatRow, type WorkspaceRow } from '../sessions/rows'
  import { createSessionsView } from '../sessions/view'
  import type { Viewport } from '../sessions/window'
  import WordMark from '../brand/WordMark.svelte'
  import DeleteChatDialog from './DeleteChatDialog.svelte'
  import EmptyState from './EmptyState.svelte'
  import NewSessionButton from './NewSessionButton.svelte'
  import SidebarHeader from './SidebarHeader.svelte'
  import SidebarNotices from './SidebarNotices.svelte'
  import WorkspaceTree from './WorkspaceTree.svelte'
  import Icon from '../icons/Icon.svelte'
  import SettingsDialog from '../settings/SettingsDialog.svelte'
  import { SETTINGS } from '../settings/copy'
  import {
    EMPTY_REASON,
    EMPTY_TITLE,
    EMPTY_WHY_ID,
    NEW_SESSION_WHY_ID,
    NO_WORKSPACE_REASON,
    readingLine,
  } from './copy'

  const sessions = createSessionsView()

  /** Whether the header's New workspace menu is open (the empty prompt opens it). */
  let newWorkspaceOpen = $state(false)
  /** The workspace row the user clicked last. */
  let clickedPath = $state<string | null>(null)
  /** The chat row the user clicked, until a push names the foreground chat. */
  let clickedChat = $state<string | null>(null)
  /** Whether the delete confirm dialog is open. */
  let deleteOpen = $state(false)
  /** Whether the Settings dialog is open (settings-dialog S1-S6). */
  let settingsOpen = $state(false)
  /** The chat row that dialog is about: the one whose Delete the user picked. */
  let deleting = $state<ChatRow | null>(null)

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

  /**
   * The name a row's inline input committed (T2): Rust appends it to the chat's
   * session file and asks for the pass that reads it back, so the row and the
   * header carry the new title on the next push. A refusal reaches the user as
   * `sessions_notice`, drawn under the two lines above the tree.
   */
  function rename(row: ChatRow, name: string): void {
    clickedChat = row.id
    void renameSession(row.id, name)
  }

  /**
   * A row's Delete (T3): the confirm dialog opens on that chat, so the user reads
   * the real file before anything is removed. Nothing is sent until Delete in the
   * dialog is pressed.
   */
  function askDelete(row: ChatRow): void {
    deleting = row
    deleteOpen = true
  }

  /**
   * The dialog's Delete: Rust removes that one file and asks for the pass that
   * heals the list. A refusal (a chat that is working asks to be stopped first)
   * reaches the user as `sessions_notice`, drawn under the lines above the tree.
   */
  async function confirmDelete(path: string): Promise<void> {
    await deleteSession(path)
  }

  function newSession(): void {
    if (currentPath !== null) newChatAt(currentPath)
  }
</script>

<aside class="side" aria-label="Workspaces">
  <WordMark />
  <NewSessionButton
    enabled={currentPath !== null}
    reason={reason}
    describedBy={describedBy}
    onNewSession={newSession}
  />
  <SidebarHeader bind:menuOpen={newWorkspaceOpen} />
  <SidebarNotices
    configNotice={model.view?.config_notice ?? null}
    workspaceNotice={model.view?.workspace_notice ?? null}
    sessionsNotice={model.view?.sessions_notice ?? null}
  />
  {#if reading !== null}
    <p class="gh" role="status">{reading}</p>
  {/if}
  {#if empty}
    <EmptyState
      title={EMPTY_TITLE}
      reason={EMPTY_REASON}
      reasonId={EMPTY_WHY_ID}
      onNewWorkspace={() => (newWorkspaceOpen = true)}
    />
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
      onRename={rename}
      onDelete={askDelete}
    />
  {/if}
  <!-- The delete's confirm dialog, over everything: it names the real file of
       the chat the user picked, and only its Delete sends the command (S4). -->
  <DeleteChatDialog
    bind:open={deleteOpen}
    path={deleting?.id ?? ''}
    onDelete={confirmDelete}
  />
  <!-- The Settings dialog, the only home the theme picker and the home setting
       have now (S1), and where the app's name and running version live (S5).
       The three 0.1.12 panes read their own slices from the same push (S1-S4).
       Closing it sends nothing: each section keeps its own path. -->
  <SettingsDialog
    bind:open={settingsOpen}
    theme={model.view?.theme ?? 'system'}
    shell={model.view?.shell ?? null}
    version={model.view?.app_version ?? null}
    profiles={model.view?.profiles ?? null}
    auth={model.view?.auth ?? null}
    updates={model.view?.updates ?? null}
    openChats={model.view?.open_chats ?? 0}
  />
  <!-- The footer keeps exactly two things (S1): the run's Preview chip and the
       gear that opens Settings. -->
  <footer class="foot">
    <span
      class="preview-chip"
      title="Preview: under active development - fixes ship fast"
      >Preview</span
    >
    <button
      class="ic"
      type="button"
      aria-label={SETTINGS}
      title={SETTINGS}
      onclick={() => (settingsOpen = true)}
    >
      <Icon name="settings" />
    </button>
  </footer>
</aside>
