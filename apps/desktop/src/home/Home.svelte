<script lang="ts">
  // View A, the welcome (welcome-screen T2, S2, S3, R1): what the main area holds
  // while NO chat is in the foreground. It is a screen of text and real data -
  // the workspace worked in last and its recent chats, both read from the session
  // list - and it starts no process: the one action is an intent, and only the
  // chat it opens costs anything.
  //
  // It is drawn from `ViewState.welcome` alone (T1), its own dictionary (both
  // languages) and the rose (T5). The three states are the sheet's: a workspace
  // with no chat yet says so and keeps the action alone; a folder that is gone
  // disables the action, prints Rust's own reason VERBATIM and links the button to
  // it; and with no configured workspace the screen says what the empty main area
  // has always said. A `null` slice means a chat is open here instead, so the
  // screen draws nothing at all.
  import Rose from '../brand/Rose.svelte'
  import Icon from '../icons/Icon.svelte'
  import { dispatchIntent } from '../bridge'
  import { appCopy } from '../app-copy'
  import { language } from '../i18n/language.svelte'
  import { onMount } from 'svelte'
  import type { WelcomeView_Serialize as WelcomeView } from '../contract/bindings'
  import { homeCopy } from './copy'
  import { ago, isJustNow } from './ago'
  import { ROSE_FALLBACK, watchRoseHeight } from './rose-height'

  let { welcome }: { welcome: WelcomeView | null } = $props()

  const copy = $derived(homeCopy(language.current))
  const shell = $derived(appCopy(language.current))
  const last = $derived(welcome?.last ?? null)
  const recents = $derived(welcome?.recents ?? [])
  const others = $derived(welcome?.others ?? [])
  /** The clock this screen was drawn with: taken once, never ticking. */
  const nowMs = Date.now()

  /**
   * The rose's height, which is the WINDOW's: the screen redraws the same picture
   * bigger in a tall window and smaller in a short or narrow one, and follows the
   * user when the window is resized. The fallback holds until the window answers.
   */
  let rosePx = $state(ROSE_FALLBACK)
  onMount(() => watchRoseHeight((px) => (rosePx = px)))

  /** The age the sidebar would word this chat's last write with. */
  function ageOf(thenMs: number | null): string {
    return thenMs === null ? '' : ago(language.current, nowMs, thenMs)
  }

  /** The line beside the folder: its age, or that it holds no chat yet. */
  function lineOf(chatCount: number, thenMs: number | null): string {
    if (chatCount === 0 || recents.length === 0) return copy.noChats
    if (thenMs === null) return ''
    return copy.lastActivity(ageOf(thenMs), isJustNow(nowMs, thenMs))
  }

  function newChatIn(path: string): void {
    void dispatchIntent({ type: 'NewChatInWorkspace', data: { path } })
  }

  function openChat(chatId: string): void {
    void dispatchIntent({ type: 'OpenChat', data: { chat_id: chatId } })
  }
</script>

{#if welcome !== null}
  <section class="home">
    <div class="home-rose">
      <Rose label={copy.roseLabel} height={rosePx} />
    </div>

    {#if last === null}
      <!-- No configured workspace: there is nothing to be last in, so only the
           line the empty main area always drew is left. -->
      <p class="home-hint">{shell.noChatHint}</p>
    {:else}
      {@const workspace = last}
      {@const line = lineOf(workspace.chat_count, workspace.last_activity_ms)}
      <h2 class="home-title">{copy.title}</h2>
      <p class="home-where">
        <Icon name="folder" size={14} />
        <span class="home-ws">{workspace.label}</span>
        {#if line !== ''}<span class="home-dim">{line}</span>{/if}
      </p>

      <button
        type="button"
        class="home-go"
        disabled={!workspace.available}
        aria-describedby={workspace.available ? undefined : 'home-why'}
        onclick={() => newChatIn(workspace.path)}
      >
        <Icon name="plus" size={14} />
        {copy.newSession(workspace.label)}
      </button>

      {#if !workspace.available}
        <!-- The reason is the sidebar's own sentence, as Rust sent it (R3). -->
        <p class="home-why" id="home-why">
          <span class="home-reason">{workspace.reason}</span>
          <span class="home-hint">{copy.pickAnother}</span>
        </p>
      {/if}

      {#if recents.length > 0}
        <section class="home-recents" aria-labelledby="home-recents">
          <h3 class="home-kicker" id="home-recents">{copy.recentsTitle(workspace.label)}</h3>
          <ul>
            {#each recents as chat (chat.chat_id)}
              <li>
                <button type="button" class="home-chat" onclick={() => openChat(chat.chat_id)}>
                  <span class="home-chat-title">{chat.title}</span>
                  {#if chat.modified_ms !== null}
                    <span class="home-chat-age">{ageOf(chat.modified_ms)}</span>
                  {/if}
                </button>
              </li>
            {/each}
          </ul>
        </section>
      {/if}

      {#if others.length > 0}
        <!-- Quiet links: Rust already leaves the last workspace out of them. -->
        <p class="home-others">
          <span>{copy.othersTitle}</span>
          {#each others as link (link.path)}
            <button
              type="button"
              class="home-quiet"
              disabled={!link.available}
              onclick={() => newChatIn(link.path)}
            >{link.label}</button>
          {/each}
        </p>
      {/if}
    {/if}
  </section>
{/if}
