<script lang="ts">
  // The composer's clip (design/composer-dock/03-attachments.html, sheet A): the
  // quiet 26px button in the row that opens the system's file dialog. Its glyph
  // is the artboard's own paperclip, drawn here rather than added to the shared
  // icon set: it is this one control's mark, and attaching is not a chat state.
  import { pickPaths } from './attachment-draft'

  let {
    label,
    disabled = false,
    onPick,
  }: {
    /** Its accessible name, which is also the dialog's title. */
    label: string
    /** A composer with no chat to attach to cannot ask for a file. */
    disabled?: boolean
    /** Opens the dialog; the paths it answers go back to the composer. */
    onPick: (paths: string[]) => Promise<void> | void
  } = $props()
</script>

<button
  class="att"
  type="button"
  aria-label={label}
  {disabled}
  onclick={async () => onPick(await pickPaths(label))}
>
  <svg viewBox="0 0 24 24" aria-hidden="true">
    <path d="M21 11l-8.5 8.5a5 5 0 0 1-7-7L14 4a3.5 3.5 0 0 1 5 5l-8.5 8.5a2 2 0 0 1-3-3L15 6" />
  </svg>
</button>
