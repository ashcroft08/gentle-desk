// The Settings search (the reference's "Find a setting…" field,
// crates/app/src/features/settings/navigation.rs:105-145): the rail above the
// panes filters on the words each entry already carries - its drawn title, its
// drawn hint and the keyword list in `sections.ts` - with EVERY word of the query
// having to land on the same entry. Seven entries, so this is a scan and nothing
// more: no index to keep, no debounce to tune, no read of the machine.
//
// Case and accents are folded on BOTH sides, so "maquina" finds "máquina" and a
// query typed in one language reaches an entry drawn in the other: the keyword
// list carries both columns of words at once (`sections.ts`), while the title and
// the hint arrive in the language the dialog is drawing right now.
import type { SettingsCopy } from './copy'
import { SECTIONS, type SectionEntry } from './sections'

/**
 * The text as the search compares it: lowercase and without accents. The
 * combining marks NFD splits off are what a typed accent becomes, so both
 * "máquina" and "maquina" fold to the same seven letters.
 */
export function fold(text: string): string {
  return text
    .normalize('NFD')
    .replace(/[\u0300-\u036f]/g, '')
    .toLowerCase()
}

/** Everything one entry can be found by: its title, its hint and its keywords. */
function searchable(entry: SectionEntry, copy: SettingsCopy): string {
  return fold([copy.titles[entry.id], copy.hints[entry.id], ...entry.keywords].join(' '))
}

/** The words a query asks for, folded: an empty query asks for nothing. */
function terms(query: string): string[] {
  return fold(query)
    .split(/\s+/)
    .filter((term) => term !== '')
}

/**
 * Whether `entry` holds every term of `query`. An empty query (or one that is
 * only spaces) holds everything, which is what draws the rail as it always was.
 */
export function matches(entry: SectionEntry, copy: SettingsCopy, query: string): boolean {
  const asked = terms(query)
  if (asked.length === 0) return true
  const haystack = searchable(entry, copy)
  return asked.every((term) => haystack.includes(term))
}

/** The entries the rail draws for `query`, in the order the rail draws them. */
export function visibleSections(copy: SettingsCopy, query: string): SectionEntry[] {
  return SECTIONS.filter((entry) => matches(entry, copy, query))
}
