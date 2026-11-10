<script lang="ts">
  // The app shell: the workspace sidebar on the left and the main area on the
  // right. With a foreground chat the main area is that chat's chrome (T6: the
  // header with its Chat | Helpers tabs, its notices), then the view the tab
  // names: the thread and its composer (T5b), or the helpers panel (T7b-1).
  // With none it stays the placeholder that names what to do next. Over all of
  // it sit the two T7a layers: the fatal launcher bar at the top, which belongs
  // to no chat, and the blocking extension dialog as a modal. The Helpers view
  // draws the list or, once a card is selected, that helper's own thread
  // (T7b-2).
  import ChatHeader from './chat/Header.svelte'
  import FatalBanner from './chat/FatalBanner.svelte'
  import type { ViewTab } from './chat/tabs'
  import { createChatView } from './chat/view'
  import Composer from './composer/Composer.svelte'
  import Dialogs from './dialogs/Dialogs.svelte'
  import HelpersPanel from './helpers/HelpersPanel.svelte'
  import Notices from './notices/Notices.svelte'
  import Sidebar from './sidebar/Sidebar.svelte'
  import Thread from './thread/Thread.svelte'
  import { createThreadView } from './thread/view'

  const thread = createThreadView()
  const chrome = createChatView()
  const model = $derived($thread)
  const chat = $derived($chrome)

  // The active view is the shell's own choice, never Rust's state: it survives a
  // push and a chat switch, and the next start begins on Chat.
  let tab = $state<ViewTab>('chat')
  // Where the reader was in the thread. The thread stays mounted while the
  // Helpers view covers it, so this is what puts them back in the same place.
  let threadScroll = $state(0)

  // One app, one pair of stores: leaving the shell ends the event listeners.
  $effect(() => () => {
    thread.dispose()
    chrome.dispose()
  })
</script>

<a class="skip" href="#main">Skip to chat</a>
{#if chat.fatal !== null}
  <FatalBanner text={chat.fatal} />
{/if}
<div class="app">
  <Sidebar />
  <main id="main" class="main">
    {#if model.chat === null}
      <header class="mh">
        <h1>No chat open</h1>
      </header>
      <p class="placeholder">Choose a chat in the workspace tree, or start a new session.</p>
    {:else}
      <ChatHeader
        title={chat.title ?? ''}
        {tab}
        running={chat.helpers?.running ?? 0}
        onSelect={(next) => (tab = next)}
      />
      {#if chat.notices.length > 0}
        <div class="notices">
          <Notices notices={chat.notices} />
        </div>
      {/if}
      <!-- Both views stay mounted, one `hidden`, as 01b draws them: the composer
           keeps its draft and the thread keeps its place across a switch. -->
      <div class="views">
        <div
          class="view chat"
          id="p-chat"
          role="tabpanel"
          aria-labelledby="t-chat"
          hidden={tab !== 'chat'}
        >
          <Thread
            messages={model.messages}
            foregroundChat={model.chat}
            visible={tab === 'chat'}
            bind:scrollTop={threadScroll}
          />
          <Composer chat={model.chat} mark={chat.mark} />
        </div>
        <div class="view" id="p-helpers" role="tabpanel" aria-labelledby="t-help" hidden={tab !== 'helpers'}>
          {#if chat.helpers !== null}
            <HelpersPanel
              helpers={chat.helpers}
              selected={chat.selectedHelper}
              thread={chat.helperThread}
            />
          {/if}
        </div>
      </div>
    {/if}
  </main>
</div>
<Dialogs dialog={chat.dialog} />
