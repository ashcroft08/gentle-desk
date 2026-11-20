// The alert's sound (S2): a short two-note greeting synthesized on the spot, so
// the app carries no audio file and links no audio crate. WebAudio is the whole
// of it - each note its own oscillator into its own gain that swells and falls -
// and ONE AudioContext per window, created the first time a tone is due and
// reused by every later one, because a context per tone would leak a voice per
// alert.
//
// Nothing here decides whether a tone is due (that is `notify.ts`), and nothing
// here throws: a webview with no WebAudio, a machine with no output device, a
// policy that keeps the context suspended - every refusal is silence, since a
// notification is a courtesy and never worth an error over the user's work.

/** How loud: a hint next to someone's ear, not an alarm. */
const PEAK = 0.08

/** How long a note takes to reach that, so it swells instead of clicking. */
const ATTACK = 0.02

/** Where an envelope starts and ends: an exponential ramp cannot reach zero. */
const SILENCE = 0.0001

/**
 * The two notes, in Hz and in seconds from the start of the tone: a rising
 * fourth (E5 then A5) that sounds like a small greeting. Both together are just
 * over 300 ms, which is as long as a notification is welcome.
 */
const NOTES: readonly { hz: number; from: number; to: number }[] = [
  { hz: 659.25, from: 0, to: 0.14 },
  { hz: 880, from: 0.15, to: 0.32 },
]

/** The window's one voice, or `null` until a tone is first due. */
let context: AudioContext | null = null

/** That voice, created on the first tone; `null` when this webview has none. */
function voice(): AudioContext | null {
  if (context !== null) return context
  if (typeof AudioContext === 'undefined') return null
  context = new AudioContext()
  return context
}

/**
 * Schedules one note from `at` seconds on the context's clock: its own
 * oscillator into its own gain, so each note carries its own envelope and the
 * second one cannot bend the first.
 */
function schedule(ctx: AudioContext, note: (typeof NOTES)[number], at: number): void {
  const oscillator = ctx.createOscillator()
  const gain = ctx.createGain()
  oscillator.type = 'sine'
  oscillator.frequency.setValueAtTime(note.hz, at + note.from)
  gain.gain.setValueAtTime(SILENCE, at + note.from)
  gain.gain.linearRampToValueAtTime(PEAK, at + note.from + ATTACK)
  gain.gain.exponentialRampToValueAtTime(SILENCE, at + note.to)
  oscillator.connect(gain)
  gain.connect(ctx.destination)
  oscillator.start(at + note.from)
  oscillator.stop(at + note.to + 0.01)
}

/**
 * Plays the alert tone, and resolves when it has been scheduled - not when it
 * has finished, because nothing waits for a sound. Never rejects.
 */
export async function playTone(): Promise<void> {
  try {
    const ctx = voice()
    if (ctx === null) return
    // A context created without a user gesture starts suspended, and a
    // suspended one plays nothing: ask for it first and give up if refused.
    if (ctx.state === 'suspended') await ctx.resume()
    if (ctx.state !== 'running') return
    const at = ctx.currentTime + 0.01
    for (const note of NOTES) schedule(ctx, note, at)
  } catch {
    // Silence, as the header says.
  }
}
