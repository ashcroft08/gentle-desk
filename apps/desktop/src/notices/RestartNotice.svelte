<script lang="ts">
  // The restart notice (the states board's error card): what happened, the one
  // action that fixes it - restart that chat's child - and a dismiss.
  //
  // The action names its own chat, so a background chat's death is fixed without
  // opening it first, and both buttons are intents: the restart and the dismissal
  // are the state's business, never the view's.
  import { dispatchIntent } from '../bridge'
  import type { NoticeView } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'
  import Mark from '../icons/Mark.svelte'
  import { DISMISS, RESTART } from './copy'
  import { restartNoticesOf } from './restart'

  let { notices = [] }: { notices?: NoticeView[] } = $props()

  const rows = $derived(restartNoticesOf(notices))

  function restart(chatId: string): void {
    void dispatchIntent({ type: 'RestartChat', data: { chat_id: chatId } })
  }

  function dismiss(id: number): void {
    void dispatchIntent({ type: 'DismissNotice', data: { id } })
  }
</script>

{#each rows as row (row.id)}
  <div class="notice err" role="alert">
    <Mark kind="Ended" />
    <span class="m">{row.text}</span>
    <button
      class="ic tip-r"
      type="button"
      data-tip={RESTART}
      aria-label={RESTART}
      onclick={() => restart(row.chatId)}
    >
      <Icon name="restart" />
    </button>
    <button
      class="ic tip-r"
      type="button"
      data-tip={DISMISS}
      aria-label={DISMISS}
      onclick={() => dismiss(row.id)}
    >
      <Icon name="x" />
    </button>
  </div>
{/each}
