<script lang="ts">
  // The terminal's OTHER two outcomes for the profile this chat uses
  // (profiles-editor T3b, S8, S9):
  //
  // - `Use in this chat`: `apply_profile` with `mode: session`, which binds the
  //   profile to THIS chat's own child and to no other, so the other open chats
  //   keep the routing they started with;
  // - `Make default`: `apply_profile` with `mode: default`, the channel's own
  //   GLOBAL apply (L30 ruling 1), which may answer its own confirmation first.
  //
  // The block sits beside the profile it acts on, and each button carries the
  // scope it touches instead of leaving the reader to guess: the copy says one
  // chat here and every chat that starts with the default there. Both are
  // disabled while an apply is in flight, and an answer that wrote nothing leaves
  // ONE short line under them (S4).
  import type { ChatProfileApplyNoticeView } from '../contract/bindings'
  import type { ApplyCopy } from './apply-copy'

  let {
    copy,
    applying = false,
    notice = null,
    onApply,
  }: {
    copy: ApplyCopy
    /** An apply is in flight: both outcomes are disabled until it answers. */
    applying?: boolean
    /** The last apply wrote nothing: its own short line says why. */
    notice?: ChatProfileApplyNoticeView | null
    /** Ask the shell to apply this chat's profile, in one of the two modes. */
    onApply?: (mode: 'session' | 'default') => void
  } = $props()

  /**
   * One outcome. Nothing is sent while an apply is in flight - the disabled
   * buttons say so, and this guard keeps a keyboard or a test from going around
   * them - so one click can never become two applies (L18, L29).
   */
  function apply(mode: 'session' | 'default'): void {
    if (applying) return
    onApply?.(mode)
  }
</script>

<div class="apply">
  <div class="abtns">
    <button class="abtn" type="button" disabled={applying} onclick={() => apply('session')}>
      {copy.useInChat}
    </button>
    <button class="abtn" type="button" disabled={applying} onclick={() => apply('default')}>
      {copy.makeDefault}
    </button>
  </div>
  <p class="anote">{copy.useInChatNote}</p>
  <p class="anote">{copy.makeDefaultNote}</p>
  {#if notice !== null}
    <!-- One short line, one per answer that wrote nothing (S4). -->
    <p class="afail" role="status">{copy.notices[notice]}</p>
  {/if}
</div>

<style>
  /* The mode row's own shape, so the block reads as one column of the profile's
     settings. */
  .apply {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 4px 10px;
  }

  .abtns {
    display: inline-flex;
    gap: 6px;
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

  .abtn:disabled {
    cursor: default;
    opacity: 0.55;
  }

  /* Each outcome's scope: this chat only, or global (S9, L30). */
  .anote {
    flex-basis: 100%;
    margin: 0;
    color: var(--muted);
    font-size: 11.5px;
    line-height: 1.5;
  }

  .afail {
    flex-basis: 100%;
    margin: 0;
    color: var(--danger);
    font-size: 11.5px;
    line-height: 1.5;
  }
</style>
