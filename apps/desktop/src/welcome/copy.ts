// The welcome question's words, after the gentle-shell welcome screen the user
// pointed to (shell-choice S2). It says nothing about what is inside pi: the app
// only checked that the folder exists, so the only fact it names is the path.

export const TITLE = 'Welcome to gentle shell'
export const INTRO = 'Everything you need is already inside this app. One question before you start.'
export const FOUND_TITLE = 'We found pi on this machine'

export const LINK_TITLE = 'Use my pi setup'
export const LINK_BADGE = 'RECOMMENDED'
export const LINK_POINTS = [
  'Your sign-ins, local models and chats appear here',
  'Chats you start here also show up in your terminal',
  'Your pi settings are never edited',
]
export const LINK_PATH = 'links to ~/.pi/agent'

export const SEPARATE_TITLE = 'Keep it separate'
export const SEPARATE_POINTS = [
  'A clean space with its own sign-ins and chats',
  'Nothing on this machine is touched',
  'You can change this later from the sidebar footer',
]
export const SEPARATE_PATH = 'creates ~/.gentle-shell/agent'

export const SKIP_NOTE =
  'No pi on the machine? This screen is skipped and gentle shell starts in its own space.'
