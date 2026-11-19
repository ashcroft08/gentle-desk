<script lang="ts">
  // The status line (design/composer-dock S1, R1): the last line of the thread,
  // above the composer, that says what the chat is doing. It shares the reasoning
  // row's anatomy instead of growing a third visual language - a thin spinner, the
  // word, the seconds - faint, 11.5px, no card, no glyph soup.
  //
  // The clock is the WEB's: it counts from the moment the chat's mark BECOMES
  // working and stops when the mark leaves, which is why the contract carries no
  // timestamp for it and the thread gains no field. The line disappears with the
  // turn; a mark that is not `Working` (opening, queued, ended) is another line's
  // business (`./Activity`).
  //
  // The right side keeps the place the session mode takes - a read-only label,
  // never a switch (R1) - and this slice draws nothing there: no gap, no stub.
  import type { ChatActivityMark } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { threadCopy } from './copy'

  let { mark = null }: { mark?: ChatActivityMark | null } = $props()

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
  <p class="work-line">
    <span class="spin" aria-hidden="true"></span>
    <span class="t">{copy.working}</span>
    <span class="n" aria-hidden="true">{seconds}s</span>
  </p>
{/if}
