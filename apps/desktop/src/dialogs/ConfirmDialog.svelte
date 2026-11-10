<script lang="ts">
  // A confirm dialog: the extension's title, its question, and iced's three
  // answers (Yes, No, Cancel). The focus opens on Yes, the answer a user
  // confirms a prompt with, and the whole modal traps it from there.
  import { Dialog } from 'bits-ui'
  import type { DialogAnswer, DialogView } from '../contract/bindings'
  import { CANCELLED, confirmedAnswer } from './answer'
  import { CANCEL, NO, YES } from './copy'

  let {
    dialog,
    onAnswer,
  }: {
    dialog: Extract<DialogView, { kind: 'Confirm' }>
    onAnswer: (answer: DialogAnswer) => void
  } = $props()

  let primary = $state<HTMLButtonElement | null>(null)

  $effect(() => {
    primary?.focus()
  })
</script>

<Dialog.Title class="dt">{dialog.title}</Dialog.Title>
<Dialog.Description class="msg">{dialog.message}</Dialog.Description>
<div class="acts">
  <button class="btn primary" type="button" bind:this={primary} onclick={() => onAnswer(confirmedAnswer(true))}>
    {YES}
  </button>
  <button class="btn" type="button" onclick={() => onAnswer(confirmedAnswer(false))}>{NO}</button>
  <button class="btn" type="button" onclick={() => onAnswer(CANCELLED)}>{CANCEL}</button>
</div>
