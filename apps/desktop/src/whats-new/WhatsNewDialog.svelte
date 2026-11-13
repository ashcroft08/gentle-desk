<script lang="ts">
  // The what's-new modal (T2, S4): the one thing a run that follows an update
  // says about the version it now runs. Rust pushes it as `ViewState.whats_new`
  // with the run's OWN version, the version the config held before, and the
  // release body as markdown; this side draws it through the thread's sanitized
  // markdown pipeline (no script, no image, links go to the shell).
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
  import Markdown from '../thread/Markdown.svelte'
  import { createMarkdownCache } from '../thread/markdown'
  import { DISMISS, ESC_HINT, HEADING, sinceLabel } from './copy'

  const markdown = createMarkdownCache()

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
      <Dialog.Content class="dlg whatsnew" onOpenAutoFocus={(event) => event.preventDefault()}>
        <header>
          <Dialog.Title class="dt">
            {HEADING} <span class="ver">{whatsNew.version}</span>
          </Dialog.Title>
          {#if sinceLabel(whatsNew.version, whatsNew.previous_version)}
            <span class="since">{sinceLabel(whatsNew.version, whatsNew.previous_version)}</span>
          {/if}
        </header>
        <Dialog.Description class="notes">
          <Markdown html={markdown.htmlFor('whats-new', whatsNew.notes)} />
        </Dialog.Description>
        <div class="acts">
          <span class="hint">{ESC_HINT}</span>
          <button class="btn primary" type="button" bind:this={dismiss} onclick={putAway}>
            {DISMISS}
          </button>
        </div>
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
{/if}
