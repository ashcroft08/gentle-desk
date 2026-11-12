<script lang="ts">
  // The Settings dialog (settings-dialog S2-S6, L4): a rail of three sections on
  // the left and the selected section's content in the pane on the right. The
  // rail carries the two controls the footer used to hold - the theme picker and
  // the home setting - plus About and updates: the app's name, the version this
  // run runs as, and the manual check with its answer drawn in place. The
  // controls themselves are the same components the footer drew, with the same
  // props and the same behavior: nothing here redesigns them, it only gives them
  // a calmer place and one section at a time.
  //
  // The rail is a vertical tablist, the chat header's pattern: one entry is the
  // tab stop and the arrow keys, Home and End walk the rail, so the dialog opens
  // on Appearance and the focus starts there. Only the selected section is
  // drawn, so nothing off-screen can take a key. Escape, the backdrop and Close
  // still close the card, and the focus stays inside while it is open.
  //
  // Closing sends nothing. Each section persists through the command it already
  // had, so Dismiss, Escape and the backdrop only close the card.
  //
  // The check is the one the startup runs (`update/plugin`), injected here so a
  // test drives it with no network; it only REPORTS, never installs (S6).
  import { Dialog } from 'bits-ui'
  import type { ShellView_Serialize as ShellView, ThemeView } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'
  import HomeSetting from '../sidebar/HomeSetting.svelte'
  import ThemePicker from '../sidebar/ThemePicker.svelte'
  import { checkForUpdate, type CheckResult } from '../update/plugin'
  import {
    APP_NAME,
    CHECK_AVAILABLE,
    CHECK_FAILED,
    CHECK_NOW,
    CHECKING,
    CLOSE,
    HOME_EXPLAIN,
    SETTINGS,
    UP_TO_DATE,
    VERSION,
    VERSION_UNKNOWN,
  } from './copy'
  import {
    DEFAULT_SECTION,
    PANEL_ID,
    RAIL_LABEL,
    SECTIONS,
    SECTION_HINTS,
    SECTION_TITLES,
    TAB_IDS,
    sectionAfter,
    type Section,
  } from './sections'
  import { checking, notChecked, type CheckState } from './updates'

  let {
    open = $bindable(false),
    theme,
    shell,
    version = null,
    checkUpdate = checkForUpdate,
  }: {
    /** Whether the dialog is open. */
    open?: boolean
    /** The theme the Appearance section draws and can change. */
    theme: ThemeView
    /** The shell's home, or null while the first view is still on its way. */
    shell: ShellView | null
    /** The version this run runs as (S5), or null while the bridge named none. */
    version?: string | null
    /** The check the Updates section runs on demand (S6): the startup one. */
    checkUpdate?: () => Promise<CheckResult>
  } = $props()

  /** The section the pane draws: Appearance on open (L4). */
  let selected = $state<Section>(DEFAULT_SECTION)

  /** The check as the section draws it: nothing asked, in flight, or the answer. */
  let check = $state<CheckState>(notChecked())

  /** What the result line says for the state the check is in, or '' for none. */
  function line(state: CheckState): string {
    if (state.kind === 'checking') return CHECKING
    if (state.kind === 'available') return CHECK_AVAILABLE(state.version)
    if (state.kind === 'current') return UP_TO_DATE
    if (state.kind === 'failed') return CHECK_FAILED(state.message)
    return ''
  }

  const result = $derived(line(check))

  /** The rail's own title per section: the entry's word is the pane's heading. */
  const title = $derived(SECTION_TITLES[selected])

  /** A key that walks the rail also selects the entry it lands on, and focuses it. */
  function keyed(event: KeyboardEvent, from: Section): void {
    const next = sectionAfter(event.key, from)
    if (next === null) return
    event.preventDefault()
    selected = next
    document.getElementById(TAB_IDS[next])?.focus()
  }

  /**
   * Asks the updater once and draws what it answers (S6). A press while the
   * check runs asks nothing more, so one press is one request.
   */
  async function checkNow(): Promise<void> {
    if (check.kind === 'checking') return
    check = checking()
    check = await checkUpdate()
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Portal>
    <Dialog.Overlay class="ovl" />
    <Dialog.Content class="dlg settings">
      <Dialog.Title class="dt">{SETTINGS}</Dialog.Title>

      <div class="settings-body">
        <!-- The rail (L4): one entry per section, the selected one highlighted.
             The entry's word is its name; the glyph is decoration. -->
        <div class="settings-rail" role="tablist" aria-label={RAIL_LABEL} aria-orientation="vertical">
          {#each SECTIONS as entry (entry.id)}
            <button
              type="button"
              role="tab"
              id={TAB_IDS[entry.id]}
              aria-controls={PANEL_ID}
              aria-selected={selected === entry.id}
              tabindex={selected === entry.id ? 0 : -1}
              onclick={() => (selected = entry.id)}
              onkeydown={(event) => keyed(event, entry.id)}
            >
              <Icon name={entry.icon} size={16} />
              <span>{entry.title}</span>
            </button>
          {/each}
        </div>

        <!-- The pane: the selected section only. Its hint is also the dialog's
             description, so the card is described by what it is showing. -->
        <div class="settings-pane" role="tabpanel" id={PANEL_ID} aria-labelledby={TAB_IDS[selected]}>
          <h3>{title}</h3>
          <Dialog.Description class="hint">{SECTION_HINTS[selected]}</Dialog.Description>

          {#if selected === 'appearance'}
            <ThemePicker {theme} />
          {:else if selected === 'home'}
            <p class="why">{HOME_EXPLAIN}</p>
            {#if shell !== null}
              <HomeSetting {shell} />
            {/if}
          {:else}
            <!-- About and updates (S5, S6): the name, the version this run runs
                 as, and the one check button with its answer drawn under it. The
                 check only reports: installing stays the update notice's own
                 flow, so a found release never installs itself here. -->
            <p class="about">
              <span class="about-name">{APP_NAME}</span>
              <span class="about-version">
                {version === null ? VERSION_UNKNOWN : VERSION(version)}
              </span>
            </p>
            <div class="check">
              <button
                type="button"
                class="btn"
                disabled={check.kind === 'checking'}
                onclick={() => void checkNow()}
              >
                {CHECK_NOW}
              </button>
              {#if check.kind !== 'idle'}
                <p class="why" role="status">{result}</p>
              {/if}
            </div>
          {/if}
        </div>
      </div>

      <div class="acts">
        <Dialog.Close class="btn" type="button">{CLOSE}</Dialog.Close>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
