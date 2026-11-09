<!-- a11y lab: bits-ui widget 3: menu (button trigger, arrow keys, Esc focus return). -->
<script lang="ts">
  import { DropdownMenu } from 'bits-ui'
</script>

<section>
  <h3>Menu</h3>
  <DropdownMenu.Root>
    <DropdownMenu.Trigger id="menu-trigger">Actions</DropdownMenu.Trigger>
    <DropdownMenu.Portal>
      <DropdownMenu.Content id="menu-content">
        {#each ['Item one', 'Item two', 'Item three'] as item (item)}
          <DropdownMenu.Item>
            {item}
          </DropdownMenu.Item>
        {/each}
      </DropdownMenu.Content>
    </DropdownMenu.Portal>
  </DropdownMenu.Root>
</section>
