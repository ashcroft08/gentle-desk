<script lang="ts">
  // The app shell: the workspace sidebar on the left and the main area on the
  // right. The chat view, composer and thread are the next slices (T5/T6), so
  // the main area is a placeholder that names what to do next.
  import Sidebar from './sidebar/Sidebar.svelte'
</script>

<a class="skip" href="#main">Skip to chat</a>
<div class="app">
  <Sidebar />
  <main id="main" class="main">
    <header class="mh">
      <h1>No chat open</h1>
    </header>
    <p class="placeholder">Choose a chat in the workspace tree, or start a new session.</p>
  </main>
</div>
