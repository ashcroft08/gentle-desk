<script lang="ts">
  // The files a chat's message will carry (design/composer-dock/03-attachments.html,
  // sheets A, B and D): its own row above the text, each file a chip with its NAME
  // (the whole path is the hover), an amber chip and one line under the row for a
  // file outside the chat's folder (R5), and the first three with a "+N more"
  // button that opens the whole list.
  //
  // Everything here is the composer's own state: Rust decides what each path IS,
  // and nothing on this row sends, opens or reads a file.
  import type { AttachmentView } from '../contract/bindings'
  import Icon from '../icons/Icon.svelte'
  import { language } from '../i18n/language.svelte'
  import { MAX_VISIBLE, collapsedOf } from './attach'
  import { composerCopy } from './copy'

  let {
    attachments,
    onRemove,
  }: {
    /** The files attached to this message, in the order they arrived. */
    attachments: AttachmentView[]
    /** Takes one chip away; nothing is sent. */
    onRemove: (path: string) => void
  } = $props()

  const copy = $derived(composerCopy(language.current))

  /** Whether the collapsed row was opened to its whole list (sheet D). */
  let expanded = $state(false)
  const row = $derived(collapsedOf(attachments, expanded))
  const anyOutside = $derived(attachments.some((view) => view.outside))

  // A row short enough to be drawn whole has nothing left to keep open, so a
  // later list does not arrive already expanded.
  $effect(() => {
    if (attachments.length <= MAX_VISIBLE) expanded = false
  })
</script>

{#if attachments.length > 0}
  <div class="chips" role="list" aria-label={copy.attachedFiles}>
    {#each row.shown as view (view.path)}
      <span class="chip" class:out={view.outside} role="listitem" title={view.path}>
        <!-- The artboard's one document glyph: the file is the content, so no
             per-file kind changes the mark. -->
        <svg viewBox="0 0 24 24" aria-hidden="true">
          <path d="M14 3v5h5" />
          <path d="M6 3h8l5 5v13H6z" />
        </svg>
        <span class="t">{view.name}</span>
        {#if view.outside}
          <span class="why">{copy.outsideWorkspace}</span>
        {/if}
        <button
          class="x"
          type="button"
          aria-label={`${copy.removeFile} ${view.name}`}
          onclick={() => onRemove(view.path)}
        >
          <Icon name="x" size={12} />
        </button>
      </span>
    {/each}
    {#if row.hidden > 0}
      <button
        class="more"
        type="button"
        aria-expanded={expanded ? 'true' : 'false'}
        aria-label={copy.showMoreLabel(row.hidden)}
        onclick={() => (expanded = !expanded)}
      >
        {copy.showMore(row.hidden)}
      </button>
    {/if}
  </div>
  {#if anyOutside}
    <!-- ONE line, whatever the number of files, and never hidden: a file outside
         the chat's folder still attaches (R5). -->
    <p class="warn" role="status">{copy.outsideWarning}</p>
  {/if}
{/if}
