// What one pushed ViewState says about the previous one (S3, S6): the three
// triggers the tone and the toast follow, and nothing else. Every case here goes
// through `detect` alone, because that is what T3 wires to the sound and the
// OS toast: no component, no store, no timers.

import { describe, expect, it } from 'vitest'
import { activity, confirmDialog, plainNotice, viewState } from '../test-support/view-state'
import { detect } from './detect'

describe('the first view after launch', () => {
  it('alerts nothing, whatever the state already holds', () => {
    const next = viewState({
      foreground_chat: 'c:one',
      activity: [activity('c:one', 'NeedsAttention'), activity('c:two', 'NewReply')],
      dialog: confirmDialog('d:1'),
      notices: [plainNotice(1, 'Chat died.', 'Error')],
    })

    expect(detect(null, next)).toEqual([])
  })
})

describe('a reply that finished', () => {
  it('alerts the chat that was Working and is gone from the activity list', () => {
    const prev = viewState({ activity: [activity('c:one', 'Working')] })
    const next = viewState({ activity: [] })

    expect(detect(prev, next)).toEqual([{ kind: 'reply', chatId: 'c:one' }])
  })

  it('alerts the chat that was Working and now carries NewReply', () => {
    const prev = viewState({ activity: [activity('c:one', 'Working')] })
    const next = viewState({ activity: [activity('c:one', 'NewReply')] })

    expect(detect(prev, next)).toEqual([{ kind: 'reply', chatId: 'c:one' }])
  })

  it('says nothing while a chat keeps working', () => {
    const prev = viewState({ activity: [activity('c:one', 'Working')] })
    const next = viewState({ activity: [activity('c:one', 'Working')] })

    expect(detect(prev, next)).toEqual([])
  })

  it('says nothing for a chat that appears already finished', () => {
    // A stored chat reopened, or a session list settling: the first time this
    // chat is seen it is already NewReply, so no turn finished while watching.
    const prev = viewState({ activity: [] })
    const next = viewState({ activity: [activity('c:one', 'NewReply')] })

    expect(detect(prev, next)).toEqual([])
  })

  it('says nothing for a chat that disappears without having worked', () => {
    const prev = viewState({ activity: [activity('c:one', 'NewReply')] })
    const next = viewState({ activity: [] })

    expect(detect(prev, next)).toEqual([])
  })

  it('alerts ONE reply for two chats that finished in the same push', () => {
    const prev = viewState({
      activity: [activity('c:one', 'Working'), activity('c:two', 'Working')],
    })
    const next = viewState({ activity: [] })

    expect(detect(prev, next)).toEqual([{ kind: 'reply', chatId: 'c:one' }])
  })

  it('is not the reply story when the chat stopped to ask', () => {
    // A turn that ends in a dialog finished no reply: it needs the user.
    const prev = viewState({ activity: [activity('c:one', 'Working')] })
    const next = viewState({ activity: [activity('c:one', 'NeedsAttention')] })

    expect(detect(prev, next)).toEqual([{ kind: 'needs-you', chatId: 'c:one' }])
  })

  it('is not the reply story when the chat died', () => {
    // A dead child is the failure story's; the turn never finished.
    const prev = viewState({ activity: [activity('c:one', 'Working')] })
    const next = viewState({ activity: [activity('c:one', 'Ended')] })

    expect(detect(prev, next)).toEqual([])
  })
})

describe('a chat that needs the user', () => {
  it('alerts a chat whose activity became NeedsAttention', () => {
    const prev = viewState({ activity: [] })
    const next = viewState({ foreground_chat: 'c:one', activity: [activity('c:one', 'NeedsAttention')] })

    expect(detect(prev, next)).toEqual([{ kind: 'needs-you', chatId: 'c:one' }])
  })

  it('alerts the first dialog that appears', () => {
    const prev = viewState({ foreground_chat: 'c:one', dialog: null })
    const next = viewState({ foreground_chat: 'c:one', dialog: confirmDialog('d:1') })

    expect(detect(prev, next)).toEqual([{ kind: 'needs-you', chatId: 'c:one' }])
  })

  it('says nothing while the same dialog stays up', () => {
    const prev = viewState({ foreground_chat: 'c:one', dialog: confirmDialog('d:1') })
    const next = viewState({ foreground_chat: 'c:one', dialog: confirmDialog('d:1') })

    expect(detect(prev, next)).toEqual([])
  })

  it('collapses a dialog and an activity that arrive together into one alert', () => {
    const prev = viewState({
      foreground_chat: 'c:one',
      dialog: null,
      activity: [activity('c:two', 'Working')],
    })
    const next = viewState({
      foreground_chat: 'c:one',
      dialog: confirmDialog('d:1'),
      activity: [activity('c:two', 'NeedsAttention')],
    })

    expect(detect(prev, next)).toEqual([{ kind: 'needs-you', chatId: 'c:one' }])
  })
})

describe('a new notice', () => {
  it('alerts a new Warning', () => {
    const prev = viewState({ notices: [] })
    const next = viewState({ notices: [plainNotice(1, 'Could not read 3 chat files.', 'Warning')] })

    expect(detect(prev, next)).toEqual([
      { kind: 'failure', text: 'Could not read 3 chat files.' },
    ])
  })

  it('alerts a new Error', () => {
    const prev = viewState({ notices: [] })
    const next = viewState({ notices: [plainNotice(2, 'Chat died.', 'Error')] })

    expect(detect(prev, next)).toEqual([{ kind: 'failure', text: 'Chat died.' }])
  })

  it('says nothing for a new Info notice', () => {
    const prev = viewState({ notices: [] })
    const next = viewState({ notices: [plainNotice(3, 'Skill registry refreshed', 'Info')] })

    expect(detect(prev, next)).toEqual([])
  })

  it('says nothing when the same notice id is pushed again', () => {
    const notice = plainNotice(4, 'Could not read 3 chat files.', 'Warning')
    const prev = viewState({ notices: [notice] })
    const next = viewState({ notices: [notice] })

    expect(detect(prev, next)).toEqual([])
  })

  it('alerts only the notice that is new, not the ones still up', () => {
    const prev = viewState({ notices: [plainNotice(5, 'Could not read 3 chat files.', 'Warning')] })
    const next = viewState({
      notices: [
        plainNotice(5, 'Could not read 3 chat files.', 'Warning'),
        plainNotice(6, 'Chat died.', 'Error'),
      ],
    })

    expect(detect(prev, next)).toEqual([{ kind: 'failure', text: 'Chat died.' }])
  })

  it('alerts ONE failure for two notices that arrive in the same push', () => {
    const prev = viewState({ notices: [] })
    const next = viewState({
      notices: [plainNotice(7, 'First warning.', 'Warning'), plainNotice(8, 'Then an error.', 'Error')],
    })

    expect(detect(prev, next)).toEqual([{ kind: 'failure', text: 'First warning.' }])
  })

  it('skips a new Info notice and alerts the new Error behind it', () => {
    const prev = viewState({ notices: [] })
    const next = viewState({
      notices: [plainNotice(9, 'Chat renamed.', 'Info'), plainNotice(10, 'Chat died.', 'Error')],
    })

    expect(detect(prev, next)).toEqual([{ kind: 'failure', text: 'Chat died.' }])
  })
})

describe('a push that carries several stories', () => {
  it('alerts one of each kind, in the order reply, needs-you, failure', () => {
    const prev = viewState({
      foreground_chat: 'c:one',
      dialog: null,
      activity: [activity('c:one', 'Working')],
      notices: [],
    })
    const next = viewState({
      foreground_chat: 'c:one',
      dialog: confirmDialog('d:1'),
      activity: [],
      notices: [plainNotice(11, 'Chat died.', 'Error')],
    })

    expect(detect(prev, next)).toEqual([
      { kind: 'reply', chatId: 'c:one' },
      { kind: 'needs-you', chatId: 'c:one' },
      { kind: 'failure', text: 'Chat died.' },
    ])
  })
})
