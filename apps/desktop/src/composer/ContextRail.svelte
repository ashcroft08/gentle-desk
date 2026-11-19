<script lang="ts">
  // The context rail (design/composer-dock S1, S5): the 2px line ABOVE the
  // composer, outside its border, that fills with how full this chat's context
  // is. It replaces the chip that used to sit in the row: the percent at the left
  // with a clock glyph, the model's window at the right, and no box anywhere -
  // the only bordered surface of the dock is the composer itself.
  //
  // It renders and asks nothing: Rust reads the chat's own stats when the turn
  // settles and when the chat opens, and pushes a `ContextUsageView`. A reading of
  // null is NOT zero (the rules live in `./context`, unchanged): no reading draws
  // no rail, and a chat that just compacted draws a dash with an empty rail.
  import type { ContextUsageView_Serialize as ContextUsageView } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'
  import { language } from '../i18n/language.svelte'
  import { barWidth, chipOf } from './context'
  import { CONTEXT_DASH, composerCopy, formatTokens } from './copy'

  let { usage }: { usage: ContextUsageView | null } = $props()

  /** The rail's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(composerCopy(language.current))
  const chip = $derived(chipOf(usage, copy))
  /** The denominator pi reported, drawn as the window size at the right. */
  const window = $derived(usage?.window ?? null)
</script>

{#if chip.kind !== 'hidden'}
  <!-- The title and the live region live on the WHOLE rail, not on the 2px bar:
       the reader hovers the percentage, not the hairline, and a screen reader must
       hear the number that changes (review R3-1, R3-2). -->
  <div class="ctx-rail {chip.kind === 'reading' ? chip.tone : 'unknown'}" title={chip.title}>
    <div class="lab" role="status" aria-label={chip.label}>
      <span class="l">
        <Icon name="clock" size={11} />
        {chip.kind === 'reading' ? `${chip.percent}%` : CONTEXT_DASH}
      </span>
      <span>{window === null ? '' : formatTokens(window)}</span>
    </div>
    <div class="rail">
      {#if chip.kind === 'reading'}
        <i style="width: {barWidth(chip.percent)}"></i>
      {/if}
    </div>
  </div>
{/if}
