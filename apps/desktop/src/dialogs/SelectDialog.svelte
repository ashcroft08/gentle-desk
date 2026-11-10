<script lang="ts">
  // A select dialog: one button per option, in the order the extension sent
  // them. The focus opens on the first option and the arrows walk the list, so a
  // keyboard user picks one without touching the mouse; Enter answers the option
  // the focus is on, and a click answers the one clicked.
  import { Dialog } from 'bits-ui'
  import type { DialogAnswer, DialogView } from '../contract/bindings'
  import { CANCELLED, valueAnswer } from './answer'
  import { CANCEL } from './copy'

  let {
    dialog,
    onAnswer,
  }: {
    dialog: Extract<DialogView, { kind: 'Select' }>
    onAnswer: (answer: DialogAnswer) => void
  } = $props()

  /** The options' buttons, for the focus and the arrow keys. */
  let buttons = $state<HTMLButtonElement[]>([])
  /** Which option the keys act on. */
  let active = $state(0)

  $effect(() => {
    buttons[active]?.focus()
  })

  function keyed(event: KeyboardEvent): void {
    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault()
      const count = dialog.options.length
      if (count === 0) return
      const step = event.key === 'ArrowDown' ? 1 : count - 1
      active = (active + step) % count
      return
    }
    // Enter answers the highlighted option here, because a key that arrives at
    // the container has no button of its own to click.
    if (event.key === 'Enter') {
      event.preventDefault()
      choose(active)
    }
  }

  function choose(index: number): void {
    const option = dialog.options[index]
    if (option !== undefined) onAnswer(valueAnswer(option))
  }
</script>

<Dialog.Title class="dt">{dialog.title}</Dialog.Title>
<div class="opts">
  {#each dialog.options as option, index (index)}
    <button
      class="btn opt"
      type="button"
      tabindex={index === active ? 0 : -1}
      bind:this={buttons[index]}
      onkeydown={keyed}
      onclick={() => choose(index)}
    >
      {option}
    </button>
  {/each}
</div>
<div class="acts">
  <button class="btn" type="button" onclick={() => onAnswer(CANCELLED)}>{CANCEL}</button>
</div>
