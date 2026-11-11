// The update line's words, in one place.

// On Windows the updater closes the app by itself to hand over to the installer,
// so the line says so before the button is pressed.
export const found = (version: string): string =>
  `Version ${version} is available. The app will close while it installs.`
export const DOWNLOADING = (version: string, percent: number | null): string =>
  percent === null
    ? `Downloading version ${version}.`
    : `Downloading version ${version}: ${percent}%.`
export const READY = (version: string): string =>
  `Version ${version} is installed. Open the app again to use it.`
export const FAILED = (version: string | null, message: string): string =>
  `The update${version === null ? '' : ` to version ${version}`} could not be installed: ${message}`

export const INSTALL = 'Install update'
export const RETRY = 'Try again'
export const DISMISS = 'Not now'
