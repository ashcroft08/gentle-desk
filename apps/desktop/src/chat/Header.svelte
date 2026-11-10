<script lang="ts">
  // The chat header: the foreground chat's title, and nothing else. The title is
  // drawn whole and cut by CSS for the width it has, so whoever asks for it - a
  // hover, a screen reader - still reads the session's full name.
  //
  // The artboard's "..." menu (Rename, Delete) and its Chat|Helpers tabs are NOT
  // drawn: their actions are unknown here and the helpers panel is T7's.
  let { title }: { title: string } = $props()
</script>

<header class="mh">
  <h1 title={title}>{title}</h1>
</header>
