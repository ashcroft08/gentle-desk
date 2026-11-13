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
  import {
    CHECK_AGAIN,
    CONNECTED,
    ERROR,
    FAILED,
    INVALID,
    LOGIN,
    LOGOUT,
    NOT_READY,
    NOTE,
    NO_PROVIDERS,
    READING,
    RETRY_SAFE,
  } from './account-copy'

  let { auth }: { auth: AuthView | null } = $props()

  /** What the terminal said when it could not open, or `null`. */
  let terminalError = $state<string | null>(null)

  const providers = $derived(auth?.providers ?? [])
  const failed = $derived(
    auth !== null && auth.status.type === 'Failed' ? auth.status.data : null,
  )
  const ready = $derived(auth !== null && auth.status.type === 'Ready')

  /** One provider's check as the row draws it: its word and its tone. */
  function drawn(status: ProviderStatusView): { label: string; tone: string } {
    if (status === 'Ready') return { label: CONNECTED, tone: 'ok' }
    if (status === 'NotReady') return { label: NOT_READY, tone: 'warn' }
    if (status === 'Invalid') return { label: INVALID, tone: 'bad' }
    return { label: ERROR, tone: 'bad' }
  }

  function check(): void {
    void dispatchIntent({ type: 'RefreshAuth' })
  }

  async function open(where: 'Login' | 'Logout'): Promise<void> {
    terminalError = await openTerminal(where)
  }
</script>

{#if failed !== null}
  <!-- The store is there and could not be read: say why, and that a check is safe. -->
  <p class="quiet" role="status">{FAILED(failed)}</p>
  <p class="quiet">{RETRY_SAFE}</p>
{:else if !ready}
  <p class="quiet">{READING}</p>
{:else if providers.length === 0}
  <p class="quiet">{NO_PROVIDERS}</p>
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
  <button class="btn" type="button" onclick={() => void open('Login')}>{LOGIN}</button>
  <button class="btn" type="button" onclick={() => void open('Logout')}>{LOGOUT}</button>
  <button class="btn accent" type="button" onclick={check}>{CHECK_AGAIN}</button>
</div>
<p class="note">{NOTE}</p>
{#if terminalError !== null}
  <p class="quiet" role="status">{terminalError}</p>
{/if}
