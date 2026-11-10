<script lang="ts">
// One notice card: the level's mark and word, the extension's own text, at most
// one action that fixes what it reports, and the dismiss. Every button is an
// intent, because the dismissal and the fix are the state's business.
//
// The word is what makes the level readable without the colour or the shape: a
// screen reader hears "Warning" before the text, and the card announced as an
// alert or a status matches the board's two roles.

import { dispatchIntent } from '../bridge'
import type { NoticeView } from '../contract/bindings'
import Icon from '../icons/Icon.svelte'
import Mark from '../icons/Mark.svelte'
import { actionOf } from './action'
import { DISMISS } from './copy'
import { noticeLookOf } from './level'

let { notice }: { notice: NoticeView } = $props()

const look = $derived(noticeLookOf(notice.level))
const action = $derived(notice.action == null ? null : actionOf(notice.action))

function ran(intent: Parameters<typeof dispatchIntent>[0]): void {
  void dispatchIntent(intent)
}
</script>

<div class="notice {look.tone}" role={look.role}>
  {#if look.mark !== null}
    <Mark kind={look.mark} />
  {/if}
  <span class="vh">{look.word}</span>
  <span class="m">{notice.text}</span>
  {#if action !== null}
    <button class="ic tip-r" type="button" data-tip={action.label} aria-label={action.label} onclick={() => ran(action.intent)}>
      <Icon name={action.icon} />
    </button>
  {/if}
  <button
    class="ic tip-r"
    type="button"
    data-tip={DISMISS}
    aria-label={DISMISS}
    onclick={() => ran({ type: 'DismissNotice', data: { id: notice.id } })}
  >
    <Icon name="x" />
  </button>
</div>
