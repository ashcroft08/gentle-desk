<script lang="ts">
  // The delete confirm dialog (T3, S4, D2(b)): the states board's own card - the
  // question, the line that says it cannot be undone, and the chat's REAL file
  // path, so the user confirms a file and not a row.
  //
  // Cancel and Escape close it and send nothing. Delete sends the path ONCE and
  // the dialog closes when Rust answers, so a second click cannot remove another
  // file. The focus opens on Cancel, the answer that changes nothing. A refusal
  // is not drawn here: a chat that is working asks the user to stop it first, and
  // the sidebar's notice line says so once the dialog is gone.
  import { Dialog } from 'bits-ui'
  import { CANCEL, DELETE_BODY, DELETE_CONFIRM, DELETE_TITLE } from './copy'

  let {
    open = $bindable(false),
    path,
    onDelete,
  }: {
    /** Whether the dialog is open. */
    open?: boolean
    /** The session file's real path, quoted exactly as the delete names it. */
    path: string
    /** Sends the path to the bridge; it resolves when Rust has answered. */
    onDelete: (path: string) => Promise<void>
  } = $props()

  /** Whether an answer is on its way: a second click waits for it, never doubles. */
  let busy = $state(false)
  let cancel = $state<HTMLButtonElement | null>(null)

  // An open dialog starts with nothing on its way, and puts the focus on the way
  // out of it (the destructive button is never the one a stray Enter lands on).
  $effect(() => {
    if (open) {
      busy = false
      cancel?.focus()
    }
  })

  async function remove(): Promise<void> {
    if (busy) return
    busy = true
    await onDelete(path)
    open = false
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Portal>
    <Dialog.Overlay class="ovl" />
    <Dialog.Content class="dlg" onOpenAutoFocus={(event) => event.preventDefault()}>
      <Dialog.Title class="dt">{DELETE_TITLE}</Dialog.Title>
      <Dialog.Description class="why">{DELETE_BODY}</Dialog.Description>
      <code>{path}</code>
      <div class="acts">
        <button class="btn" type="button" bind:this={cancel} onclick={() => (open = false)}>
          {CANCEL}
        </button>
        <button class="btn d" type="button" disabled={busy} onclick={remove}>
          {DELETE_CONFIRM}
        </button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
