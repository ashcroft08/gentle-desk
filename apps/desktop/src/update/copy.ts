// The update line's words, in one place.

export const found = (version: string): string => `Version ${version} is available.`
export const DOWNLOADING = (version: string, percent: number | null): string =>
  percent === null
    ? `Downloading version ${version}.`
    : `Downloading version ${version}: ${percent}%.`
export const READY = (version: string): string =>
  `Version ${version} is installed. Restart the app to use it.`
export const FAILED = (version: string | null, message: string): string =>
  `The update${version === null ? '' : ` to version ${version}`} could not be installed: ${message}`

export const INSTALL = 'Install and restart'
export const RETRY = 'Try again'
export const DISMISS = 'Not now'
