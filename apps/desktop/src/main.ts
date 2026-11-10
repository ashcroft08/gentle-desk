import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import { startTheme } from './theme/paint'

// The theme is painted BEFORE the app mounts: the first read of the state names
// the theme of this run, and nothing of the UI exists while it is in flight, so
// the window never draws another theme's colors first. `startTheme` never
// rejects: a bridge that cannot answer leaves `index.html`'s default theme.
void startTheme().then(() => {
  mount(App, { target: document.getElementById('app')! })
})
