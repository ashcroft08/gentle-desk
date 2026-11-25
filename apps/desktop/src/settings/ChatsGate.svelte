<script lang="ts">
  // The Updates gate (S4 of the original pane, close-chats S1/S3): the sentence that
  // says the update buttons need every chat closed, the button that closes them in
  // one step, and - when some chats are in the middle of a turn - the line that says
  // why the button leaves those open.
  //
  // Everything it draws stops existing the moment no chat is open: the pane only
  // renders this while `openChats > 0`, and the update buttons come alive by the
  // pane's own rule. Nothing here runs an update, asks for a confirmation or stops
  // a running turn.
  import { dispatchIntent } from '../bridge'
  import type { UpdatesCopy } from './updates-copy'

  let {
    hint,
    hintId,
    copy,
    workingChats = 0,
  }: {
    /** The gate sentence, already worded for the count (`copy.gateHint`). */
    hint: string
    /** The id the update buttons point at as their description. */
    hintId: string
    /** The pane's own words. */
    copy: UpdatesCopy
    /** How many open chats are mid-turn: the button does not close those. */
    workingChats?: number
  } = $props()
</script>

<p class="gate" id={hintId} role="status">{hint}</p>
<div class="acts">
  <button class="btn" type="button" onclick={() => void dispatchIntent({ type: 'CloseAllChats' })}>
    {copy.closeChats}
  </button>
</div>
{#if workingChats > 0}
  <p class="quiet" role="status">{copy.chatsWorking(workingChats)}</p>
{/if}
