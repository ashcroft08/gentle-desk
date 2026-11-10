<script lang="ts">
  // A user message is PLAIN text (crates/ui/src/view/thread.rs draws it with a
  // `text` widget): the markdown a person typed is shown as typed, so what the
  // model receives is what the person sees. CSS keeps the line breaks.
  let { text = '' }: { text?: string } = $props()
</script>

<p class="ut">{text}</p>
