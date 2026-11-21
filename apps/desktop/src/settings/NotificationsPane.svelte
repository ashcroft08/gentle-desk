<script lang="ts">
  // The Notifications section (notifications S4, S5; the approved design
  // `design/notifications/`): ONE setting, in the same segmented row Language
  // already uses - a tone and a system notification while the window is not in
  // front (the default), or off - the explanation of what the app does, and one
  // "Try it" button that plays the tone and sends one toast right now, so the
  // maintainer can hear it and learn whether Windows allows it without waiting
  // for a real event (S5).
  //
  // The setting reaches this pane as a prop from the dialog, which keeps drawing
  // it from the live push (`ViewState.notifications`, the same way theme and the
  // app's language do): choosing one dispatches `SetNotifications` and Rust
  // re-emits the slice, so the row follows what the app HOLDS, never what the
  // click hoped. Choosing the mode already held sends nothing.
  //
  // The tone and the toast are T3's (`../notifications/try`): this pane knows only
  // its two answers - `sent` and `denied` - and writes each as one status line
  // under the action, never a dialog inside the dialog. `denied` also draws the
  // way to allow it in Windows Settings, because the tone DID play and only the
  // system toast was refused.
  //
  // While the setting is off the button is `aria-disabled`, not `disabled`: the
  // reason under it has to stay reachable with the keyboard and readable by a
  // screen reader (03-off). The explanation is never hidden either - whoever
  // turns it off has to be able to read what they turned off.
  import { dispatchIntent } from '../bridge'
  import type { NotificationsMode } from '../contract/bindings'
  import { language } from '../i18n/language.svelte'
  import { tryNotification } from '../notifications/try'
  import { notificationsCopy } from './notifications-copy'

  let { notifications }: {
    /** The mode the app holds, from the live push the dialog draws. */
    notifications: NotificationsMode
  } = $props()

  /** The pane's own words in the language the app holds (0.2.0, D4). */
  const copy = $derived(notificationsCopy(language.current))

  /** The two answers, in the order the artboards draw them: the default, then off. */
  const MODES: readonly NotificationsMode[] = ['when-unfocused', 'off']

  /** The last try's answer, or `null` before the first one and after a choice. */
  let result = $state<'sent' | 'denied' | null>(null)

  /** One choice, one intent: the mode already held is not a change. */
  function choose(next: NotificationsMode): void {
    if (next === notifications) return
    // The line spoke about the mode the user just left: it goes with it (02).
    result = null
    void dispatchIntent({ type: 'SetNotifications', data: { mode: next } })
  }

  /** Plays the tone and sends one toast, and draws which of them got through. */
  async function attempt(): Promise<void> {
    if (notifications === 'off') return
    result = await tryNotification(language.current)
  }
</script>

<div class="seg" role="radiogroup" aria-label={copy.group}>
  {#each MODES as mode (mode)}
    <button
      type="button"
      role="radio"
      aria-checked={notifications === mode}
      onclick={() => choose(mode)}>{mode === 'when-unfocused' ? copy.whenUnfocused : copy.off}</button
    >
  {/each}
</div>

<p class="why">{copy.explain}</p>

<div class="acts">
  <button
    class="btn"
    type="button"
    aria-disabled={notifications === 'off'}
    onclick={() => void attempt()}>{copy.tryIt}</button
  >
</div>

{#if notifications === 'off'}
  <!-- Off: the reason Try it is dead, in place of any answer (03-off). It is also
       what a push to off leaves behind, so an old try can never outlive it. -->
  <p class="quiet">{copy.offReason}</p>
{:else if result === 'sent'}
  <p class="quiet" role="status">
    <span class="tag ok">{copy.sentTag}</span>{copy.sentLine}
  </p>
{:else if result === 'denied'}
  <p class="quiet" role="status">
    <span class="tag warn">{copy.deniedTag}</span>{copy.deniedLine}
  </p>
  <p class="note">{copy.deniedNote}</p>
{/if}
