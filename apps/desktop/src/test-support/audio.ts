// A WINDOW that has WebAudio (notifications S2's tone): jsdom has none, so a
// test that wants to hear the alert installs this one and then reads what the
// app asked it to play. Only the slice the tone uses exists - one oscillator per
// note, one gain stage each, the graph the two connect - and every scheduling
// call is recorded rather than played, so a test asserts the envelope the
// maintainer would hear instead of listening to it.
//
// The app reaches WebAudio through the global `AudioContext`, which is exactly
// where `vi.stubGlobal` puts this, so nothing in the app knows it is mocked:
// `unsupported` is a browser without WebAudio, `refuses` one whose constructor
// throws and `blocked` one whose context will not resume.
import { vi } from 'vitest'

/** One scheduled automation point of a parameter, in the order asked for. */
export type AudioPoint = { kind: 'set' | 'linear' | 'exponential'; at: number; value: number }

/** One note as the app played it: what, when, and how its gain moved. */
export type AudioNote = {
  /** The wave the app chose for it. */
  wave: string
  /** The frequency it scheduled first, in Hz. */
  frequency: number
  /** When the note starts and stops, on the context's own clock. */
  start: number
  stop: number
  /** The envelope of the note's own gain stage. */
  envelope: AudioPoint[]
  /** Whether that gain stage reaches the context's destination, or to nothing. */
  audible: boolean
}

/** What one context did; the readings follow it as it changes. */
export type AudioContextStub = {
  readonly state: AudioContextState
  /** How many times the app asked this context to resume. */
  readonly resumes: number
  readonly closes: number
  /** Every note this context played, in order. */
  readonly notes: AudioNote[]
}

export type AudioWindow = {
  /** Every context the app created, oldest first: one, never one per tone. */
  readonly contexts: AudioContextStub[]
  /** Every note played, across every context, oldest first. */
  notes(): AudioNote[]
}

export type AudioOptions = {
  /** The state a new context starts in; `suspended` is what autoplay policy does. */
  state?: 'running' | 'suspended'
  /** `new AudioContext()` throws, as it can in a locked-down webview. */
  refuses?: boolean
  /** `resume()` rejects, as it does while the webview forbids audio. */
  blocked?: boolean
  /** There is no `AudioContext` global at all, as in a browser without WebAudio. */
  unsupported?: boolean
}

/** One parameter of one node: only the calls the app is allowed to need. */
class Param {
  readonly points: AudioPoint[] = []

  private at(kind: AudioPoint['kind'], value: number, when: number): this {
    this.points.push({ kind, at: when, value })
    return this
  }

  setValueAtTime(value: number, when: number): this {
    return this.at('set', value, when)
  }

  linearRampToValueAtTime(value: number, when: number): this {
    return this.at('linear', value, when)
  }

  exponentialRampToValueAtTime(value: number, when: number): this {
    return this.at('exponential', value, when)
  }

  cancelScheduledValues(): this {
    return this
  }
}

/** One node of the graph: it knows what it feeds, so a test can follow the path. */
class Node {
  readonly outputs: Node[] = []

  connect(target: Node): Node {
    this.outputs.push(target)
    return target
  }

  disconnect(): void {
    this.outputs.length = 0
  }
}

class Oscillator extends Node {
  type = 'sine'
  readonly frequency = new Param()
  readonly starts: number[] = []
  readonly stops: number[] = []

  start(when: number): void {
    this.starts.push(when)
  }

  stop(when: number): void {
    this.stops.push(when)
  }
}

class Gain extends Node {
  readonly gain = new Param()
}

/** The context itself: a running clock at zero and the two node kinds used. */
class Context {
  readonly destination = new Node()
  readonly currentTime = 0
  readonly oscillators: Oscillator[] = []
  readonly gains: Gain[] = []
  state: AudioContextState
  resumes = 0
  closes = 0

  constructor(
    private readonly options: AudioOptions,
    state: AudioContextState,
  ) {
    this.state = state
  }

  createOscillator(): Oscillator {
    const oscillator = new Oscillator()
    this.oscillators.push(oscillator)
    return oscillator
  }

  createGain(): Gain {
    const gain = new Gain()
    this.gains.push(gain)
    return gain
  }

  resume(): Promise<void> {
    this.resumes += 1
    if (this.options.blocked === true) return Promise.reject(new Error('audio is blocked'))
    this.state = 'running'
    return Promise.resolve()
  }

  close(): Promise<void> {
    this.closes += 1
    this.state = 'closed'
    return Promise.resolve()
  }
}

/** What one oscillator sounded, with the gain stage it fed and where that went. */
function notesOf(context: Context): AudioNote[] {
  return context.oscillators.map((oscillator) => {
    const gain = context.gains.find((candidate) => oscillator.outputs.includes(candidate))
    return {
      wave: oscillator.type,
      frequency: oscillator.frequency.points[0]?.value ?? 0,
      start: oscillator.starts[0] ?? 0,
      stop: oscillator.stops[0] ?? 0,
      envelope: gain?.gain.points ?? [],
      audible: gain !== undefined && gain.outputs.includes(context.destination),
    }
  })
}

/** One context as a test reads it, live. */
function reading(context: Context): AudioContextStub {
  return {
    get state() {
      return context.state
    },
    get resumes() {
      return context.resumes
    },
    get closes() {
      return context.closes
    },
    get notes() {
      return notesOf(context)
    },
  }
}

/**
 * Installs the window's `AudioContext` for one test and reports what the app
 * played. Every context the app creates is kept, so "one per window" is a
 * reading rather than a promise the app could break silently.
 */
export function audioWindow(options: AudioOptions = {}): AudioWindow {
  const created: Context[] = []

  // A plain function used with `new`: returning the context makes it the result,
  // which is what the real constructor also hands back.
  function Stub(this: unknown): Context {
    if (options.refuses === true) throw new Error('no audio device')
    const context = new Context(options, options.state ?? 'running')
    created.push(context)
    return context
  }

  vi.stubGlobal('AudioContext', options.unsupported === true ? undefined : Stub)

  return {
    get contexts() {
      return created.map(reading)
    },
    notes(): AudioNote[] {
      return created.flatMap(notesOf)
    },
  }
}
