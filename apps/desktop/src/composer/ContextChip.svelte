<script lang="ts">
  // The context chip (0.1.12, S5): how full the foreground chat's context is, as
  // pi itself measured it. One number and one tiny bar, beside the model pills;
  // the tokens behind the number are the hover, never the default.
  //
  // It renders and asks nothing: Rust reads the chat's own stats when the turn
  // settles and when the chat opens, and pushes a `ContextUsageView`. A reading
  // of null is NOT zero - a chat that just compacted draws a dash, and a chat
  // with no reading at all draws nothing (the rules live in `./context`).
  import type { ContextUsageView_Serialize as ContextUsageView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { barWidth, chipOf } from './context'
  import { composerCopy } from './copy'

  let { usage }: { usage: ContextUsageView | null } = $props()

  /** The chip's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(composerCopy(language.current))
  const chip = $derived(chipOf(usage, copy))
</script>

{#if chip.kind === 'reading'}
  <span
    class="ctx {chip.tone}"
    role="status"
    aria-label={chip.label}
    title={chip.title}
    data-percent={chip.percent}
  >
    <span class="bar" aria-hidden="true"><i style="width: {barWidth(chip.percent)}"></i></span>
    {chip.percent}%
  </span>
{:else if chip.kind === 'unknown'}
  <span class="ctx unknown" role="status" aria-label={chip.label} title={chip.title}>
    &ndash;
  </span>
{/if}
