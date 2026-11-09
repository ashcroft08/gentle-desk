import { vitePreprocess } from '@sveltejs/vite-plugin-svelte'
import { preprocessMeltUI, sequence } from '@melt-ui/pp'

export default {
  // Consult https://svelte.dev/docs#compile-time-svelte-preprocess
  // for more information about preprocessors
  // The melt-ui preprocessor is required by every `use:melt` directive; the
  // lab's melt candidate (src/a11y-lab/meltui) does not work without it.
  preprocess: sequence([preprocessMeltUI(), vitePreprocess()])
}
