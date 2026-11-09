// The module runner's source-map stack transform can itself throw on some
// frames ("column must be >= 0"), which then masks the real error thrown from
// a Svelte effect. Plain V8 stacks keep the original error visible.
Error.prepareStackTrace = (err, trace) => {
  return `${err.name}: ${err.message}` + trace.map((f) => `\n    at ${String(f)}`).join('')
}

// Globals that jsdom lacks but the floating-position code of the candidates
// calls as soon as a popover opens. Import this file at the top of each lab
// test file, before mounting components.
// eslint-disable-next-line @typescript-eslint/no-explicit-any
;(globalThis as any).ResizeObserver ??= class ResizeObserverStub {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

if (typeof window.matchMedia === 'undefined') {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  ;(window as any).matchMedia = () => ({
    matches: false,
    media: '',
    onchange: null,
    addListener(): void {},
    removeListener(): void {},
    addEventListener(): void {},
    removeEventListener(): void {},
    dispatchEvent: () => false
  })
}

// The pointer-capture aside, jsdom reports every element rect as 0x0, which
// floating-ui's `hide` middleware reads as "the reference is not visible" and
// switches the popover wrapper to visibility: hidden - a test-environment
// artifact, not library behaviour. Non-zero rects inside the jsdom viewport
// keep the floating layer visible.
if (typeof window !== 'undefined') {
  ;(window.HTMLElement.prototype as unknown as { getBoundingClientRect: unknown }).getBoundingClientRect =
    function stubRect(this: HTMLElement): DOMRect {
      const tag = this.tagName
      if (tag === 'HTML' || tag === 'BODY') {
        return new DOMRect(0, 0, 1024, 768)
      }
      return new DOMRect(10, 10, 100, 20)
    }
}

// The hide middleware also checks the reference against the viewport; jsdom
// reports zero-sized viewport dimensions, which would hide every popover.
if (typeof window !== 'undefined') {
  const viewportStubs: Array<[string, number]> = [
    ['innerWidth', 1024],
    ['innerHeight', 768]
  ]
  for (const [prop, value] of viewportStubs) {
    Object.defineProperty(window, prop, { value, configurable: true })
  }
  for (const el of [document.documentElement, document.body]) {
    for (const prop of ['clientWidth', 'clientHeight', 'offsetWidth', 'offsetHeight']) {
      Object.defineProperty(el, prop, { value: prop.endsWith('Height') ? 768 : 1024, configurable: true })
    }
  }
  ;(window.HTMLElement.prototype as unknown as { getClientRects: unknown }).getClientRects =
    function stubRects(this: HTMLElement): DOMRectList {
      const r = this.getBoundingClientRect()
      const list = Object.create(null) as DOMRectList
      // jsdom's DOMRectList type exposes only indexed access; one stubbed rect
      Object.defineProperty(list, 'length', { value: 1 })
      for (let i = 0; i < 1; i++) {
        ;(list as unknown as DOMRect[])[i] = r
      }
      return list
    }
}

// Nothing scrollable in jsdom; components call this on open.
Element.prototype.scrollIntoView ??= (): void => {}

// Pointer capture exists in real browsers; jsdom does not implement it and
// bits-ui calls it from its triggers' pointerdown handlers.
;;(Element.prototype as { hasPointerCapture?: unknown }).hasPointerCapture ??= (): boolean =>
  false
;;(Element.prototype as { releasePointerCapture?: unknown }).releasePointerCapture ??= (): void => {}
;;(Element.prototype as { setPointerCapture?: unknown }).setPointerCapture ??= (): void => {}
