<script lang="ts">
  // An input dialog: one line of text, the extension's hint as its placeholder,
  // Enter to submit and Cancel. What is typed lives HERE and nowhere else (S2):
  // it is never pushed, and the host remounts this component when the dialog id
  // changes, which is what empties the field for the next question.
  import { Dialog } from 'bits-ui'
  import type { DialogAnswer, DialogView } from '../contract/bindings'
  import { CANCELLED, valueAnswer } from './answer'
  import { CANCEL, SUBMIT } from './copy'

  let {
    dialog,
    onAnswer,
  }: {
    dialog: Extract<DialogView, { kind: 'Input' }>
    onAnswer: (answer: DialogAnswer) => void
  } = $props()

  let typed = $state('')
  let field = $state<HTMLInputElement | null>(null)

  $effect(() => {
    field?.focus()
  })

  function keyed(event: KeyboardEvent): void {
    if (event.key !== 'Enter') return
    event.preventDefault()
    submit()
  }

  function submit(): void {
    onAnswer(valueAnswer(typed))
  }
</script>

<Dialog.Title class="dt">{dialog.title}</Dialog.Title>
<input
  class="field"
  type="text"
  autocomplete="off"
  spellcheck="false"
  aria-label={dialog.title}
  placeholder={dialog.placeholder ?? ''}
  bind:this={field}
  bind:value={typed}
  onkeydown={keyed}
/>
<div class="acts">
  <button class="btn" type="button" onclick={() => onAnswer(CANCELLED)}>{CANCEL}</button>
  <button class="btn primary" type="button" onclick={submit}>{SUBMIT}</button>
</div>
