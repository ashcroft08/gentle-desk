<script lang="ts">
  // View B's hero (welcome-screen T3, S1, S3): the rose, the session's title and
  // the folder it will run in - the three things sheet B of
  // design/welcome-screen/02-hero-composer.html draws over the centred composer.
  // It is the thread's place, not a screen of its own: the pane swaps THIS for the
  // thread while the chat still has no rows, and the composer below it never moves
  // out of the tree.
  //
  // No mode chip, no context rail and no "pick up with" line here, on purpose: the
  // mode is read from a process that does not exist yet (R2) and the recents are
  // not in the state while a chat is open (R4). The folder is the one real fact -
  // a new chat in a workspace names it from the start (composer-dock L12) - and a
  // chat with no folder yet says nothing instead of inventing a line.
  import { onMount } from 'svelte'
  import Rose from '../brand/Rose.svelte'
  import Icon from '../icons/Icon.svelte'
  import type { FolderView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { NEW_SESSION_ROSE_MAX, ROSE_FALLBACK, watchRoseHeight } from '../home/rose-height'
  import { chatCopy } from './copy'

  let { folder = null }: {
    /** The folder this chat runs in, or `null` while Rust has not named one. */
    folder?: FolderView | null
  } = $props()

  const copy = $derived(chatCopy(language.current))

  /**
   * The rose's height, which is the window's - the same steps view A takes, held
   * at 116 so the picture never crowds the composer that shares this pane.
   */
  let rosePx = $state(Math.min(ROSE_FALLBACK, NEW_SESSION_ROSE_MAX))
  onMount(() => watchRoseHeight((px) => (rosePx = Math.min(px, NEW_SESSION_ROSE_MAX))))
</script>

<div class="new-session">
  <div class="new-session-rose">
    <Rose label={copy.roseLabel} height={rosePx} />
  </div>
  <h2 class="new-session-title">{copy.newSessionTitle}</h2>
  {#if folder !== null}
    <!-- The one line of real data: the folder, as the composer's chip names it. -->
    <p class="new-session-where">
      <Icon name="folder" size={14} />
      <span>{copy.inFolder(folder.name)}</span>
    </p>
  {/if}
</div>
