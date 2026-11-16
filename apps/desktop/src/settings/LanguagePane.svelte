<script lang="ts">
  // The Language section (0.2.0, S2 and S4): the two languages the app's own
  // words are written in, each named in its own language - a language's own name
  // is never translated - with the current one marked. Choosing one dispatches
  // immediately and the dialog's own words change in the same tick, because the
  // rail reads the same reactive language this control sets; nothing restarts.
  //
  // The marked language is what the app HOLDS, not what the picker last sent:
  // a run with no stored preference follows the machine (i18n/language.svelte),
  // and a stored one is the user's until they choose again.
  import type { LanguageView } from '../contract/bindings'
  import { language, setLanguage } from '../i18n/language.svelte'
  import { settingsCopy } from './copy'

  /** The two languages, each named in its own language (0.2.0, S4). */
  const LANGUAGES: { id: LanguageView; name: string }[] = [
    { id: 'en', name: 'English' },
    { id: 'es', name: 'Español' },
  ]

  /** The group's own name, in the language the dialog is written in. */
  const label = $derived(settingsCopy(language.current).languageGroup)

  /** One choice, one intent: the same language again changes nothing in Rust. */
  function choose(next: LanguageView): void {
    if (next === language.current) return
    setLanguage(next)
  }
</script>

<div class="seg" role="radiogroup" aria-label={label}>
  {#each LANGUAGES as option (option.id)}
    <button
      type="button"
      role="radio"
      aria-checked={language.current === option.id}
      lang={option.id}
      onclick={() => choose(option.id)}>{option.name}</button
    >
  {/each}
</div>
