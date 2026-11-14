<script lang="ts">
  // The Settings dialog (settings-dialog S2-S6, L4; 0.1.12 S1-S7): a rail of
  // seven sections on the left and the selected section's content in the pane on
  // the right. The rail carries the two controls the footer used to hold - the
  // theme picker and the home setting - then the 0.1.12 panes, then Usage, then
  // About: the app's name and the version this run runs as. The controls themselves are the
  // same components the footer drew, with the same props and the same behavior:
  // nothing here redesigns them, it only gives them a calmer place and one
  // section at a time.
  //
  // The rail is a vertical tablist, the chat header's pattern: one entry is the
  // tab stop and the arrow keys, Home and End walk the rail, so the dialog opens
  // on Appearance and the focus starts there. Only the selected section is
  // drawn, so nothing off-screen can take a key, and a pane that reads the
  // machine (Profiles) only starts reading when it is the one on screen.
  //
  // Closing sends nothing. Each section persists through the command it already
  // had, so Dismiss, Escape and the backdrop only close the card.
  import { Dialog } from 'bits-ui'
  import type {
    AuthView_Serialize as AuthView,
    ProfilesView_Serialize as ProfilesView,
    ShellView_Serialize as ShellView,
    ThemeView,
    UpdatesView_Serialize as UpdatesView,
    UsageView_Serialize as UsageView,
  } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'
  import HomeSetting from '../sidebar/HomeSetting.svelte'
  import ThemePicker from '../sidebar/ThemePicker.svelte'
  import AccountPane from './AccountPane.svelte'
  import ProfilesPane from './ProfilesPane.svelte'
  import UpdatesPane from './UpdatesPane.svelte'
  import UsagePane from './UsagePane.svelte'
  import { APP_NAME, CLOSE, HOME_EXPLAIN, SETTINGS, VERSION, VERSION_UNKNOWN } from './copy'
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

  let {
    open = $bindable(false),
    theme,
    shell,
    version = null,
    profiles = null,
    auth = null,
    updates = null,
    usage = null,
    openChats = 0,
  }: {
    /** Whether the dialog is open. */
    open?: boolean
    /** The theme the Appearance section draws and can change. */
    theme: ThemeView
    /** The shell's home, or null while the first view is still on its way. */
    shell: ShellView | null
    /** The version this run runs as (S5), or null while the bridge named none. */
    version?: string | null
    /** The Profiles pane's slice (0.1.12, S1), or null before the first push. */
    profiles?: ProfilesView | null
    /** The Account pane's slice (0.1.12, S1), or null before the first push. */
    auth?: AuthView | null
    /** The Updates pane's slice (0.1.12, S2), or null before the first check. */
    updates?: UpdatesView | null
    /** The Usage pane's slice, or null before the first read. */
    usage?: UsageView | null
    /** How many chats are open: the Updates pane's gate (S4). */
    openChats?: number
  } = $props()

  /** The section the pane draws: Appearance on open (L4). */
  let selected = $state<Section>(DEFAULT_SECTION)

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
</script>

<Dialog.Root bind:open>
  <Dialog.Portal>
    <Dialog.Overlay class="ovl" />
    <!-- A press on the backdrop does NOT close the dialog: closing is the Close
     button's job (and Escape's, which the primitive keeps for the keyboard).
     A mis-click outside must not throw away what the user was reading. -->
    <Dialog.Content class="dlg settings" interactOutsideBehavior="ignore">
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
          <!-- Keyed on the section, so the body fades in on each switch: the
               swap is a calm step, never a jump. That fade is all the wrapper
               does - the section's content is the same component as before. -->
          {#key selected}
            <div class="pane-body">
              <h3>{title}</h3>
              <Dialog.Description class="hint">{SECTION_HINTS[selected]}</Dialog.Description>

              {#if selected === 'appearance'}
                <ThemePicker {theme} />
              {:else if selected === 'home'}
                <p class="why">{HOME_EXPLAIN}</p>
                {#if shell !== null}
                  <HomeSetting {shell} />
                {/if}
              {:else if selected === 'updates'}
                <UpdatesPane {updates} {openChats} />
              {:else if selected === 'profiles'}
                <ProfilesPane {profiles} />
              {:else if selected === 'account'}
                <AccountPane {auth} />
              {:else if selected === 'usage'}
                <UsagePane {usage} />
              {:else}
                <!-- About (S5): the name and the version this run runs as. What
                     is installed, and whether something newer is out, is the
                     Updates section's own card. -->
                <p class="about">
                  <span class="about-name">{APP_NAME}</span>
                  <span class="about-version">
                    {version === null ? VERSION_UNKNOWN : VERSION(version)}
                  </span>
                </p>
              {/if}
            </div>
          {/key}
        </div>
      </div>

      <div class="acts">
        <Dialog.Close class="btn" type="button">{CLOSE}</Dialog.Close>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
