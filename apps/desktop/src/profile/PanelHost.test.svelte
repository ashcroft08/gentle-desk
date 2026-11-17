<script lang="ts">
  // The Profile tab mounted the way App.svelte mounts it: its props read off a
  // derived of a store that every state push replaces with a NEW object, even
  // when the chat it names is the same one.
  import type { Readable } from 'svelte/store'
  import type { ChatProfileView_Serialize as ChatProfile } from '../contract/bindings'
  import ProfilePanel from './ProfilePanel.svelte'

  let { source }: { source: Readable<{ chat: string | null; chatProfile: ChatProfile | null }> } = $props()
  const chat = $derived($source)
</script>

<ProfilePanel chatId={chat.chat} view={chat.chatProfile} active />
