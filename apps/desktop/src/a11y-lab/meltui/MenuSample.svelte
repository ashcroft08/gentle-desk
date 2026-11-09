<!-- a11y lab: melt-ui widget 3: menu (button trigger, arrow keys, Esc focus return). -->
<script lang="ts">
  import { createDropdownMenu, melt } from '@melt-ui/svelte'

  const dropdown = createDropdownMenu()
  const { trigger, menu: menuContent, item } = dropdown.elements
  const open = dropdown.states.open
</script>

<section>
  <h3>Menu</h3>
  <button use:melt={$trigger}>Actions</button>
  {#if $open}
    <div use:melt={$menuContent}>
      {#each ['Item one', 'Item two', 'Item three'] as text (text)}
        <div use:melt={$item}>
          {text}
        </div>
      {/each}
    </div>
  {/if}
</section>
