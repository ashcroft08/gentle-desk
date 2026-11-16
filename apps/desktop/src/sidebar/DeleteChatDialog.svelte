<script lang="ts">
  // The delete confirm dialog (T3, S4, D2(b)), upgraded to the removal dialog's
  // anatomy (workspace-removal S3): the question is the title, one consequence
  // line carries its scope in bold, the session path is plain monospaced text
  // with its middle elided (never a boxed `code` card), and the actions are
  // right-aligned with the destructive one last.
  //
  // The file's own `<timestamp>_<id>.jsonl` name stays reachable from the row's
  // menu (`Copy file path`), where it is asked for instead of printed at everyone.
  //
  // Cancel and Escape close it and send nothing. Delete sends the path ONCE and
  // the dialog closes when Rust answers, so a second click cannot remove another
  // file. The focus opens on Cancel, the answer that changes nothing. A refusal
  // is not drawn here: a chat that is working asks the user to stop it first, and
  // the sidebar's notice line says so once the dialog is gone.
  import { Dialog } from 'bits-ui'
  import { language } from '../i18n/language.svelte'
  import { sidebarCopy } from './copy'
  import { elideMiddle } from './removal'

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

  /** The dialog's own words, in the language this run is set to (0.2.0, S3/D4). */
  const copy = $derived(sidebarCopy(language.current))

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
      <Dialog.Title class="dt">{copy.deleteTitle}</Dialog.Title>
      <p class="path mono">{elideMiddle(path)}</p>
      <Dialog.Description class="why"
        >{copy.deleteLine.before}<b>{copy.deleteLine.scope}</b>{copy.deleteLine.after}</Dialog.Description
      >
      <div class="acts">
        <button class="btn" type="button" bind:this={cancel} onclick={() => (open = false)}>
          {copy.cancel}
        </button>
        <button class="btn d" type="button" disabled={busy} onclick={remove}>
          {copy.deleteConfirm}
        </button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
