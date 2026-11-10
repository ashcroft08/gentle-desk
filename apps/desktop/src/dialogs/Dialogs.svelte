<script lang="ts">
  // Where a blocking extension dialog appears, over everything: the first
  // pending dialog of the foreground chat, as a modal (bits-ui Dialog: role
  // dialog, aria-modal, the focus trapped inside). While it is open the composer
  // behind it takes no input, and the thread keeps updating: the state keeps
  // arriving, only the answer waits.
  //
  // Three rules live here, because all four kinds share them:
  //
  // - Escape and Cancel answer `cancelled`. Clicking outside does NOT: the
  //   extension is blocked until the user really answers, as in iced, where the
  //   dialog had no way out but its own buttons.
  // - One answer per dialog. Rust drops a dialog it just answered, so the next
  //   push proves the answer landed; if it arrives before the answer is on the
  //   wire, a second click is dropped here instead of reaching the child twice.
  // - The dialog id is what the four components are keyed by, so a different
  //   question starts with an empty field even though the host stays mounted.
  import { Dialog } from 'bits-ui'
  import { dispatchIntent } from '../bridge'
  import type { DialogAnswer, DialogView } from '../contract/bindings'
  import { answerIntent, CANCELLED } from './answer'
  import ConfirmDialog from './ConfirmDialog.svelte'
  import EditorDialog from './EditorDialog.svelte'
  import InputDialog from './InputDialog.svelte'
  import SelectDialog from './SelectDialog.svelte'

  let { dialog }: { dialog: DialogView | null } = $props()

  /** The dialog this host has already answered, if it still waits for the push. */
  let answered = $state<string | null>(null)

  // A slice with no dialog, or another one, proves the last answer landed.
  $effect(() => {
    if (dialog === null || dialog.id !== answered) answered = null
  })

  function send(answer: DialogAnswer): void {
    if (dialog === null || answered === dialog.id) return
    answered = dialog.id
    void dispatchIntent(answerIntent(dialog, answer))
  }

  function escape(event: Event): void {
    // The primitive would close the modal on Escape; a blocking dialog closes
    // only when the state says so, so the key becomes the cancelled answer.
    event.preventDefault()
    send(CANCELLED)
  }
</script>

{#if dialog !== null}
  <Dialog.Root open={true}>
    <Dialog.Portal>
      <Dialog.Overlay class="ovl" />
      <Dialog.Content
        class="dlg"
        onOpenAutoFocus={(event) => event.preventDefault()}
        onEscapeKeydown={escape}
        onInteractOutside={(event) => event.preventDefault()}
      >
        {#key dialog.id}
          {#if dialog.kind === 'Confirm'}
            <ConfirmDialog {dialog} onAnswer={send} />
          {:else if dialog.kind === 'Select'}
            <SelectDialog {dialog} onAnswer={send} />
          {:else if dialog.kind === 'Input'}
            <InputDialog {dialog} onAnswer={send} />
          {:else}
            <EditorDialog {dialog} onAnswer={send} />
          {/if}
        {/key}
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
{/if}
