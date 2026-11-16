<script lang="ts">
  // The removal dialog (S1, S3; design/workspace-removal-dialog/02-evidence-disclosure.html):
  // artboard B mapped onto the house's bits-ui dialog and its own tokens.
  //
  // The question is the title, one consequence line carries its scope in bold,
  // the workspace's path is plain monospaced text with its middle elided, and the
  // evidence sits under one quiet disclosure - never an inner card, never a boxed
  // path. Nothing about the count or the size is drawn before the offer answers
  // (`workspace_removal_offer`): a number that had not arrived yet would be a
  // claim about what the red button does.
  //
  // Cancel and Escape close it and send nothing, and the focus opens on Cancel.
  // The destructive answer is the LAST button, red and labelled with its scope; a
  // workspace with no chat only forgets its bookmark, so that state's answer is a
  // plain Remove - nothing is destroyed, so nothing is red (S1).
  import { Dialog } from 'bits-ui'
  import type { WorkspaceRemovalView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { sidebarCopy } from './copy'
  import { chatCount, chatScope, elideMiddle, removalOffer } from './removal'

  let {
    open = $bindable(false),
    path,
    titles = [],
    onRemove,
  }: {
    /** Whether the dialog is open. */
    open?: boolean
    /** The workspace the user picked: the identity this dialog names and sends. */
    path: string
    /** The titles of that workspace's chats, as the sidebar's page holds them. */
    titles?: string[]
    /** Removes the workspace; it resolves when Rust has answered. */
    onRemove: (path: string) => Promise<void>
  } = $props()

  /** The offer Rust answered, or `null` while none is known. */
  let offer = $state<WorkspaceRemovalView | null>(null)
  /** Whether an answer is on its way: a second click waits for it, never doubles. */
  let busy = $state(false)
  let cancel = $state<HTMLButtonElement | null>(null)

  /** The dialog's own words, in the language this run is set to (S7). */
  const copy = $derived(sidebarCopy(language.current))
  const scope = $derived(offer === null ? '' : chatScope(offer.chats, offer.bytes))
  const line = $derived(
    offer === null
      ? null
      : offer.chats === 0
        ? copy.removeKeepLine
        : copy.removeChatsLine(scope),
  )
  // The disclosure lists every chat the offer counts; a page that holds fewer of
  // their titles is never drawn as if it listed them all.
  const listed = $derived(offer !== null && offer.chats > 0 && titles.length === offer.chats)

  // The offer is read when the dialog opens and dropped when it closes, so a
  // second open asks Rust again instead of repeating a count that may have moved.
  $effect(() => {
    if (!open) {
      offer = null
      busy = false
      return
    }
    let live = true
    void removalOffer(path).then((view) => {
      if (live) offer = view
    })
    return () => {
      live = false
    }
  })

  // The focus opens on the answer that changes nothing, never on the red one.
  $effect(() => {
    if (open) cancel?.focus()
  })

  async function remove(): Promise<void> {
    if (busy || offer === null) return
    busy = true
    await onRemove(path)
    open = false
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Portal>
    <Dialog.Overlay class="ovl" />
    <Dialog.Content class="dlg" onOpenAutoFocus={(event) => event.preventDefault()}>
      <Dialog.Title class="dt">{copy.removeTitle}</Dialog.Title>
      <p class="path mono">{elideMiddle(path)}</p>
      {#if offer !== null && line !== null}
        <Dialog.Description class="why"
          >{line.before}<b>{line.scope}</b>{line.after}</Dialog.Description
        >
        {#if listed}
          <details>
            <summary>{copy.showChats(scope)}</summary>
            <ul class="chats">
              {#each titles as title (title)}
                <li>{title}</li>
              {/each}
            </ul>
          </details>
        {/if}
        <div class="acts">
          <button class="btn" type="button" bind:this={cancel} onclick={() => (open = false)}>
            {copy.cancel}
          </button>
          {#if offer.chats === 0}
            <button class="btn" type="button" disabled={busy} onclick={remove}>
              {copy.removeKeepConfirm}
            </button>
          {:else}
            <button class="btn d" type="button" disabled={busy} onclick={remove}>
              {copy.removeChatsConfirm(chatCount(offer.chats))}
            </button>
          {/if}
        </div>
      {/if}
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
