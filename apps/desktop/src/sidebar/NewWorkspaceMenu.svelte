<script lang="ts">
  // The New workspace control (D7, Q1 option A, L13): the header's folder-plus
  // icon opens the artboard's two-item menu, and each item starts one of the two
  // W3c paths. The menu is bits-ui's DropdownMenu, so the trigger carries
  // `aria-haspopup` and `aria-expanded`, the items are menu items, the arrow
  // keys, Home and End move between them, Escape closes and gives the focus back.
  import { DropdownMenu } from 'bits-ui'
  import Icon from '../icons/Icon.svelte'
  import { language } from '../i18n/language.svelte'
  import { sidebarCopy } from './copy'

  let {
    open = $bindable(false),
    trigger = $bindable(null),
    onChooseFolder,
    onCreateFolder,
  }: {
    /** Whether the menu is open; the empty first-run prompt opens it too. */
    open?: boolean
    /** The icon element, so a closing dialog gives the focus back to it. */
    trigger?: HTMLElement | null
    /** "Choose an existing folder": the folder picker path. */
    onChooseFolder: () => void
    /** "Create a new folder": opens the name dialog. */
    onCreateFolder: () => void
  } = $props()

  /** The control's and its two items' words (0.2.0, S3/D4). */
  const copy = $derived(sidebarCopy(language.current))
</script>

<DropdownMenu.Root bind:open>
  <DropdownMenu.Trigger
    class="ic tip-r"
    aria-label={copy.newWorkspace}
    data-tip={copy.newWorkspace}
    bind:ref={trigger}
  >
    <Icon name="folder-plus" />
  </DropdownMenu.Trigger>
  <DropdownMenu.Portal>
    <DropdownMenu.Content class="menu" aria-label={copy.newWorkspace} side="bottom" align="end" sideOffset={6}>
      <DropdownMenu.Item onSelect={onChooseFolder}>{copy.menuChoose}</DropdownMenu.Item>
      <DropdownMenu.Item onSelect={onCreateFolder}>{copy.menuCreate}</DropdownMenu.Item>
    </DropdownMenu.Content>
  </DropdownMenu.Portal>
</DropdownMenu.Root>
