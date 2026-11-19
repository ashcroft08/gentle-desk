<script lang="ts">
  // The status line's CLOCK (design/composer-dock S1): the last line of the
  // thread, above the composer, that says what the chat is doing. It shares the
  // reasoning row's anatomy instead of growing a third visual language - a thin
  // spinner, the word, the seconds - faint, 11.5px, no card, no glyph soup.
  //
  // The clock is the WEB's: it counts from the moment the chat's mark BECOMES
  // working and stops when the mark leaves, which is why the contract carries no
  // timestamp for it and the thread gains no field. The line disappears with the
  // turn; a mark that is not `Working` (opening, queued, ended) is another line's
  // business (`./Activity`).
  //
  // The row itself is drawn by `./SessionModeControl`, which owns the session
  // mode at its right edge and the refusal line under it (composer-dock T3c); the
  // clock is what this file hands it, and the only thing here that ticks.
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

{#if mark === 'Working'}
  <SessionModeControl {chatId} {sessionMode}>
    <span class="spin" aria-hidden="true"></span>
    <span class="t">{copy.working}</span>
    <span class="n" aria-hidden="true">{seconds}s</span>
  </SessionModeControl>
{/if}
