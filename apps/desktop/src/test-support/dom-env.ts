// The browser APIs jsdom does not implement but the headless primitives need
// (proven in T2's a11y lab): floating-ui reads element rects and the viewport to
// place a menu, and bits-ui calls pointer capture and scrollIntoView as soon as
// a popover or a dialog opens. Every stub below stands in for a missing browser
// API; none of them decides anything the components under test decide.
//
// Import it at the top of a test file, before mounting a component.

type Stub = Record<string, unknown>
const globals = globalThis as unknown as Stub

/** The slice of `ResizeObserver` floating-ui builds. */
class ResizeObserverStub {
  observe(): void {}
  unobserve(): void {}
  disconnect(): void {}
}

globals.ResizeObserver ??= ResizeObserverStub

if (typeof window.matchMedia === 'undefined') {
  window.matchMedia = (query: string): MediaQueryList =>
    ({
      matches: false,
      media: query,
      onchange: null,
      addListener(): void {},
      removeListener(): void {},
      addEventListener(): void {},
      removeEventListener(): void {},
      dispatchEvent: () => false,
    }) as unknown as MediaQueryList
}

const sized = (width: number, height: number): DOMRect => new DOMRect(10, 10, width, height)

// jsdom reports every rect as 0x0, which floating-ui's hide middleware reads as
// "the reference is not visible" and hides the whole popover.
window.HTMLElement.prototype.getBoundingClientRect = function rect(this: HTMLElement): DOMRect {
  const tag = this.tagName
  return tag === 'HTML' || tag === 'BODY' ? new DOMRect(0, 0, 1024, 768) : sized(100, 20)
}

window.HTMLElement.prototype.getClientRects = function rects(this: HTMLElement): DOMRectList {
  const r = this.getBoundingClientRect()
  const list: DOMRect[] = [r]
  return list as unknown as DOMRectList
}

// The same hide middleware measures the viewport, which jsdom reports as 0x0.
Object.defineProperty(window, 'innerWidth', { value: 1024, configurable: true })
Object.defineProperty(window, 'innerHeight', { value: 768, configurable: true })
for (const el of [document.documentElement, document.body]) {
  for (const prop of ['clientWidth', 'offsetWidth']) {
    Object.defineProperty(el, prop, { value: 1024, configurable: true })
  }
  for (const prop of ['clientHeight', 'offsetHeight']) {
    Object.defineProperty(el, prop, { value: 768, configurable: true })
  }
}

Element.prototype.scrollIntoView = function scrollIntoView(): void {}

const captureable = Element.prototype as unknown as Stub
captureable.hasPointerCapture ??= (): boolean => false
captureable.releasePointerCapture ??= (): void => {}
captureable.setPointerCapture ??= (): void => {}
