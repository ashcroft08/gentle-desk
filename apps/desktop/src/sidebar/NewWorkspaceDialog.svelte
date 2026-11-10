<script lang="ts">
  // The create-folder dialog, which no artboard draws. It asks for the folder
  // NAME only: Rust validates the name and then asks for the parent folder
  // (workspaces-design Q6), so nothing of those rules is repeated here; the one
  // rule this side owns is that an empty or blank name cannot be sent. The name
  // picker can stay open for minutes, so Create shows a busy state and a second
  // click while it runs does nothing. A refusal is not drawn here: the dialog
  // closes on the answer and the sidebar's notice line says what happened.
  import { Dialog } from 'bits-ui'
  import {
    CANCEL,
    CREATE_BUSY,
    CREATE_IDLE,
    NEW_FOLDER_HINT,
    NEW_FOLDER_LABEL,
    NEW_FOLDER_TITLE,
  } from './copy'

  let {
    open = $bindable(false),
    trigger = null,
    onCreate,
  }: {
    /** Whether the dialog is open. */
    open?: boolean
    /** The New workspace icon, which takes the focus back when the dialog closes. */
    trigger?: HTMLElement | null
    /** Sends the name to the bridge; it resolves when Rust has answered. */
    onCreate: (name: string) => Promise<void>
  } = $props()

  let typed = $state('')
  let busy = $state(false)
  let field = $state<HTMLInputElement | null>(null)

  const name = $derived(typed.trim())
  const ready = $derived(name !== '' && !busy)

  // A fresh dialog starts empty, and an open one puts the caret in the name.
  $effect(() => {
    if (!open) {
      typed = ''
      busy = false
    } else if (field !== null) {
      field.focus()
    }
  })

  async function create(): Promise<void> {
    if (!ready) return
    busy = true
    await onCreate(name)
    open = false
  }

  /** Escape and Cancel close; the focus goes back to the icon, not the menu item. */
  function closedFocus(event: Event): void {
    event.preventDefault()
    trigger?.focus()
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Portal>
    <Dialog.Overlay class="ovl" />
    <Dialog.Content
      class="dlg"
      onOpenAutoFocus={(event) => event.preventDefault()}
      onCloseAutoFocus={closedFocus}
    >
      <Dialog.Title class="dt">{NEW_FOLDER_TITLE}</Dialog.Title>
      <Dialog.Description class="why">{NEW_FOLDER_HINT}</Dialog.Description>
      <label for="new-folder-name">{NEW_FOLDER_LABEL}</label>
      <input
        id="new-folder-name"
        type="text"
        autocomplete="off"
        spellcheck="false"
        bind:this={field}
        bind:value={typed}
      />
      <div class="acts">
        <Dialog.Close class="btn" type="button">{CANCEL}</Dialog.Close>
        <button class="btn" type="button" disabled={!ready} aria-busy={busy} onclick={create}>
          {busy ? CREATE_BUSY : CREATE_IDLE}
        </button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
