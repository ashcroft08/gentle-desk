<script lang="ts">
  // The what's-new modal (T2, S4): the one thing a run that follows an update
  // says about the version it now runs. Rust pushes it as `ViewState.whats_new`
  // with the run's OWN version and the release body as plain text; this side
  // draws that text, one paragraph per blank-line chunk, and nothing else.
  //
  // Dismiss, Escape and a backdrop click all mean the same: put the modal away
  // and tell the shell (`storeLastRunVersion`), which stores the version and
  // clears the slice. That ONE call per version is the whole of "once per
  // version" here: the notes keep arriving until the push that clears them
  // lands, so a close is remembered by the version it already sent and a second
  // Escape sends nothing. No other persistence lives on this side.
  import { Dialog } from 'bits-ui'
  import { storeLastRunVersion } from '../bridge'
  import type { WhatsNewView } from '../contract/bindings'
  import { DISMISS, WHATS_NEW_TITLE, paragraphsOf } from './copy'

  let { whatsNew }: { whatsNew: WhatsNewView | null } = $props()

  /** Whether the modal stands; the slice says which version, this says if drawn. */
  let open = $state(false)
  /** The version this dialog has already sent the dismiss for, if any. */
  let sent = $state<string | null>(null)
  let dismiss = $state<HTMLButtonElement | null>(null)

  // A push with notes opens the modal; the push that clears the slice closes
  // it, so it is never left standing over a version it already sent.
  $effect(() => {
    if (whatsNew === null) {
      open = false
      return
    }
    if (whatsNew.version === sent) return
    open = true
  })

  // The modal has one answer, and the focus opens on it, so Enter and the
  // keyboard land on the way out. The focus never goes back to the window
  // behind: bits-ui moves it to the start of the page when the modal is gone.
  $effect(() => {
    if (open) dismiss?.focus()
  })

  function putAway(): void {
    if (whatsNew === null || sent === whatsNew.version) return
    sent = whatsNew.version
    open = false
    void storeLastRunVersion()
  }

  /** Escape and the backdrop close through the primitive; both send once. */
  function changed(next: boolean): void {
    if (!next) putAway()
  }
</script>

{#if whatsNew !== null}
  <Dialog.Root bind:open onOpenChange={changed}>
    <Dialog.Portal>
      <Dialog.Overlay class="ovl" />
      <Dialog.Content class="dlg" onOpenAutoFocus={(event) => event.preventDefault()}>
        <Dialog.Title class="dt">{WHATS_NEW_TITLE(whatsNew.version)}</Dialog.Title>
        <Dialog.Description class="notes">
          {#each paragraphsOf(whatsNew.notes) as paragraph}
            <p>{paragraph}</p>
          {/each}
        </Dialog.Description>
        <div class="acts">
          <button class="btn" type="button" bind:this={dismiss} onclick={putAway}>
            {DISMISS}
          </button>
        </div>
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
{/if}
