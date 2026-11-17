<script lang="ts">
  // The Profile tab of one chat (profiles-editor T2c, artboard 06-profile-tab.html):
  // the READ-ONLY half of the chosen design. It says which profile this chat runs
  // and in whose scope, lists the profiles the store defines, and draws the pins of
  // this chat's own worktree - and it writes NOTHING: Save, Use in this chat, Make
  // default, the model pickers and the fallback editor are T3 and later, and none of
  // them is drawn here, not even disabled.
  //
  // When it reads (L13): opening the tab asks ONCE for the foreground chat's view;
  // the window taking the focus back while the tab is open asks again, and so does
  // the unavailable state's own Try again. The asks are COALESCED like the Settings
  // panes' (`settings/files-changed.ts`): one read at a time, one queued, and
  // nothing at all once the tab closes - a follow-up queued while a read was in
  // flight is dropped with it. A chat with no id asks for nothing.
  import { dispatchIntent } from '../bridge'
  import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { coalesced, type CoalescedRead } from '../settings/files-changed'
  import ProfileBody from './ProfileBody.svelte'
  import ProfileHead from './ProfileHead.svelte'
  import ProfileNotice from './ProfileNotice.svelte'
  import { profileCopy } from './copy'
  import { panelState } from './rules'

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

  /** This open tab's read, so Try again and a window focus share one path. */
  let read: CoalescedRead | null = null

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
</script>

<div class="pv">
  <div class="pv-in">
    {#if shape === 'reading'}
      <!-- The first read of a freshly opened tab: quiet, never an error. -->
      <p class="quiet">{copy.reading}</p>
    {:else if shape === 'unavailable'}
      <!-- State 10: this chat's child did not answer, or no store could be read.
           Edit in terminal is T-later and is deliberately NOT drawn. -->
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
        <ProfileBody {view} {copy} />
      {/key}
      <!-- The footer says what this slice is: read-only. The three outcomes of the
           artboard's footer arrive with T3, T3b. -->
      <p class="foot">{copy.readOnly}</p>
    {/if}
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
</style>
