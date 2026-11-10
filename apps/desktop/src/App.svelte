<script lang="ts">
  // The app shell: the workspace sidebar on the left and the main area on the
  // right. With a foreground chat the main area is that chat's chrome (T6: the
  // header, its notices), its thread (T5b) and its composer; with none it stays
  // the placeholder that names what to do next. Over all of it sit the two T7a
  // layers: the fatal launcher bar at the top, which belongs to no chat, and the
  // blocking extension dialog as a modal. The helpers panel and their thread are
  // the next slice.
  import ChatHeader from './chat/Header.svelte'
  import FatalBanner from './chat/FatalBanner.svelte'
  import { createChatView } from './chat/view'
  import Composer from './composer/Composer.svelte'
  import Dialogs from './dialogs/Dialogs.svelte'
  import Notices from './notices/Notices.svelte'
  import Sidebar from './sidebar/Sidebar.svelte'
  import Thread from './thread/Thread.svelte'
  import { createThreadView } from './thread/view'

  const thread = createThreadView()
  const chrome = createChatView()
  const model = $derived($thread)
  const chat = $derived($chrome)

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
      <ChatHeader title={chat.title ?? ''} />
      {#if chat.notices.length > 0}
        <div class="notices">
          <Notices notices={chat.notices} />
        </div>
      {/if}
      <Thread messages={model.messages} foregroundChat={model.chat} />
      <Composer chat={model.chat} mark={chat.mark} />
    {/if}
  </main>
</div>
<Dialogs dialog={chat.dialog} />
