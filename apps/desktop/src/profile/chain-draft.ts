// The Profile tab's chain DRAFT rules (profiles-editor T5, L30-3): what the user
// types in one chain's editor, before anything is sent. They are plain data, so a
// component draws a decision and a test can pin the decision itself (the
// `rules.ts` pattern).
//
// The validation is the OWNER's own (`normalizeFallbacksFile`, quoted in
// `docs/desktop-rpc.md:345` and `:382`), so the editor can never make the shell
// send a document gentle-pi answers `invalid_data` to: at most eight steps, each
// step naming a model, no model twice in one chain, and never an empty chain.
import type { ChatProfileStepView_Serialize as Step } from '../contract/bindings'

/** The owner's own cap on one chain (`desktop-rpc.md:345`). */
export const MAX_CHAIN_STEPS = 8

/** One step as the user edits it: both fields are text, so an empty effort is
 * simply no effort (the store's `thinking` is absent, never the empty string). */
export interface DraftStep {
  model: string
  thinking: string
}

/** The store's chain, as the editor holds it. */
export function draftOf(steps: Step[]): DraftStep[] {
  return steps.map((step) => ({ model: step.model, thinking: step.thinking ?? '' }))
}

/** The draft as the store's steps: an empty effort input is NO effort. */
export function stepsOf(draft: DraftStep[]): Step[] {
  return draft.map((step) => ({
    model: step.model,
    thinking: step.thinking.trim() === '' ? null : step.thinking,
  }))
}

/** Whether the draft still says exactly what the store holds. */
export function sameChain(draft: DraftStep[], steps: Step[]): boolean {
  const stored = draftOf(steps)
  return (
    draft.length === stored.length &&
    draft.every(
      (step, index) => step.model === stored[index].model && step.thinking === stored[index].thinking,
    )
  )
}

/** Why one draft is not a chain the owner would keep, or `null` when it is. */
export type ChainProblem =
  | { kind: 'empty' }
  | { kind: 'too-long'; steps: number }
  | { kind: 'missing-model'; index: number }
  | { kind: 'repeated-model'; model: string }

/**
 * The first rule a draft breaks, in the owner's own order: the count, then every
 * step, then the repetition. `index` is 1-BASED, because it is drawn for a person.
 */
export function chainProblem(draft: DraftStep[]): ChainProblem | null {
  if (draft.length === 0) return { kind: 'empty' }
  if (draft.length > MAX_CHAIN_STEPS) return { kind: 'too-long', steps: draft.length }
  const seen = new Set<string>()
  for (const [index, step] of draft.entries()) {
    const model = step.model.trim()
    if (model === '') return { kind: 'missing-model', index: index + 1 }
    if (seen.has(step.model)) return { kind: 'repeated-model', model: step.model }
    seen.add(step.model)
  }
  return null
}

/** One more empty step at the end, or the same draft at the owner's cap. */
export function addStep(draft: DraftStep[]): DraftStep[] {
  if (draft.length >= MAX_CHAIN_STEPS) return draft
  return [...draft, { model: '', thinking: '' }]
}

/** The draft without step `index`; never emptied, because an empty chain is not
 * a chain the owner keeps. */
export function removeStep(draft: DraftStep[], index: number): DraftStep[] {
  if (index < 0 || index >= draft.length || draft.length <= 1) return draft
  return draft.filter((_, at) => at !== index)
}

/** Step `index` moved by `by` places (`-1` up, `1` down), when there is room. */
export function moveStep(draft: DraftStep[], index: number, by: -1 | 1): DraftStep[] {
  const to = index + by
  if (index < 0 || index >= draft.length || to < 0 || to >= draft.length) return draft
  const moved = [...draft]
  const [step] = moved.splice(index, 1)
  moved.splice(to, 0, step)
  return moved
}
