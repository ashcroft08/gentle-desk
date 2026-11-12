<script lang="ts">
  // The Settings dialog (settings-dialog S2-S6): the two controls the footer used
  // to carry - the theme picker and the home setting - each in a section of its
  // own, with the explanation the welcome screen gives for the two homes, then
  // About and updates: the app's name, the version this run runs as, and the
  // manual check with its answer drawn in place. Both controls are the same
  // components the footer drew, with the same props and the same behavior:
  // nothing here redesigns them, it only gives them a calmer place.
  //
  // Closing sends nothing. Each section persists through the command it already
  // had, so Dismiss, Escape and the backdrop only close the card. The focus
  // starts on the first control the dialog draws (the theme button) and stays
  // inside while it is open, which is bits-ui's default: no onOpenAutoFocus here
  // hands the focus to the first thing, and the card traps it (S6).
  //
  // The check is the one the startup runs (`update/plugin`), injected here so a
  // test drives it with no network; it only REPORTS, never installs (S6).
  import { Dialog } from 'bits-ui'
  import type { ShellView_Serialize as ShellView, ThemeView } from '../contract/bindings'
  import HomeSetting from '../sidebar/HomeSetting.svelte'
  import ThemePicker from '../sidebar/ThemePicker.svelte'
  import { checkForUpdate, type CheckResult } from '../update/plugin'
  import { checking, notChecked, type CheckState } from './updates'
  import {
    ABOUT_TITLE,
    APP_NAME,
    APPEARANCE_TITLE,
    CHECK_AVAILABLE,
    CHECK_FAILED,
    CHECK_NOW,
    CHECKING,
    CLOSE,
    HOME_EXPLAIN,
    HOME_TITLE,
    HINT,
    SETTINGS,
    UP_TO_DATE,
    VERSION,
    VERSION_UNKNOWN,
  } from './copy'

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
      <Dialog.Description class="why">{HINT}</Dialog.Description>

      <section class="setting" aria-labelledby="settings-appearance">
        <h3 id="settings-appearance">{APPEARANCE_TITLE}</h3>
        <ThemePicker {theme} />
      </section>

      <section class="setting" aria-labelledby="settings-home">
        <h3 id="settings-home">{HOME_TITLE}</h3>
        <p class="why">{HOME_EXPLAIN}</p>
        {#if shell !== null}
          <HomeSetting {shell} />
        {/if}
      </section>

      <!-- About and updates (S5, S6): the name, the version this run runs as,
           and the one check button with its answer drawn under it. The check
           only reports: installing stays the update notice's own flow, so a
           found release never installs itself here. -->
      <section class="setting" aria-labelledby="settings-about">
        <h3 id="settings-about">{ABOUT_TITLE}</h3>
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
      </section>

      <div class="acts">
        <Dialog.Close class="btn" type="button">{CLOSE}</Dialog.Close>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
