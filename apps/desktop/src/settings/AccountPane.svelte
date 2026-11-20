<script lang="ts">
  // The Account pane (0.1.12, S1, T3, L11): where the shell's authentication
  // stands, one row per provider the shell's own store lists. NAMES only - the
  // shell's store is read for its top-level keys and nothing else travels up -
  // and every row's state is the text of its own check's answer, never a colour
  // on its own.
  //
  // The terminal does the work: login and logout are the shell's commands, so
  // each button opens a window on it, and this pane only shows where the user
  // stands. "Check again" asks the dialog for another load; it never refreshes a
  // token (the check passes `--no-refresh` on the shell's side).
  //
  // The pane reads NOTHING when it appears (settings-snapshot): the dialog's
  // one load already answered for it, so a visit costs no subprocess. It asks for
  // another load only when something OUTSIDE it changed the store - the terminal
  // window closing and the desk taking the focus back, or the native watcher's
  // notice that `auth.json` was written - and when the user presses "Check
  // again". Every ask goes through the dialog's single coalesced load.
  import { openTerminal } from '../bridge'
  import type { AuthView_Serialize as AuthView, ProviderStatusView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { accountCopy } from './account-copy'
  import { followSettingsFiles } from './files-changed'

  let {
    auth,
    onRefresh = () => {},
  }: {
    /** The sign-ins as the dialog's one load answered, or null before any read. */
    auth: AuthView | null
    /** Asks the dialog for one more load: the pane's only read path. */
    onRefresh?: () => void
  } = $props()

  /** The pane's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(accountCopy(language.current))

  /** What the terminal said when it could not open, or `null`. */
  let terminalError = $state<string | null>(null)

  const providers = $derived(auth?.providers ?? [])
  const failed = $derived(
    auth !== null && auth.status.type === 'Failed' ? auth.status.data : null,
  )
  const ready = $derived(auth !== null && auth.status.type === 'Ready')

  /**
   * The terminal window closes and the desk is focused again, while this pane is
   * the one on screen: one more load, the same path, never a timer.
   */
  $effect(() => {
    window.addEventListener('focus', onRefresh)
    return () => window.removeEventListener('focus', onRefresh)
  })

  /**
   * While this pane is on screen, the native watcher's notice asks for the same
   * load - only for the shell's auth store, and never after the pane is gone:
   * unmounting drops a notice's queued ask along with the listener, so no
   * subprocess runs in the background with Settings closed (rpc-settings-sync
   * T3b-2).
   */
  $effect(() => {
    const stop = followSettingsFiles('account', async () => onRefresh())
    return () => stop()
  })

  /** One provider's check as the row draws it: its word and its tone. */
  function drawn(status: ProviderStatusView): { label: string; tone: string } {
    if (status === 'Ready') return { label: copy.connected, tone: 'ok' }
    if (status === 'Unverified') return { label: copy.unverified, tone: 'quiet' }
    if (status === 'NotReady') return { label: copy.notReady, tone: 'warn' }
    if (status === 'Invalid') return { label: copy.invalid, tone: 'bad' }
    return { label: copy.error, tone: 'bad' }
  }

  async function open(where: 'Login' | 'Logout'): Promise<void> {
    terminalError = await openTerminal(where)
  }
</script>

{#if failed !== null}
  <!-- The store is there and could not be read: say why, and that a check is safe. -->
  <p class="quiet" role="status">{copy.failed(failed)}</p>
  <p class="quiet">{copy.retrySafe}</p>
{:else if !ready}
  <p class="quiet">{copy.reading}</p>
{:else if providers.length === 0}
  <p class="quiet">{copy.noProviders}</p>
{:else}
  <ul class="providers">
    {#each providers as provider (provider.name)}
      <li class="provider">
        <span class="provider-name">{provider.name}</span>
        <span class="tag {drawn(provider.status).tone}">{drawn(provider.status).label}</span>
        {#if provider.reason !== null}
          <span class="reason">{provider.reason}</span>
        {/if}
      </li>
    {/each}
  </ul>
{/if}

<div class="acts">
  <button class="btn accent" type="button" onclick={() => void open('Login')}>{copy.openShell}</button>
  <button class="btn" type="button" onclick={onRefresh}>{copy.checkAgain}</button>
</div>
<p class="note">{copy.note}</p>
{#if terminalError !== null}
  <p class="quiet" role="status">{terminalError}</p>
{/if}
