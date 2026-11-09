<!-- a11y lab: bits-ui widget 2: list of options (arrows, selected state). -->
<!-- bits-ui has no bare listbox; Select is its listbox carrier (ARIA-1.2:
     trigger role combobox + popup role listbox). -->
<script lang="ts">
  import { Select } from 'bits-ui'

  const options = [
    { value: 'alpha', label: 'Alpha' },
    { value: 'beta', label: 'Beta' },
    { value: 'gamma', label: 'Gamma' }
  ]

  let value = $state('alpha')

  const chosen = $derived(options.find((option) => option.value === value)?.label ?? 'Choose a value')
</script>

<section>
  <h3>List</h3>
  <Select.Root type="single" bind:value>
    <Select.Trigger id="list-trigger">{chosen}</Select.Trigger>
    <Select.Content id="list-content">
      {#each options as option (option.value)}
        <Select.Item value={option.value} label={option.label}>{option.label}</Select.Item>
      {/each}
    </Select.Content>
  </Select.Root>
</section>
