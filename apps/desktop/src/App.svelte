<script lang="ts">
  // The app shell: the workspace sidebar on the left and the main area on the
  // right. With a foreground chat the main area is that chat's chrome (T6: the
  // header, the restart notice), its thread (T5b) and its composer; with none it
  // stays the placeholder that names what to do next. The dialogs and the
  // helpers panel are later slices.
  import ChatHeader from './chat/Header.svelte'
  import { createChatView } from './chat/view'
  import Composer from './composer/Composer.svelte'
  import RestartNotice from './notices/RestartNotice.svelte'
  import { restartNoticesOf } from './notices/restart'
  import Sidebar from './sidebar/Sidebar.svelte'
  import Thread from './thread/Thread.svelte'
  import { createThreadView } from './thread/view'

  const thread = createThreadView()
  const chrome = createChatView()
  const model = $derived($thread)
  const chat = $derived($chrome)
  // The strip is drawn only when there is something in it: one row of the main
  // grid per element, so an empty strip would still take its gap under the
  // header.
  const restart = $derived(restartNoticesOf(chat.notices))

  // One app, one pair of stores: leaving the shell ends the event listeners.
  $effect(() => () => {
    thread.dispose()
    chrome.dispose()
  })
</script>

<a class="skip" href="#main">Skip to chat</a>
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
      {#if restart.length > 0}
        <div class="notices">
          <RestartNotice notices={chat.notices} />
        </div>
      {/if}
      <Thread messages={model.messages} foregroundChat={model.chat} />
      <Composer chat={model.chat} mark={chat.mark} />
    {/if}
  </main>
</div>
