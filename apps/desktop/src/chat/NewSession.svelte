<script lang="ts">
  // View B's hero (welcome-screen T3, S1, S3): the rose, the session's title and
  // the folder it will run in - the two things sheet B of
  // design/welcome-screen/02-hero-composer.html draws over the centred composer.
  // It is the thread's place, not a screen of its own: the pane swaps THIS for the
  // thread while the chat still has no rows, and the composer below it never moves
  // out of the tree.
  //
  // No mode chip, no context rail and no "pick up with" line here, on purpose: the
  // mode is read from a process that does not exist yet (R2) and the recents are
  // not in the state while a chat is open (R4). The composer's own row already
  // names it, so the hero carries nothing that repeats it (the maintainer called
  // the folder line redundant).
  import { onMount } from 'svelte'
  import Rose from '../brand/Rose.svelte'
  import { language } from '../i18n/language.svelte'
  import { NEW_SESSION_ROSE_MAX, ROSE_FALLBACK, watchRoseHeight } from '../home/rose-height'
  import { chatCopy } from './copy'

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
</div>
