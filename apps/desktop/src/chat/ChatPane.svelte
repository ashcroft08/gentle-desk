<script lang="ts">
  // The chat pane: ONE tree for both faces of a chat (welcome-screen T3, S1, S3,
  // R1). With rows it is the thread and the composer at the bottom, as it always
  // was; with none, in a NEW chat, it is the hero in the thread's place and the
  // composer centred as the protagonist. Only the class on the pane changes: the
  // thread keeps its element (its mount, its place in the transcript) and the
  // composer is the very same one - never a second box, never a re-mount - so the
  // draft, the focus and the attached chips cross over untouched.
  //
  // The move is a FLIP. The dock's top is measured on the frame BEFORE the class
  // leaves (`$effect.pre`: the only moment the old place exists), the inverse
  // translate is applied, and one frame later that transform goes to 0 through a
  // 240 ms ease-out transition - sheet B2 of design/welcome-screen/
  // 02-hero-composer.html. It is skipped entirely under reduced motion (R6) and on
  // a first paint, where there is no earlier place to come from, and nothing keeps
  // running once it arrives: the one frame it asked for is cancelled on the way
  // out and the listener that tidies the element fires once.
  //
  // This pane is also where a NEW chat's process is asked for (welcome-screen T4,
  // R2): opening a session allocates the chat and starts nothing, and the ask
  // travels from here - see [`ensureStarted`] - so the seconds a child takes to
  // come up are spent when the box is in front of the user, never while the app
  // opens.
  import { dispatchIntent } from '../bridge'
  import Composer from '../composer/Composer.svelte'
  import Thread from '../thread/Thread.svelte'
  import type { ThreadViewState } from '../thread/view'
  import NewSession from './NewSession.svelte'
  import { DOCK_MOVE_MS, flipOffset, isFreshChat, isNewSession } from './new-session'
  import type { ChatViewState } from './view'

  let {
    chat,
    thread,
    visible = true,
    scrollTop = $bindable(0),
  }: {
    /** This chat's chrome: the mark the composer reads, the folder view B names. */
    chat: ChatViewState
    /** The foreground chat and its rows: the pane's other half of "is it new". */
    thread: ThreadViewState
    /** Whether the Chat tab is the one drawn (Helpers and Profile share the space). */
    visible?: boolean
    /** Where the reader was in the thread, bound so a tab switch keeps it. */
    scrollTop?: number
  } = $props()

  /** View B is this chat with no row at all yet (welcome-screen T3). */
  const fresh = $derived(isNewSession(chat.chat, thread.messages, thread.items))

  /**
   * The chats whose child this pane already asked Rust to start (welcome-screen
   * T4, R2), kept for as long as the pane is on screen: the ask is per chat and
   * it happens once, whatever brings the same chat back through this render.
   */
  const startedFor = new Set<string>()

  /**
   * The lazy start of a new chat (welcome-screen T4, R2). The click that opens a
   * new session ALLOCATES the chat and starts nothing; its process is asked for
   * when the box it is typed into comes to the front - which is this very
   * render, because the composer puts the cursor in that box on every chat
   * change (`Composer`'s focus effect) - or when the first key lands in it,
   * whichever comes first.
   *
   * Only a NEW chat asks. A stored chat started its child when it was opened, so
   * it never needs this, and that is also what keeps a reopen of the chat on
   * screen from asking twice: Rust's own rule answers a second ask for a chat it
   * already started with nothing.
   */
  function ensureStarted(): void {
    const open = chat.chat
    if (open === null || !isFreshChat(open) || startedFor.has(open)) return
    startedFor.add(open)
    void dispatchIntent({ type: 'EnsureStarted', data: { chat_id: open } })
  }

  // The chat that arrives here is the chat being looked at, and its box is about
  // to take the cursor: that is the first of the two triggers. The second - the
  // first key landing in the box - is the dock's own `oninput` below, and the set
  // above is what makes the two of them one ask.
  $effect(() => {
    // Reading the chat is what makes this follow the change, and only it: a push
    // that names the same chat asks nothing new.
    void chat.chat
    ensureStarted()
  })

  /** The dock that travels, and the frame it is waiting for. */
  let dock = $state<HTMLElement | null>(null)
  let waiting = 0
  /** The state the pane was last drawn in, and the dock's top on the way out. */
  let wasFresh: boolean | null = null
  let fromTop: number | null = null

  // Measured BEFORE the class moves: the pane is still in the state it leaves, so
  // this is the one moment the dock's old place can be read.
  $effect.pre(() => {
    const now = fresh
    if (wasFresh === now) return
    const leaving = wasFresh === true && !now
    wasFresh = now
    fromTop = leaving ? (dock?.getBoundingClientRect().top ?? null) : null
  })

  // And applied after it moved, when the dock's new place is the real one. A first
  // paint has no offset to apply, and arriving in view B moves nothing either.
  $effect(() => {
    // Reading it is what makes this follow the class; the offset is a plain field,
    // written by the effect above and taken here.
    if (fresh) return
    const from = fromTop
    if (from === null) return
    fromTop = null
    move(from)
  })

  /** Puts the dock back where it was, then lets it travel to where it is now. */
  function move(from: number): void {
    const element = dock
    if (element === null) return
    cancelAnimationFrame(waiting)
    element.style.transition = ''
    element.style.transform = ''
    // R6: with no motion asked for, the composer is simply where it belongs.
    if (reducedMotion()) return
    const offset = flipOffset(from, element.getBoundingClientRect().top)
    if (offset === null || offset === 0) return
    element.style.transform = `translateY(${offset}px)`
    // One frame AT the old place, so the browser paints it there before it can
    // travel; then the transition carries it home.
    waiting = requestAnimationFrame(() => {
      element.style.transition = `transform ${DOCK_MOVE_MS}ms ease-out`
      element.style.transform = ''
      element.addEventListener('transitionend', tidy, { once: true })
    })
  }

  /** Nothing of the move stays on the element once it arrives. */
  function tidy(event: Event): void {
    const element = event.currentTarget as HTMLElement
    element.style.transition = ''
  }

  /** Whether this window asked for no animation at all. */
  function reducedMotion(): boolean {
    return (
      typeof globalThis.matchMedia === 'function' &&
      globalThis.matchMedia('(prefers-reduced-motion: reduce)').matches
    )
  }

  // A pane that leaves mid-move asks for no frame of its own.
  $effect(() => () => cancelAnimationFrame(waiting))
</script>

<div class="chat-pane" class:is-fresh={fresh}>
  {#if fresh}
    <!-- The thread's place, while it has nothing to hold. -->
    <NewSession />
  {/if}
  <div class="chat-pane-body">
    <Thread
      messages={thread.messages}
      items={thread.items}
      foregroundChat={thread.chat}
      mark={chat.mark}
      sessionMode={chat.sessionMode}
      provisional={thread.provisional}
      {visible}
      bind:scrollTop
    />
  </div>
  <!-- The dock is also where the second trigger of a new chat's start lives
       (welcome-screen T4): an `input` event bubbles out of the box, so the first
       key typed in it asks Rust for the child. -->
  <div class="chat-pane-dock" bind:this={dock} oninput={ensureStarted}>
    <Composer
      chat={chat.chat}
      mark={chat.mark}
      images={chat.images}
      imagesNotice={chat.imagesNotice}
      chip={chat.modelChip}
      contextUsage={chat.contextUsage}
      folder={chat.folder}
      branch={chat.branch}
    />
  </div>
</div>
