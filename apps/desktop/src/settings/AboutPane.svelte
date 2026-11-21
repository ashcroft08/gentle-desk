<script lang="ts">
  // The About pane (settings-redesign S9, L5, L6, L7): the maintainer's "aquí lo
  // siento muy vacío" answered with FORM and with facts the app already has - its
  // own name and version, the two installed versions the Updates read already
  // brought into this dialog, when that read last ran (D8) and the repository.
  // Nothing here reads the machine: an invented number would be worse than the
  // one line this pane used to be.
  //
  // The words are reused, never reinvented: the app's name and version line are
  // `settings/copy.ts`'s, and the two row names, the "unknown" word and the
  // last-checked sentence are `settings/updates-copy.ts`'s - one word, one owner -
  // with the age itself from `home/ago.ts`, exactly the rule the Updates pane
  // words its line with.
  //
  // The what's-new button (D6, D7) is the pane's own click: it asks the shell for
  // the notes of the version this run RUNS as, on every run. When they are shown
  // (`Shown`) they travelled in `ViewState.whats_new` and the existing modal is
  // already up, so this pane draws NOTHING of its own - it renders no second
  // modal and hands the slice to nobody. A read with nothing to show answers
  // `Unavailable(line)`, already worded in the app's language by the shell, and
  // that line - one, quiet, under the button - is the pane's whole answer until
  // the next press clears it.
  //
  // The repository opens through the path the thread's links already use: the
  // click is cancelled so the WebView never navigates, the href is checked by
  // `forwardableHref`, and `openLink` asks the shell to open it.
  import { openLink, showWhatsNew } from '../bridge'
  import type { UpdatesView_Serialize as UpdatesView } from '../contract/bindings'
  import { ago, isJustNow } from '../home/ago'
  import { language } from '../i18n/language.svelte'
  import { forwardableHref } from '../thread/links'
  import { settingsCopy } from './copy'
  import { updatesCopy } from './updates-copy'

  /** The repository the README names, and the address the row draws. */
  const REPOSITORY = 'https://github.com/ashcroft08/gentle-desk'
  const REPOSITORY_LABEL = REPOSITORY.replace(/^https:\/\//, '')

  let {
    version = null,
    updates = null,
    onShown = () => {},
  }: {
    /** The version this run runs as, or null while the bridge named none (S5). */
    version?: string | null
    /** The Updates slice the dialog already holds: the same one read, no new one. */
    updates?: UpdatesView | null
    /**
     * Called once the notes are on screen. The what's-new window's backdrop sits
     * under this dialog's card (both share the app's overlay layers), so the dialog
     * closes itself rather than stand undimmed behind the notes.
     */
    onShown?: () => void
  } = $props()

  /** The pane's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(settingsCopy(language.current))
  /** The Updates pane's words: the row names and the "unknown" word are its own. */
  const words = $derived(updatesCopy(language.current))

  /** The installed versions the Updates read already reported, or null. */
  const shellVersion = $derived(updates?.shell?.current ?? null)
  const piVersion = $derived(updates?.pi?.current ?? null)

  /** The one quiet line a read that had nothing to show answered, or none. */
  let unavailable = $state<string | null>(null)
  /** Whether that read is in flight: the button waits instead of firing twice. */
  let asking = $state(false)

  /**
   * When that read last ran (D8): the same age rule, composed the way the Updates
   * pane composes it - once, when the line is drawn, never on a ticker.
   */
  const checkedLine = $derived.by(() => {
    const ms = updates?.checked_ms ?? null
    if (ms === null) return words.checkedNever
    const now = Date.now()
    return words.checkedAt(ago(language.current, now, ms), isJustNow(now, ms))
  })

  /**
   * Asks for the running version's notes. `Shown` needs nothing from here: the
   * modal opens over the pushed slice. Every other answer is the one quiet line,
   * and a new press clears the old line before the answer can arrive.
   */
  async function whatsNew(): Promise<void> {
    if (asking) return
    unavailable = null
    asking = true
    try {
      const answer = await showWhatsNew()
      if (answer?.type === 'Unavailable') unavailable = answer.data
      else if (answer?.type === 'Shown') onShown()
    } finally {
      asking = false
    }
  }

  /** Opens the repository in the system browser, never in the window. */
  function repository(event: MouseEvent): void {
    event.preventDefault()
    const href = forwardableHref(REPOSITORY)
    if (href !== null) void openLink(href)
  }
</script>

<div class="atitle">
  <span class="aname">{copy.appName}</span>
  <span class="aver">{version === null ? copy.versionUnknown : copy.version(version)}</span>
</div>

<div class="agroup">
  <dl class="afacts">
    <div class="afact">
      <dt>{words.shellLabel}</dt>
      <dd class:aunknown={shellVersion === null}>{shellVersion ?? words.unknownVersion}</dd>
    </div>
    <div class="afact">
      <dt>{words.piLabel}</dt>
      <dd class:aunknown={piVersion === null}>{piVersion ?? words.unknownVersion}</dd>
    </div>
    <div class="afact">
      <dt>{copy.repository}</dt>
      <dd>
        <a class="alink" href={REPOSITORY} onclick={repository}>{REPOSITORY_LABEL}</a>
      </dd>
    </div>
  </dl>
  <!-- D8: when the same read last ran, in that pane's own sentence. -->
  <p class="acheck">{checkedLine}</p>
</div>

<!-- D7: the button alone, with no note under it. It opens the window the app
     shows after it updates, and the one line a read that could not show the
     notes answers with is the only thing this pane draws of its own. -->
<div class="agroup">
  <button class="btn" type="button" disabled={asking} onclick={whatsNew}>{copy.whatsNew}</button>
  {#if unavailable !== null}
    <p class="quiet" role="status">{unavailable}</p>
  {/if}
</div>
