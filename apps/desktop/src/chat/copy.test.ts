// The chat header's own words (0.2.0, S3 and D5): the tablist's name, the three
// tabs and the working badge live in ONE language-keyed dictionary, beside the
// rule module that keeps their order, their ids and their walk (`./tabs`). This
// file pins the two columns; the words a DRAWN tablist shows are pinned in
// `copy.flip.test.ts`.
//
// The English column is quoted literally - it is what 01b draws - so a change of
// copy is a change here. The chat's TITLE is not a word of this module: it is the
// session's own name and arrives exactly as stored (D7).
import { describe, expect, it } from 'vitest'
import { chatCopy } from './copy'

/** The English words, exactly as the header drew them before 0.2.0. */
const ENGLISH = {
  tabsLabel: 'View',
  tabs: { chat: 'Chat', helpers: 'Helpers', profile: 'Profile' },
  // The pencil that renames the chat in place (chat-chrome-helpers S2), and the
  // name its field carries (the row's own field word, reused).
  renameChat: 'Rename chat',
  chatName: 'Chat name',
  // Why the pencil renames nothing while no chat is open.
  renameReason: 'No chat is open to rename.',
}

/** The Spanish words: neutral Spanish, no voseo, no regionalisms, accents kept. */
const SPANISH = {
  tabsLabel: 'Vista',
  // "Chat" is the same word in Spanish, so it is shared on purpose.
  tabs: { chat: 'Chat', helpers: 'Ayudantes', profile: 'Perfil' },
  renameChat: 'Renombrar el chat',
  chatName: 'Nombre del chat',
  renameReason: 'Ningún chat abierto para renombrar.',
}

/** The one tab both languages spell the same. */
const SAME_IN_BOTH = new Set<'chat' | 'helpers' | 'profile'>(['chat'])

/** The plain words of one language: the badge's name is a line, not a word. */
function words(language: 'en' | 'es'): unknown {
  const copy = chatCopy(language)
  return {
    tabsLabel: copy.tabsLabel,
    tabs: copy.tabs,
    renameChat: copy.renameChat,
    chatName: copy.chatName,
    renameReason: copy.renameReason,
  }
}

describe('the chat copy', () => {
  it('draws the English words the header already had', () => {
    expect(words('en')).toEqual(ENGLISH)
  })

  it('draws Spanish for the same shape', () => {
    expect(words('es')).toEqual(SPANISH)
  })

  it('translates every word: nothing is left in English', () => {
    const english = chatCopy('en')
    const spanish = chatCopy('es')

    expect(spanish.tabsLabel).not.toBe(english.tabsLabel)
    expect(spanish.renameChat).not.toBe(english.renameChat)
    expect(spanish.chatName).not.toBe(english.chatName)
    expect(spanish.renameReason).not.toBe(english.renameReason)
    expect(Object.keys(spanish.tabs).sort()).toEqual(Object.keys(english.tabs).sort())
    for (const tab of ['chat', 'helpers', 'profile'] as const) {
      if (SAME_IN_BOTH.has(tab)) {
        expect(spanish.tabs[tab], tab).toBe(english.tabs[tab])
        continue
      }
      expect(spanish.tabs[tab], tab).not.toBe(english.tabs[tab])
    }
  })

  it('names the working badge in the language of the moment', () => {
    expect(chatCopy('en').workingBadge(1)).toBe('1 helper working')
    expect(chatCopy('en').workingBadge(3)).toBe('3 helpers working')
    // One helper is singular, every other count plural, in both languages.
    expect(chatCopy('es').workingBadge(1)).toBe('1 ayudante trabajando')
    expect(chatCopy('es').workingBadge(3)).toBe('3 ayudantes trabajando')
  })
})
