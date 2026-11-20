// The session permission mode's own naming (composer-dock T3c, L7/L8): the four
// wire strings the channel speaks, and the label each one is drawn with.
//
// The mapping lives here and not in a language dictionary, because the four
// names are the CHANNEL's own - `normal`, `auto`, `yolo`, `super-yolo` - and not
// words of ours: `Normal`, `Auto`, `YOLO` and `Super YOLO` read the same in both
// languages, exactly as a model's name does (the same reason `tokens` stays out
// of `composer/copy.ts`). Anything the channel sends that this build does not
// know is shown RAW: the mode a session really runs is never hidden and never
// mapped onto one the page does know.
//
// The two widest modes are the ones the page asks about first (`yolo`,
// `super-yolo`); which of them needs a confirmation is the design's ruling
// (`02-mode-picker.html`, sheet B), which is why it is a rule here instead of a
// field of the contract's enum.
import type { SessionModeChoice } from '../contract/bindings'

/** The four modes the page may ask for, in the order the menu lists them. */
export const MODE_CHOICES: SessionModeChoice[] = ['normal', 'auto', 'yolo', 'super-yolo']

/** What the control and the menu call a mode, and the raw value as it came. */
export function modeLabel(mode: string): string {
  switch (mode) {
    case 'normal':
      return 'Normal'
    case 'auto':
      return 'Auto'
    case 'yolo':
      return 'YOLO'
    case 'super-yolo':
      return 'Super YOLO'
    default:
      return mode
  }
}

/** Whether a mode is one of the two the row draws in the danger tone. */
export function modeIsDanger(mode: string): boolean {
  return mode === 'yolo' || mode === 'super-yolo'
}
