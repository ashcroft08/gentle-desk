<script lang="ts">
  // The Settings dialog (settings-dialog S2-S6, L4; 0.1.12 S1-S7; 0.2.0 S2/S4):
  // a rail of seven sections on the left and the selected section's content in
  // the pane on the right. The rail carries the two controls the footer used to
  // hold - the theme cards and the home setting - then the 0.1.12 panes, then
  // Usage, then Language (0.2.0), then About: the app's name and the version
  // this run runs as. The controls themselves are the same components the footer
  // drew, with the same props and the same behavior: nothing here redesigns them,
  // it only gives them a calmer place and one section at a time.
  //
  // Profiles left this dialog in profiles-editor T2d (L21, L22): the maintainer
  // ruled that a Settings section made the profiles look global while they are
  // per chat and per worktree, so the chat's own Profile tab holds them and this
  // dialog names them nowhere. Nothing here reads a config file of its own but
  // the auth store's pane.
  //
  // The rail's own words are read from the language-keyed dictionary through the
  // reactive language (0.2.0, D4), so choosing a language in the Language section
  // redraws the titles and hints in this same tick: that is the visible proof
  // that a change needs no restart.
  //
  // The rail is a vertical tablist, the chat header's pattern: one entry is the
  // tab stop and the arrow keys, Home and End walk the rail, so the dialog opens
  // on Appearance and the focus starts there. Only the selected section is
  // drawn, so nothing off-screen can take a key, and a pane that reads the
  // machine (Account) only starts reading when it is the one on screen.
  //
  // The field above the rail searches the seven entries (the reference's "Find a
  // setting…", settings/navigation.rs): it filters on each entry's title, hint
  // and keywords, in both languages, and it never moves the section the pane
  // draws - a query that hides the open entry leaves the pane exactly as the
  // user left it. Escape in the field clears the query and nothing else.
  //
  // Closing sends nothing. Each section persists through the command it already
  // had, so Dismiss, Escape and the backdrop only close the card.
  //
  // ONE read per open (settings-snapshot): the card asks for the whole plain
  // data of every section once - `settings_snapshot`, gathered off the window's
  // thread - and each page draws from that one object instead of fetching for
  // itself. Before this, walking the rail re-ran the Account pane's per-provider
  // checks and the Usage pane's run of the sources on every visit; now a visit
  // costs nothing, and "Check again" asks for the same one load again (coalesced:
  // a burst of asks is one extra load, never one per ask). The push the dialog
  // already receives is what it draws until the answer lands, so no section is
  // ever blank while the load is on its way.
  import { Dialog } from 'bits-ui'
  import type {
    AuthView_Serialize as AuthView,
    ShellView_Serialize as ShellView,
    ThemeView,
    UpdatesView_Serialize as UpdatesView,
    UsageView_Serialize as UsageView,
  } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'
  import { language } from '../i18n/language.svelte'
  import HomeSetting from '../sidebar/HomeSetting.svelte'
  import AccountPane from './AccountPane.svelte'
  import LanguagePane from './LanguagePane.svelte'
  import ThemeCards from './ThemeCards.svelte'
  import UpdatesPane from './UpdatesPane.svelte'
  import UsagePane from './UsagePane.svelte'
  import { settingsCopy } from './copy'
  import { visibleSections } from './search'
  import { createSettingsRead } from './snapshot.svelte'
  import {
    DEFAULT_SECTION,
    PANEL_ID,
    TAB_IDS,
    sectionAfter,
    type Section,
  } from './sections'

  let {
    open = $bindable(false),
    theme,
    shell,
    version = null,
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
    /** The Account pane's names, until the one read of this open answers. */
    auth?: AuthView | null
    /** The Updates slice, until the one read of this open answers. */
    updates?: UpdatesView | null
    /** The Usage slice, until the one read of this open answers. */
    usage?: UsageView | null
    /** How many chats are open: the Updates pane's gate (S4). */
    openChats?: number
  } = $props()

  /** The section the pane draws: Appearance on open (L4). */
  let selected = $state<Section>(DEFAULT_SECTION)

  /**
   * What the search field holds. It narrows the RAIL and nothing else: the pane
   * keeps drawing `selected`, so a query that stops matching the open section
   * never moves what the user is reading (see `search.ts` for the match).
   */
  let query = $state('')

  /**
   * The one read of this open (settings-snapshot): the whole plain data of
   * every section, gathered off the window's thread, asked once when the card
   * opens. "Check again" asks the SAME read again, coalesced, so a burst of asks
   * costs one extra load (`snapshot.svelte.ts`).
   */
  const read = createSettingsRead()

  $effect(() => {
    if (!open) return
    return read.start()
  })

  /** Asks for the one read again: what every section's "Check again" calls. */
  function again(): void {
    read.again()
  }

  /**
   * The pane data: the one read when it has answered, and the push the dialog
   * already received before that. The theme, the home and the language are NOT
   * taken from here: the user changes all three from inside this card, so they
   * keep following the live slice (the snapshot carries them for the record).
   */
  const authView = $derived(read.snapshot?.auth ?? auth)
  const updatesView = $derived(read.snapshot?.updates ?? updates)
  const usageView = $derived(read.snapshot?.usage ?? usage)

  /**
   * Whether a load the USER asked for is in flight: the first load of the open
   * does not count, because until it answers the sections draw what the push
   * carried and no section has anything to re-read for itself.
   */
  const rechecking = $derived(read.busy && read.snapshot !== null)

  /** The rail's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(settingsCopy(language.current))

  /** The entries the rail draws: every one while the field is empty. */
  const drawn = $derived(visibleSections(copy, query))

  /**
   * The entry that takes the tab. The selected one keeps its place while the
   * rail still draws it - a search must not move the user's place in the rail -
   * and otherwise the first result takes it, since the panel is still whatever
   * the user opened and the rail has to stay reachable from the keyboard.
   */
  const tabStop = $derived(
    drawn.some((entry) => entry.id === selected) ? selected : (drawn[0]?.id ?? null),
  )

  /** The rail's own title per section: the entry's word is the pane's heading. */
  const title = $derived(copy.titles[selected])

  /**
   * The dialog opens where it always did: on the selected rail entry, not on the
   * first control the card happens to hold. The search field now sits above the
   * rail, and the primitive focuses the first tabbable element it finds, so the
   * focus it would place on the field is taken back here and given to the entry
   * the pane's own name points at.
   */
  function opened(event: Event): void {
    event.preventDefault()
    document.getElementById(TAB_IDS[selected])?.focus()
  }

  /** A key that walks the rail also selects the entry it lands on, and focuses it. */
  function keyed(event: KeyboardEvent, from: Section): void {
    const next = sectionAfter(event.key, from, drawn)
    if (next === null) return
    event.preventDefault()
    selected = next
    document.getElementById(TAB_IDS[next])?.focus()
  }

  /**
   * Escape in the field empties it and stops there: the field is a search, not a
   * way out of the dialog, so the key must not reach the primitive that closes
   * the card. With nothing to clear the key keeps its own meaning and closes.
   */
  function searchKeys(event: KeyboardEvent): void {
    if (event.key !== 'Escape' || query === '') return
    event.preventDefault()
    event.stopPropagation()
    query = ''
  }
</script>

<Dialog.Root bind:open>
  <Dialog.Portal>
    <Dialog.Overlay class="ovl" />
    <!-- A press on the backdrop does NOT close the dialog: closing is the Close
     button's job (and Escape's, which the primitive keeps for the keyboard).
     A mis-click outside must not throw away what the user was reading. -->
    <Dialog.Content class="dlg settings" interactOutsideBehavior="ignore" onOpenAutoFocus={opened}>
      <Dialog.Title class="dt">{copy.settings}</Dialog.Title>

      <div class="settings-body">
        <!-- The rail (L4) and the field above it: the field NARROWS the rail to
             the entries a query reaches and changes nothing else, so the pane
             below keeps the section the user opened. -->
        <div class="settings-side">
          <div class="settings-find">
            <Icon name="search" size={14} />
            <input
              type="search"
              aria-label={copy.searchLabel}
              placeholder={copy.searchPlaceholder}
              bind:value={query}
              onkeydown={searchKeys}
            />
          </div>
          {#if drawn.length > 0}
            <!-- One entry per section the query reaches, the selected one
                 highlighted. The entry's word is its name; the glyph is
                 decoration. -->
            <div
              class="settings-rail"
              role="tablist"
              aria-label={copy.railLabel}
              aria-orientation="vertical"
            >
              {#each drawn as entry (entry.id)}
                <button
                  type="button"
                  role="tab"
                  id={TAB_IDS[entry.id]}
                  aria-controls={PANEL_ID}
                  aria-selected={selected === entry.id}
                  tabindex={entry.id === tabStop ? 0 : -1}
                  onclick={() => (selected = entry.id)}
                  onkeydown={(event) => keyed(event, entry.id)}
                >
                  <Icon name={entry.icon} size={16} />
                  <span>{copy.titles[entry.id]}</span>
                </button>
              {/each}
            </div>
          {:else}
            <!-- One quiet line instead of an empty rail: the pane keeps the
                 section the user opened, and searching never moves it. -->
            <p class="settings-none">{copy.noMatches}</p>
          {/if}
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
              <Dialog.Description class="hint">{copy.hints[selected]}</Dialog.Description>

              {#if selected === 'appearance'}
                <ThemeCards {theme} />
              {:else if selected === 'home'}
                <p class="why">{copy.homeExplain}</p>
                {#if shell !== null}
                  <HomeSetting {shell} />
                {/if}
              {:else if selected === 'updates'}
                <UpdatesPane updates={updatesView} {openChats} refreshing={rechecking} onRefresh={again} />
              {:else if selected === 'account'}
                <AccountPane auth={authView} onRefresh={again} />
              {:else if selected === 'usage'}
                <UsagePane usage={usageView} refreshing={rechecking} onRefresh={again} />
              {:else if selected === 'language'}
                <!-- The app's own words (0.2.0, S2): the two languages, each named
                     in its own language, and the current one marked. -->
                <LanguagePane />
              {:else}
                <!-- About (S5): the name and the version this run runs as. What
                     is installed, and whether something newer is out, is the
                     Updates section's own card. -->
                <p class="about">
                  <span class="about-name">{copy.appName}</span>
                  <span class="about-version">
                    {version === null ? copy.versionUnknown : copy.version(version)}
                  </span>
                </p>
              {/if}
            </div>
          {/key}
        </div>
      </div>

      <div class="acts">
        <Dialog.Close class="btn" type="button">{copy.close}</Dialog.Close>
      </div>
    </Dialog.Content>
  </Dialog.Portal>
</Dialog.Root>
