<script lang="ts">
  // The channel's OWN confirmation, drawn verbatim (profiles-editor T3b, L30
  // ruling 2). `Make default` is the channel's global apply, and gentle-pi asks
  // before it writes: the title is its title, the message is its message, in the
  // language it wrote them in. Nothing here rewrites, shortens or translates that
  // text - it is the child's own dialog, exactly like the terminal's.
  //
  // The two buttons are the app's: Accept sends the SAME profile and mode back
  // with the token of what was shown (the token is bound to it), and Cancel sends
  // NOTHING at all - the flow just ends and the dialog goes away.
  //
  // The component owns which dialog is DONE with (`settled`): a state push that
  // carries the same answer is a new view of the same chat, not a reason to
  // reopen a dialog the user closed, so only a NEW token asks again.
  import { dispatchIntent } from '../bridge'
  import type { ChatProfileApplyConfirmationView as Confirmation } from '../contract/bindings'
  import type { ApplyCopy } from './apply-copy'

  let {
    chatId = null,
    confirmation = null,
    copy,
  }: {
    /** The chat the apply belongs to: the one the token was issued for. */
    chatId?: string | null
    /** The child's own dialog as the last answer carried it, or `null`. */
    confirmation?: Confirmation | null
    copy: ApplyCopy
  } = $props()

  /** The token of a dialog this tab is done with: cancelled, or being accepted. */
  let settled = $state<string | null>(null)

  /** Whether the child is asking right now and nobody has answered yet. */
  const asking = $derived(confirmation !== null && confirmation.token !== settled)

  /** Accept: the SAME profile and mode, plus the token of what was shown. */
  function accept(shown: Confirmation): void {
    if (chatId === null) return
    settled = shown.token
    void dispatchIntent({
      type: 'ApplyChatProfile',
      data: {
        chat_id: chatId,
        profile: shown.profile,
        mode: shown.mode,
        confirmation: shown.token,
      },
    })
  }

  /** Cancel: nothing is sent, and the dialog goes away. */
  function cancel(): void {
    if (confirmation !== null) settled = confirmation.token
  }

  /** Escape cancels, as a dialog's own key does: nothing is sent either way. */
  function key(event: KeyboardEvent): void {
    if (!asking) return
    if (event.key === 'Escape') {
      event.preventDefault()
      cancel()
    }
  }
</script>

<svelte:window onkeydown={key} />

{#if asking && confirmation !== null}
<div class="ovl" aria-hidden="true"></div>
<div
  class="confirm"
  role="alertdialog"
  aria-modal="true"
  aria-labelledby="apply-confirm-title"
  aria-describedby="apply-confirm-message"
>
  <!-- The child's own words, and only they: no label, no punctuation, no
       translation of our own. -->
  <h2 id="apply-confirm-title">{confirmation.title}</h2>
  <p id="apply-confirm-message" class="cmsg">{confirmation.message}</p>
  <p class="cnote">{copy.confirmNote}</p>
  <div class="cacts">
    <button class="abtn" type="button" onclick={cancel}>{copy.confirmCancel}</button>
    <button class="abtn primary" type="button" onclick={() => accept(confirmation)}>
      {copy.confirmAccept}
    </button>
  </div>
</div>
{/if}

<style>
  /* The artboard's own backdrop and card (06-profile-tab.html), in the app's
     tokens: the panel is one column, so the dialog centers over the window. */
  .ovl {
    position: fixed;
    inset: 0;
    z-index: 40;
    background: rgb(0 0 0 / 45%);
  }

  .confirm {
    position: fixed;
    z-index: 41;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    width: min(440px, calc(100% - 32px));
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 14px;
    box-shadow: var(--shadow);
    padding: 18px;
  }

  h2 {
    margin: 0 0 6px;
    font-size: 15px;
    font-weight: 600;
  }

  .cmsg {
    margin: 0 0 8px;
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.55;
    /* The child words its message in lines of its own; they are kept. */
    white-space: pre-wrap;
  }

  .cnote {
    margin: 0 0 8px;
    color: var(--faint);
    font-size: 11.5px;
    line-height: 1.5;
  }

  .cacts {
    display: flex;
    gap: 8px;
    justify-content: flex-end;
    margin-top: 14px;
  }

  .abtn {
    border: 1px solid var(--border);
    border-radius: 8px;
    background: transparent;
    color: var(--text);
    font: inherit;
    font-size: 12.5px;
    padding: 5px 12px;
    cursor: pointer;
  }

  .abtn:hover {
    background: var(--surface2);
  }

  .abtn:focus-visible {
    outline: 1px solid var(--text);
    outline-offset: -2px;
  }

  .abtn.primary {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--bg);
    font-weight: 600;
  }
</style>
