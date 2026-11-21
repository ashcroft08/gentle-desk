// Version equality as the update rules read it (0.8.0 settings-polish S2): the
// version the user skipped and the one a row publishes are compared as VERSIONS,
// never as text, and a value that is not a strict `x.y.z` is no version at all -
// so a skip this app cannot read hides nothing.
//
// The one shape is `gentle_runtime::updates::parse`'s: three dot-separated
// numbers, trimmed, nothing else. This file pins the two halves the pane and the
// gear's dot share.
import { describe, expect, it } from 'vitest'
import { parseVersion, sameVersion } from './version'

describe('a version as the update rules read it', () => {
  it('reads three numbers, and trims what surrounds them', () => {
    expect(parseVersion('2.0.5')).toEqual([2, 0, 5])
    expect(parseVersion(' 2.0.5 ')).toEqual([2, 0, 5])
    expect(parseVersion('0.8.0')).toEqual([0, 8, 0])
    expect(parseVersion('12.34.56')).toEqual([12, 34, 56])
  })

  it('reads nothing else', () => {
    for (const text of ['', '   ', '2', '2.0', '2.0.5.1', 'v2.0.5', '2.0.5-beta.1', 'latest', '2.x.5', '+2.0.5', '-2.0.5', null, undefined]) {
      expect(parseVersion(text), String(text)).toBeNull()
    }
  })

  it('calls two spellings of one version the same version', () => {
    expect(sameVersion('2.0.5', '2.0.5')).toBe(true)
    expect(sameVersion('2.0.5', ' 02.0.5 ')).toBe(true)
    expect(sameVersion('0.8.0', '0.8.00')).toBe(true)
  })

  it('never calls two different versions the same, nor a value it cannot read', () => {
    expect(sameVersion('2.0.5', '2.0.6')).toBe(false)
    expect(sameVersion('2.0.5', '2.0.50')).toBe(false)
    expect(sameVersion('latest', 'latest')).toBe(false)
    expect(sameVersion(null, null)).toBe(false)
    expect(sameVersion('2.0.5', null)).toBe(false)
    expect(sameVersion(null, '2.0.5')).toBe(false)
  })
})
