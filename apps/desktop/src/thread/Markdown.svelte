<script lang="ts">
  // The drawn markdown of one reply: html already rendered by `markdown.ts` and
  // already sanitized by `sanitize.ts` (DOMPurify with the thread's allowlist),
  // so this component only places it and routes a click on a link to the shell.
  import { linkClickHandler } from './links'

  let { html = '' }: { html?: string } = $props()
</script>

<!-- A click or a middle click on a link is cancelled here and forwarded to
     `open_link`; a drawn reply never navigates the view. -->
<div class="md" onclick={linkClickHandler} onauxclick={linkClickHandler} role="presentation">
  <!-- Every block of this html is DOMPurify output from sanitize.ts, which
       drops script, style, iframe, form, img and every attribute but a link's
       href, so the warning below is answered here. -->
  <!-- eslint-disable-next-line svelte/no-at-html-tags -->
  {@html html}
</div>
