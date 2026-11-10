<script lang="ts">
  // An editor dialog: iced's text box, which here is a multiline one. Enter is a
  // newline (the box is for writing), so the submit is Ctrl+Enter (Cmd+Enter on
  // a Mac). The text starts from the extension's prefill, lives here and nowhere
  // else (S2), and empties again when the dialog id changes.
  import { untrack } from 'svelte'
  import { Dialog } from 'bits-ui'
  import type { DialogAnswer, DialogView } from '../contract/bindings'
  import { CANCELLED, valueAnswer } from './answer'
  import { CANCEL, SUBMIT } from './copy'

  let {
    dialog,
    onAnswer,
  }: {
    dialog: Extract<DialogView, { kind: 'Editor' }>
    onAnswer: (answer: DialogAnswer) => void
  } = $props()

  // The prefill is the text the box STARTS with: the host remounts this
  // component when the dialog id changes, so a later push never rewrites what
  // the user is writing.
  let typed = $state(untrack(() => dialog.prefill) ?? '')
  let box = $state<HTMLTextAreaElement | null>(null)

  $effect(() => {
    box?.focus()
  })

  function keyed(event: KeyboardEvent): void {
    // A plain Enter keeps the newline the box gives it; only the chord submits.
    if (event.key !== 'Enter' || !(event.ctrlKey || event.metaKey)) return
    event.preventDefault()
    submit()
  }

  function submit(): void {
    onAnswer(valueAnswer(typed))
  }
</script>

<Dialog.Title class="dt">{dialog.title}</Dialog.Title>
<textarea
  class="field area"
  rows="6"
  spellcheck="false"
  aria-label={dialog.title}
  bind:this={box}
  bind:value={typed}
  onkeydown={keyed}
></textarea>
<div class="acts">
  <button class="btn" type="button" onclick={() => onAnswer(CANCELLED)}>{CANCEL}</button>
  <button class="btn primary" type="button" onclick={submit}>{SUBMIT}</button>
</div>
