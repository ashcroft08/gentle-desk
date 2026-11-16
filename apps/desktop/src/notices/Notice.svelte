<script lang="ts">
// One notice card: the level's mark and word, the extension's own text, at most
// one action that fixes what it reports, and the dismiss. Every button is an
// intent, because the dismissal and the fix are the state's business.
//
// The word is what makes the level readable without the colour or the shape: a
// screen reader hears "Warning" before the text, and the card announced as an
// alert or a status matches the board's two roles. That word, the dismiss and
// the action's name are the notice's own words, so the card reads them from the
// copy dictionary of the current language (0.2.0, D4): the text itself is the
// child's and is drawn exactly as it arrived (D7).

import { dispatchIntent } from '../bridge'
import type { NoticeView } from '../contract/bindings'
import Icon from '../icons/Icon.svelte'
import Mark from '../icons/Mark.svelte'
import { markLabelsCopy } from '../icons/shapes'
import { language } from '../i18n/language.svelte'
import { actionOf } from './action'
import { noticesCopy } from './copy'
import { noticeLookOf } from './level'

let { notice }: { notice: NoticeView } = $props()

const copy = $derived(noticesCopy(language.current))
const look = $derived(noticeLookOf(notice.level))
const action = $derived(notice.action == null ? null : actionOf(notice.action, copy))
// The mark's own name, in the card's language: a bare Mark would draw the
// English default in both (0.2.0, S3/D4).
const marks = $derived(markLabelsCopy(language.current))

function ran(intent: Parameters<typeof dispatchIntent>[0]): void {
  void dispatchIntent(intent)
}
</script>

<div class="notice {look.tone}" role={look.role}>
  {#if look.mark !== null}
    <Mark kind={look.mark} label={marks[look.mark]} />
  {/if}
  <span class="vh">{copy.levels[notice.level]}</span>
  <span class="m">{notice.text}</span>
  {#if action !== null}
    <button class="ic tip-r" type="button" data-tip={action.label} aria-label={action.label} onclick={() => ran(action.intent)}>
      <Icon name={action.icon} />
    </button>
  {/if}
  <button
    class="ic tip-r"
    type="button"
    data-tip={copy.dismiss}
    aria-label={copy.dismiss}
    onclick={() => ran({ type: 'DismissNotice', data: { id: notice.id } })}
  >
    <Icon name="x" />
  </button>
</div>
