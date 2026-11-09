<script lang="ts">
  // The first run, as the states board draws it: no workspace yet, so the only
  // way forward is New workspace. The reason line is the one New session is
  // described by, so the disabled button and this block say the same thing once.
  let {
    title = '',
    reason = '',
    reasonId = 'workspace-empty-why',
  }: { title?: string; reason?: string; reasonId?: string } = $props()
</script>

<div class="empty">
  <p class="et">{title}</p>
  <p class="why" id={reasonId}>{reason}</p>
</div>
