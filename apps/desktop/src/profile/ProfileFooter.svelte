<script lang="ts">
  // The Profile tab's footer (profiles-editor T2d, L21-L22): the one action the
  // tab still offers the terminal, and the line that keeps it honest about what
  // this tab writes itself (the used profile's mode, its chains, the pin of this
  // chat's worktree) and what is still the terminal's (creating and editing a
  // profile).
  //
  // It is drawn in EVERY state, because a chat whose read failed is exactly when a
  // profile may need the terminal. Nothing of the tab is written by it: the shell
  // resolves the launcher, and a refusal - no launcher, no console - is the one
  // line drawn under the button.
  import { openTerminal } from '../bridge'
  import type { ProfileCopy } from './copy'

  let { copy }: { copy: ProfileCopy } = $props()

  /** What the terminal said when it could not open, or `null`. */
  let terminalError = $state<string | null>(null)

  async function edit(): Promise<void> {
    terminalError = await openTerminal('Profiles')
  }
</script>

<div class="foot">
  <button class="btn" type="button" onclick={() => void edit()}>{copy.editInTerminal}</button>
  <p class="note">{copy.editNote}</p>
  {#if terminalError !== null}
    <p class="quiet" role="status">{terminalError}</p>
  {/if}
</div>

<style>
  .foot {
    max-width: var(--chat-max);
    margin: 16px auto 8px;
    color: var(--faint);
    font-size: 11.5px;
  }

  .note {
    margin: 8px 0 0;
    color: var(--muted);
    font-size: 11.5px;
    line-height: 1.5;
  }

  .quiet {
    margin: 0 auto;
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.5;
  }
</style>
