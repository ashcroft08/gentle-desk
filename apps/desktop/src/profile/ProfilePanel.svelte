<script lang="ts">
  // The Profile tab of one chat (profiles-editor T2c, T3-mode, artboard
  // 06-profile-tab.html): which profile this chat runs and in whose scope, the
  // profiles the store defines, the pins of this chat's own worktree - and the one
  // thing it WRITES, the used profile's orchestration mode (the Organic / Full
  // delegation switch, saved through the shell in the profile itself). Save, Use
  // in this chat, Make default, the model pickers and the fallback editor are T3
  // and T3b: none of them is drawn here, not even disabled.
  //
  // The one action it does offer is the terminal (T2d, L21-L22): Settings > Profiles
  // is gone, so the profiles screen the terminal already edits is opened from this
  // footer. It is honest about what it is - a window, not a write - and it is drawn
  // in every state, because a chat whose read failed is exactly when a profile may
  // need the terminal.
  //
  // When it reads (L13): opening the tab asks ONCE for the foreground chat's view;
  // the window taking the focus back while the tab is open asks again, and so does
  // the unavailable state's own Try again. The asks are COALESCED like the Settings
  // panes' (`settings/files-changed.ts`): one read at a time, one queued, and
  // nothing at all once the tab closes - a follow-up queued while a read was in
  // flight is dropped with it. A chat with no id asks for nothing.
  import { dispatchIntent, openTerminal } from '../bridge'
  import type {
    ChatProfileView_Serialize as ChatProfile,
    OrchestratorModeView,
  } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { coalesced, type CoalescedRead } from '../settings/files-changed'
  import ProfileBody from './ProfileBody.svelte'
  import ProfileHead from './ProfileHead.svelte'
  import ProfileNotice from './ProfileNotice.svelte'
  import ProfileRoutes from './ProfileRoutes.svelte'
  import { profileCopy } from './copy'
  import { panelState, usedProfile } from './rules'

  let {
    chatId = null,
    view = null,
    active = false,
  }: {
    /** The foreground chat this view belongs to, or `null` while none is open. */
    chatId?: string | null
    /** The profile view Rust pushed for that chat, or `null`. */
    view?: ChatProfile | null
    /** Whether this is the tab on screen: a read happens only while it is. */
    active?: boolean
  } = $props()

  /** The tab's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(profileCopy(language.current))

  /** Whether a read has ANSWERED: before the first one, no view is not a failure. */
  let asked = $state<boolean>(false)
  // Not named `state`: inside a component `$state` would then read as a store
  // subscription of a local, and the rune itself is what this file needs.
  const shape = $derived(panelState(view, asked))

  /** The profile this chat runs, when the store still defines it (L19). */
  const used = $derived(usedProfile(view))

  /** This open tab's read, so Try again and a window focus share one path. */
  let read: CoalescedRead | null = null

  /** What the terminal said when it could not open, or `null` (T2d). */
  let terminalError = $state<string | null>(null)

  /**
   * The chat this tab reads, as a VALUE. The parent hands `chatId` down off an
   * object that every state push replaces, so reading the prop subscribes to that
   * object: without this derived, each push - one per child event of a streaming
   * chat, and one per answered read - re-ran the read effect below and asked
   * again, a read loop that froze the window. A derived of a string changes only
   * when the string does, so the effect depends on the chat, not on the push.
   */
  const chat = $derived(chatId)

  /** One ask for this chat's view, or nothing when there is no chat to ask about. */
  function ask(): Promise<void> {
    if (chat === null) return Promise.resolve()
    return dispatchIntent({ type: 'RefreshChatProfile', data: { chat_id: chat } })
  }

  // One read per open, one per focus, and none after the tab closes. The read is
  // built per activation, so a tab opened a second time reads again instead of
  // inheriting the cancelled one. A chat with no id asks for nothing and answers
  // nothing: the tab is not drawn without a chat anyway.
  $effect(() => {
    if (!active) return
    asked = false
    const current = coalesced(async () => {
      if (chat === null) return
      await ask()
      asked = true
    })
    read = current
    void current()
    const again = () => void current()
    window.addEventListener('focus', again)
    return () => {
      window.removeEventListener('focus', again)
      current.cancel()
      if (read === current) read = null
    }
  })

  /** The unavailable state's one action: the same coalesced read as the rest. */
  function tryAgain(): void {
    void read?.()
  }

  /**
   * Saves the used profile's orchestration mode (profiles-editor T3-mode): the
   * chat the tab is about, the profile THIS chat uses, and the mode the user
   * chose. Nothing is sent for the mode the profile already stores, and nothing
   * while a save is in flight - the switch draws both rules disabled.
   */
  function saveMode(mode: OrchestratorModeView): void {
    if (chat === null || used === null || mode === used.mode) return
    void dispatchIntent({
      type: 'SaveChatProfileMode',
      data: { chat_id: chat, profile: used.name, mode },
    })
  }

  /**
   * The footer's one action: the shell's own profiles screen in a terminal
   * window, the way the Settings pane opened it. Nothing of this tab is written
   * and nothing is sent: Rust resolves the launcher, and a refusal - no launcher,
   * no console - is the one line drawn under the button.
   */
  async function edit(): Promise<void> {
    terminalError = await openTerminal('Profiles')
  }
</script>

<div class="pv">
  <div class="pv-in">
    {#if shape === 'reading'}
      <!-- The first read of a freshly opened tab: quiet, never an error. -->
      <p class="quiet">{copy.reading}</p>
    {:else if shape === 'unavailable'}
      <!-- State 10: this chat's child did not answer, or no store could be read.
           The footer's terminal action stays: editing a profile in the terminal
           does not depend on this chat answering. -->
      <div class="notice">
        <ProfileNotice
          title={copy.unavailable}
          why={copy.unavailableWhy}
          tryAgain={copy.tryAgain}
          onTryAgain={tryAgain}
        />
      </div>
    {:else if view !== null}
      <!-- Keyed by chat: another chat's profile starts from its own first group,
           never from the one the reader left open. -->
      {#key view.chat_id}
        <ProfileHead {view} {copy} />
        {#if used !== null}
          <!-- The profile's own pairs and its mode, as Settings > Profiles draws
               them (L19): the switch SAVES the profile's mode through the shell
               (T3-mode, L23), and the copy says whose mode it is. -->
          <ProfileRoutes
            profile={used}
            {copy}
            saving={view.saving}
            saveFailed={view.save_failed}
            onSave={saveMode}
          />
        {/if}
        <ProfileBody {view} {copy} />
      {/key}
      <!-- The footer line says what this slice saves: the profile's mode, and
           nothing else yet. The three outcomes of the artboard's footer arrive
           with T3b. -->
      <p class="foot-line">{copy.footer}</p>
    {/if}

    <!-- The footer, in every state (T2d, L21-L22): the tab saves the used
         profile's mode and nothing else, so the way to edit, create or switch a
         profile is the terminal the Settings pane used to open. -->
    <div class="foot">
      <button class="btn" type="button" onclick={() => void edit()}>{copy.editInTerminal}</button>
      <p class="note">{copy.editNote}</p>
      {#if terminalError !== null}
        <p class="quiet" role="status">{terminalError}</p>
      {/if}
    </div>
  </div>
</div>

<style>
  /* The artboard's `.pv`: the view fills the panel and scrolls its own body. */
  .pv {
    display: flex;
    flex-direction: column;
    flex: 1 1 0;
    min-height: 0;
    background: var(--bg);
  }

  .pv-in {
    flex: 1 1 0;
    min-height: 0;
    padding: 22px 24px 8px;
    overflow: auto;
  }

  .notice {
    max-width: var(--chat-max);
    margin: 0 auto;
  }

  .quiet {
    max-width: var(--chat-max);
    margin: 0 auto;
    color: var(--muted);
    font-size: 12.5px;
    line-height: 1.5;
  }

  .foot {
    max-width: var(--chat-max);
    margin: 16px auto 8px;
    color: var(--faint);
    font-size: 11.5px;
  }

  /* The line e892d5d drew in the footer before the terminal action joined it:
     what this tab saves, in the footer's own quiet weight. */
  .foot-line {
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
</style>
