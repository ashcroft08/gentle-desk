<script lang="ts">
  // The status row's CLOCK (design/composer-dock S1; sheet D of
  // 02-mode-picker): the last line of the thread, above the composer. It shares
  // the reasoning row's anatomy instead of growing a third visual language - a
  // thin spinner, the word, the seconds - faint, 11.5px, no card, no glyph soup.
  //
  // The ROW is permanent since sheet D: the chat always has it, and the session
  // mode control sits at its right edge whether the chat is working or idle. Only
  // this side of it belongs to the turn - the clock is drawn while the mark is
  // `Working` and nothing at all is drawn while it is not. The rotating body of
  // the clock is below; the row and the control are `./SessionModeControl`'s, and
  // what it draws is what this file hands it.
  //
  // The clock is the WEB's: it counts from the moment the chat's mark BECOMES
  // working and stops when the mark leaves, which is why the contract carries no
  // timestamp for it and the thread gains no field. The line disappears with the
  // turn; a mark that is not `Working` (opening, queued, ended) is another line's
  // business (`./Activity`).
  import type {
    ChatActivityMark,
    SessionModeView_Serialize as SessionModeView,
  } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import SessionModeControl from './SessionModeControl.svelte'
  import { threadCopy } from './copy'

  let {
    mark = null,
    chatId = null,
    sessionMode = null,
  }: {
    mark?: ChatActivityMark | null
    /** The foreground chat the write names, or `null` while none is open. */
    chatId?: string | null
    /** What that chat's child reported (T3b), or `null` when nothing answered. */
    sessionMode?: SessionModeView | null
  } = $props()

  /** The line's own word in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(threadCopy(language.current))

  /** How long the chat has been working, in whole seconds. */
  let seconds = $state(0)

  // One clock per turn: it starts the moment the mark becomes `Working`, ticks
  // while the line is drawn, and is thrown away - never left running - the moment
  // the mark changes. A remount starts from zero, which is the turn's own zero.
  $effect(() => {
    if (mark !== 'Working') {
      seconds = 0
      return
    }
    const start = Date.now()
    seconds = 0
    const tick = setInterval(() => {
      seconds = Math.floor((Date.now() - start) / 1000)
    }, 1000)
    return () => clearInterval(tick)
  })
</script>

<!-- The row is always there (sheet D): the control on the right always, the
     clock on the left only while the mark is `Working`, and nothing standing in
     for the clock when it is not. -->
<SessionModeControl {chatId} {sessionMode}>
  {#if mark === 'Working'}
    <span class="spin" aria-hidden="true"></span>
    <span class="t">{copy.working}</span>
    <span class="n" aria-hidden="true">{seconds}s</span>
  {/if}
</SessionModeControl>
