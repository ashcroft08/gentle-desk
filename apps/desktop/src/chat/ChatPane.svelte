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
  import Composer from '../composer/Composer.svelte'
  import Thread from '../thread/Thread.svelte'
  import type { ThreadViewState } from '../thread/view'
  import NewSession from './NewSession.svelte'
  import { DOCK_MOVE_MS, flipOffset, isNewSession } from './new-session'
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
  <div class="chat-pane-dock" bind:this={dock}>
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
