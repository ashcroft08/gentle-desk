<script lang="ts">
  // The app shell: the workspace sidebar on the left and the main area on the
  // right. With a foreground chat the main area is that chat's thread (T5b);
  // with none it stays the placeholder that names what to do next. The header
  // of an open chat, the composer and the dialogs are later slices.
  import Sidebar from './sidebar/Sidebar.svelte'
  import Thread from './thread/Thread.svelte'
  import { createThreadView } from './thread/view'

  const thread = createThreadView()
  const model = $derived($thread)

  // One view, one store: leaving the shell ends the event listener.
  $effect(() => () => thread.dispose())
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
      <Thread messages={model.messages} foregroundChat={model.chat} />
    {/if}
  </main>
</div>
