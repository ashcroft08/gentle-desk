<script lang="ts">
  // The Account pane (0.1.12, S1, T3, L11): where the shell's authentication
  // stands, one row per provider the shell's own store lists. NAMES only - the
  // shell's store is read for its top-level keys and nothing else travels up -
  // and every row's state is the text of its own check's answer, never a colour
  // on its own.
  //
  // The terminal does the work: login and logout are the shell's commands, so
  // each button opens a window on it, and this pane only shows where the user
  // stands. "Check again" runs the same per-provider check again; it never
  // refreshes a token (the check passes `--no-refresh` on the shell's side).
  import { dispatchIntent, openTerminal } from '../bridge'
  import type { AuthView_Serialize as AuthView, ProviderStatusView } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { accountCopy } from './account-copy'

  let { auth }: { auth: AuthView | null } = $props()

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
   * The pane reads when it appears, never waiting for a button: the check runs
   * on the intent thread (~1s per provider) and the quiet "Reading..." above is
   * the waiting state. This is the Profiles pane's own pattern (0.1.12, L17).
   */
  $effect(() => {
    check()
  })

  /**
   * The terminal window closes and the desk is focused again, while this pane is
   * the one on screen: one more check, the same push, never a timer.
   */
  $effect(() => {
    window.addEventListener('focus', check)
    return () => window.removeEventListener('focus', check)
  })

  /**
   * The same check the "Check again" button asks for, and the one the pane asks
   * for itself: the shell's per-provider check, never a token refresh.
   */
  function check(): void {
    void dispatchIntent({ type: 'RefreshAuth' })
  }

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
  <button class="btn" type="button" onclick={check}>{copy.checkAgain}</button>
</div>
<p class="note">{copy.note}</p>
{#if terminalError !== null}
  <p class="quiet" role="status">{terminalError}</p>
{/if}
