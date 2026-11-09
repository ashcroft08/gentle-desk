<!-- a11y lab: melt-ui widget 2: list of options (arrows, selected state). -->
<!-- NOTE: melt-ui does not export its internal listbox builder publicly; the
     closest public surface is createSelect (trigger role combobox, popup role
     listbox), which is what this sample uses. -->
<script lang="ts">
  import { createSelect, melt } from '@melt-ui/svelte'

  const options = [
    { value: 'alpha', label: 'Alpha' },
    { value: 'beta', label: 'Beta' },
    { value: 'gamma', label: 'Gamma' }
  ]

  const select = createSelect<string>({ defaultSelected: options[0] })
  const { trigger, option, menu, label } = select.elements
  const open = select.states.open
  const selectedLabel = select.states.selectedLabel
</script>

<section>
  <h3>List</h3>
  <span use:melt={$label}>Lab list</span>
  <button use:melt={$trigger}>{$selectedLabel}</button>
  {#if $open}
    <ol use:melt={$menu}>
      {#each options as item (item.value)}
        <li use:melt={$option(item)}>{item.label}</li>
      {/each}
    </ol>
  {/if}
</section>
