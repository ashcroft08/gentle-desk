<script lang="ts">
  // Which profile this chat runs, said in one glance (06-profile-tab.html, states 1
  // and 2): its name, the scope it carries, the line that says what that scope
  // means, and ONE short honest line about where the answer comes from - the chat's
  // own session file, never the channel (L8), so the maintainer's approximation
  // (L9) is on screen and not just in the code.
  //
  // A chat with no binding and no default profile draws no name and no tag: the
  // neutral line is the whole truth then, and inventing a name for it would be a
  // guess.
  import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
  import type { ProfileCopy } from './copy'
  import { usedName } from './rules'

  let { view, copy }: { view: ChatProfile; copy: ProfileCopy } = $props()

  const name = $derived(usedName(view))
  /** Whether this chat bound a profile of its own, which is the tag's own rule. */
  const bound = $derived(view.bound !== null)
  const line = $derived(
    name === null ? copy.lineDefaultNone : bound ? copy.lineSession(name) : copy.lineDefault(name),
  )
</script>

<header class="head">
  <p class="kicker">{copy.kicker}</p>
  {#if name !== null}
    <h2>
      <span class="pname">{name}</span>
      <span class="tag" class:here={bound}>{bound ? copy.tagSession : copy.tagDefault}</span>
    </h2>
  {/if}
  <p class="line">{line}</p>
  <p class="approx">{copy.approximation}</p>
</header>

<style>
  /* The artboard's `.pv-head`: it shares the thread's width, so the view lines up
     with the conversation it describes. */
  .head {
    max-width: var(--chat-max);
    margin: 0 auto;
  }

  .kicker {
    color: var(--faint);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  h2 {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin: 2px 0 4px;
    font-size: 18px;
    font-weight: 600;
  }

  .pname {
    overflow-wrap: anywhere;
  }

  /* The house's tag: the scope is its WORD, and the colour only repeats it. */
  .tag {
    padding: 2px 8px;
    border: 1px solid currentColor;
    border-radius: 999px;
    color: var(--muted);
    font-size: 11px;
    font-weight: 600;
    white-space: nowrap;
  }

  .tag.here {
    color: var(--accent);
  }

  .line {
    max-width: 640px;
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.55;
  }

  .approx {
    margin-top: 6px;
    color: var(--faint);
    font-size: 11.5px;
    line-height: 1.5;
  }
</style>
