<script lang="ts">
  // The Settings dialog (settings-dialog S2-S4): the two controls the footer used
  // to carry - the theme picker and the home setting - each in a section of its
  // own, with the explanation the welcome screen gives for the two homes, and the
  // shell of the About and updates section T2 fills in. Both controls are the
  // same components the footer drew, with the same props and the same behavior:
  // nothing here redesigns them, it only gives them a calmer place.
  //
  // Closing sends nothing. Each section persists through the command it already
  // had, so Dismiss, Escape and the backdrop only close the card. The focus
  // starts on the first control the dialog draws (the theme button) and stays
  // inside while it is open, which is bits-ui's default: no onOpenAutoFocus here
  // hands the focus to the first thing, and the card traps it (S6).
  import { Dialog } from 'bits-ui'
  import type { ShellView_Serialize as ShellView, ThemeView } from '../contract/bindings'
  import HomeSetting from '../sidebar/HomeSetting.svelte'
  import ThemePicker from '../sidebar/ThemePicker.svelte'
  import {
    ABOUT_TITLE,
    APPEARANCE_TITLE,
    CLOSE,
    HOME_EXPLAIN,
    HOME_TITLE,
    HINT,
    SETTINGS,
  } from './copy'

  let {
    open = $bindable(false),
    theme,
    shell,
  }: {
    /** Whether the dialog is open. */
    open?: boolean
    /** The theme the Appearance section draws and can change. */
    theme: ThemeView
    /** The shell's home, or null while the first view is still on its way. */
    shell: ShellView | null
  } = $props()
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

      <!-- The About and the Updates fields are T2's: the section keeps the title
           the design gives it, so the dialog's order is already the full one. -->
      <section class="setting" aria-labelledby="settings-about">
        <h3 id="settings-about">{ABOUT_TITLE}</h3>
      </section>

      <div class="acts">
        <Dialog.Close class="btn" type="button">{CLOSE}</Dialog.Close>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
