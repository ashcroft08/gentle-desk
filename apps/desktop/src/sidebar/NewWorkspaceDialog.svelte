<script lang="ts">
  // The create-folder dialog, which no artboard draws. It asks for the folder
  // NAME only: Rust validates the name and then asks for the parent folder
  // (workspaces-design Q6), so nothing of those rules is repeated here; the one
  // rule this side owns is that an empty or blank name cannot be sent. The name
  // picker can stay open for minutes, so Create shows a busy state and a second
  // click while it runs does nothing. A refusal is not drawn here: the dialog
  // closes on the answer and the sidebar's notice line says what happened.
  import { Dialog } from 'bits-ui'
  import { language } from '../i18n/language.svelte'
  import { sidebarCopy } from './copy'

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

  /** The dialog's own words, in the language this run is set to (0.2.0, S3/D4). */
  const copy = $derived(sidebarCopy(language.current))

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
      <Dialog.Title class="dt">{copy.newFolderTitle}</Dialog.Title>
      <Dialog.Description class="why">{copy.newFolderHint}</Dialog.Description>
      <label for="new-folder-name">{copy.newFolderLabel}</label>
      <input
        id="new-folder-name"
        type="text"
        autocomplete="off"
        spellcheck="false"
        bind:this={field}
        bind:value={typed}
      />
      <div class="acts">
        <Dialog.Close class="btn" type="button">{copy.cancel}</Dialog.Close>
        <button class="btn" type="button" disabled={!ready} aria-busy={busy} onclick={create}>
          {busy ? copy.createBusy : copy.createIdle}
        </button>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
