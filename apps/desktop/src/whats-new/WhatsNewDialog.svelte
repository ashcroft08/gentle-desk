<script lang="ts">
  // The what's-new modal (T2, S4; design/whats-new-modal 04): the one thing a run
  // that follows an update says about the version it now runs. Rust pushes it as
  // `ViewState.whats_new` with the run's OWN version, the version the config held
  // before, and the release body as markdown; this side draws it through the
  // thread's sanitized markdown pipeline (no script, no image, links go to the
  // shell).
  //
  // The body is released content, never rewritten here (D7), but it usually
  // carries the same news twice - English, then Spanish under a `Novedades en
  // X.Y.Z` line. Artboard D draws it as one panel per language: the body is split
  // at that marker (`split.ts`) and the tab of the language the app holds is the
  // one it opens on. The reader can switch to the other reading, and that choice
  // lives only here - nothing is stored, nothing persists. A body with no marker
  // draws exactly what the modal drew before the tabs: one panel, whole body.
  //
  // Dismiss, Escape and a backdrop click all mean the same: put the modal away
  // and tell the shell (`storeLastRunVersion`), which stores the version and
  // clears the slice. That ONE call per version is the whole of "once per
  // version" here: the notes keep arriving until the push that clears them
  // lands, so a close is remembered by the version it already sent and a second
  // Escape sends nothing. No other persistence lives on this side.
  import { Dialog } from 'bits-ui'
  import { storeLastRunVersion } from '../bridge'
  import type { LanguageView, WhatsNewView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import Markdown from '../thread/Markdown.svelte'
  import { createMarkdownCache } from '../thread/markdown'
  import { sinceLabel, whatsNewCopy } from './copy'
  import { splitNotes } from './split'
  import { LANGUAGES, PANEL_ID, TAB_IDS, tabAfter } from './tabs'

  const markdown = createMarkdownCache()

  let { whatsNew }: { whatsNew: WhatsNewView | null } = $props()

  /** The modal's own words in the language the app holds (0.2.0, S3/D4). */
  const copy = $derived(whatsNewCopy(language.current))

  /** Whether the modal stands; the slice says which version, this says if drawn. */
  let open = $state(false)
  /** The version this dialog has already sent the dismiss for, if any. */
  let sent = $state<string | null>(null)
  /**
   * The tab the reader chose here, alongside the version they chose it in: the
   * choice lives only in the modal, and only for the release it was made in (a
   * push that redraws the same version keeps it; a new version starts fresh).
   */
  let picked = $state<{ version: string; language: LanguageView } | null>(null)
  let dismiss = $state<HTMLButtonElement | null>(null)

  /** The two halves of the body, or `null` while it names no second language. */
  const split = $derived(whatsNew === null ? null : splitNotes(whatsNew.notes))

  /** The tab being read: the reader's choice, else the language the app holds. */
  const chosen = $derived<LanguageView>(
    picked !== null && picked.version === whatsNew?.version ? picked.language : language.current,
  )

  /** What the one panel draws: the whole body, or the chosen language's half. */
  const panel = $derived(
    split === null
      ? { key: 'whats-new', text: whatsNew?.notes ?? '' }
      : { key: `whats-new-${chosen}`, text: split[chosen] },
  )

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

  /** The reader's choice of reading, remembered for the version it was made in. */
  function pick(next: LanguageView): void {
    if (whatsNew === null) return
    picked = { version: whatsNew.version, language: next }
  }

  /** A key that walks the tabs also takes the reading and the focus with it. */
  function keyed(event: KeyboardEvent, from: LanguageView): void {
    const next = tabAfter(event.key, from)
    if (next === null) return
    event.preventDefault()
    pick(next)
    document.getElementById(TAB_IDS[next])?.focus()
  }

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
            {copy.heading} <span class="ver">{whatsNew.version}</span>
          </Dialog.Title>
          {#if sinceLabel(whatsNew.version, whatsNew.previous_version, copy)}
            <span class="since"
              >{sinceLabel(whatsNew.version, whatsNew.previous_version, copy)}</span
            >
          {/if}
        </header>
        {#if split !== null}
          <!-- The language tabs (artboard D): one reading per tab, styled as
               folder tabs so the selected one opens into the notes below. Each
               language names itself, whatever the app's own language is. -->
          <div class="tabs" role="tablist" aria-label={copy.tabsLabel}>
            {#each LANGUAGES as tab (tab.id)}
              <button
                type="button"
                role="tab"
                id={TAB_IDS[tab.id]}
                aria-controls={PANEL_ID}
                aria-selected={chosen === tab.id}
                tabindex={chosen === tab.id ? 0 : -1}
                lang={tab.id}
                onclick={() => pick(tab.id)}
                onkeydown={(event) => keyed(event, tab.id)}>{tab.name}</button
              >
            {/each}
          </div>
        {/if}
        <!-- The notes are the dialog's description in both readings: one panel,
             which the tabs open and name while the body carries two languages. -->
        <Dialog.Description
          class="notes"
          id={split === null ? undefined : PANEL_ID}
          role={split === null ? undefined : 'tabpanel'}
          aria-labelledby={split === null ? undefined : TAB_IDS[chosen]}
        >
          <Markdown html={markdown.htmlFor(panel.key, panel.text)} />
        </Dialog.Description>
        <div class="acts">
          <span class="hint">{copy.escHint}</span>
          <button class="btn primary" type="button" bind:this={dismiss} onclick={putAway}>
            {copy.dismiss}
          </button>
        </div>
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
{/if}
