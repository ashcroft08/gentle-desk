<!-- a11y lab: bits-ui widget 1: modal dialog (focus trap, Esc, focus return). -->
<script lang="ts">
  import { Dialog } from 'bits-ui'
</script>

<section>
  <h3>Dialog</h3>
  <Dialog.Root>
    <Dialog.Trigger id="dialog-trigger">Open dialog</Dialog.Trigger>
    <Dialog.Portal>
      <Dialog.Content id="dialog-content">
        <Dialog.Title id="dialog-title">Lab dialog</Dialog.Title>
        <Dialog.Description id="dialog-description">
          bits-ui dialog sample
        </Dialog.Description>
        <Dialog.Close id="dialog-close">Close</Dialog.Close>
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
</section>
