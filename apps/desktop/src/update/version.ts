// Version equality as the update rules read it (0.8.0 settings-polish S2): the
// version the user skipped and the one a row publishes are compared as VERSIONS,
// never as text, so `2.0.5` and `02.0.5` are one version - and a value that is not
// a strict `x.y.z` is no version at all, which is what makes a skip this app
// cannot read hide nothing.
//
// The one shape is the check's own (`gentle_runtime::updates::parse`): three
// dot-separated numbers, trimmed, and nothing else. The two rules of this module
// are shared by the pane (`settings/updates.ts`) and the gear's dot (`dot.ts`), so
// the two cannot disagree about which version was skipped.

/** One version as its three numbers, or `null` for anything that is not `x.y.z`. */
export function parseVersion(text: string | null | undefined): [number, number, number] | null {
  if (typeof text !== 'string') return null
  const parts = text.trim().split('.')
  if (parts.length !== 3) return null
  const numbers: number[] = []
  for (const part of parts) {
    if (part === '' || !/^[0-9]+$/.test(part)) return null
    const value = Number(part)
    // A component this app cannot hold as a number is no version either; the
    // Rust half answers `None` for the same values.
    if (!Number.isSafeInteger(value)) return null
    numbers.push(value)
  }
  return [numbers[0], numbers[1], numbers[2]]
}

/**
 * Whether `a` and `b` name the same version. Two unreadable values are never the
 * same version: `sameVersion('latest', 'latest')` is false, so a skip nobody can
 * compare hides nothing.
 */
export function sameVersion(a: string | null | undefined, b: string | null | undefined): boolean {
  const left = parseVersion(a)
  const right = parseVersion(b)
  if (left === null || right === null) return false
  return left[0] === right[0] && left[1] === right[1] && left[2] === right[2]
}
