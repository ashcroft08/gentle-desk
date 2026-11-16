<script lang="ts">
  // The chat header: the foreground chat's title, the pencil that renames it in
  // place and the Chat | Helpers tablist, pinned to the right edge (01a,
  // chat-chrome-helpers S2). The title is drawn whole and cut by CSS for the
  // width it has, so whoever asks for it - a hover, a screen reader - still reads
  // the session's full name. The artboard's "..." menu (Delete and the rest) is
  // NOT drawn: those actions live on the row, which keeps its own menu.
  //
  // The pencil opens the SAME inline rename the row's menu opens - the title
  // becomes a field holding it, Enter commits, Escape cancels, losing the focus
  // commits - and reports the committed name through `onRename`, which the shell
  // sends as the row's own rename intent. The house has no pencil glyph in
  // `icons/shapes.ts`, so the artboard's Lucide outline is inlined here.
  import SidebarToggle from './SidebarToggle.svelte'
  import ViewTabs from './ViewTabs.svelte'
  import { language } from '../i18n/language.svelte'
  import { chatCopy } from './copy'
  import type { ViewTab } from './tabs'

  let {
    title,
    tab = 'chat',
    running = 0,
    collapsed = false,
    onToggleSidebar = () => {},
    onSelect = () => {},
    onRename = () => {},
  }: {
    title: string
    /** The active view, as the shell keeps it. */
    tab?: ViewTab
    /** How many helpers run right now, for the tab's badge. */
    running?: number
    /** Whether the sidebar is collapsed, for the toggle's `aria-expanded`. */
    collapsed?: boolean
    onToggleSidebar?: () => void
    /** The view the user asked for. */
    onSelect?: (tab: ViewTab) => void
    /** The name the user typed for this chat, once the field commits. */
    onRename?: (name: string) => void
  } = $props()

  /** The header's own words in the language the app holds (S5). */
  const copy = $derived(chatCopy(language.current))

  /** Whether the title is a field right now. */
  let renaming = $state(false)
  /** What the field holds; the title until the user types. */
  let draft = $state('')
  /** The field, so opening a rename can focus it. */
  let field = $state<HTMLInputElement | null>(null)

  $effect(() => {
    if (renaming && field !== null) {
      field.focus()
      field.select()
    }
  })

  function startRename(): void {
    draft = title
    renaming = true
  }

  function cancel(): void {
    renaming = false
    draft = ''
  }

  function commit(): void {
    // A field that already closed (Escape, or the commit itself) sends nothing:
    // the blur that follows a commit is not a second one.
    if (!renaming) return
    const name = draft.trim()
    renaming = false
    // An unchanged name would only add a line the file does not need. Anything
    // else goes through, a blank name included: the reason it cannot be used is
    // the backend's to say, exactly as the row's rename has it.
    if (name !== title) onRename(name)
  }

  /** The field owns the keyboard while it is open. */
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

<header class="mh">
  <SidebarToggle {collapsed} onToggle={onToggleSidebar} />
  {#if renaming}
    <input
      class="rn"
      type="text"
      aria-label={copy.chatName}
      bind:value={draft}
      bind:this={field}
      onkeydown={typing}
      onblur={commit}
    />
  {:else}
    <h1 title={title}>{title}</h1>
    <button
      type="button"
      class="ic"
      aria-label={copy.renameChat}
      data-tip={copy.renameChat}
      onclick={startRename}
    >
      <!-- 01a's `i-pencil` outline, inlined because the house has no pencil. -->
      <svg class="pencil" viewBox="0 0 24 24" aria-hidden="true">
        <path
          d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"
        />
        <path d="m15 5 4 4" />
      </svg>
    </button>
  {/if}
  <span class="sp"></span>
  <ViewTabs {tab} {running} {onSelect} />
</header>

<style>
  .pencil {
    display: block;
    width: 16px;
    height: 16px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.8;
    stroke-linecap: round;
    stroke-linejoin: round;
  }
</style>
