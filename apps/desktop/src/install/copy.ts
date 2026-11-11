// The words of the missing gentle-shell screen (shell-choice S4). It says what
// will run and what it changes before it asks, because the permission only
// means something if the user knows what they are allowing. It also says what
// the command installs - the program, not a home - and that where chats and
// sign-ins live is the question after it, so the install is not read as "all of
// gentle-shell".

export const TITLE = 'gentle-shell is not installed'
export const WHY =
  'This app talks to your models through gentle-shell, and it was not found on this machine. Without it nothing here can work.'
export const COMMAND_INTRO = 'Installing it runs this command:'
export const COMMAND = 'npm install -g gentle-pi'
export const INSTALLS =
  'This installs the gentle-shell program only, about 250 MB, in the npm global folder. It makes no place for chats or sign-ins.'
export const HOME =
  'Where your chats and sign-ins live is a separate choice, made next: your own pi, or a space of its own.'
export const WARNING =
  'The install changes this computer: it needs Node and npm, and it can take a few minutes.'

export const INSTALL = 'Install gentle-shell'
export const CHECK_AGAIN = 'Check again'

export const RUNNING = 'Installing gentle-shell. This can take a few minutes.'
export const DONE = 'gentle-shell was installed. Check again to continue.'
export const FAILED_TITLE = 'The install did not finish'
export const FAILED_HINT =
  'If npm could not be found, install Node.js (nodejs.org), then press Install again.'
