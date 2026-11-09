import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'

// A11y lab (T2c): mounted only when the URL hash selects a candidate, through
// a dynamic import. Without that hash the normal app path below is unchanged.
const lab = window.location.hash.match(/^#a11y-lab\/(bitsui|meltui)$/)

if (lab) {
  const labModule =
    lab[1] === 'bitsui'
      ? import('./a11y-lab/bitsui/Lab.svelte')
      : import('./a11y-lab/meltui/Lab.svelte')
  labModule.then((m) => {
    mount(m.default, { target: document.getElementById('app')! })
  })
} else {
  mount(App, { target: document.getElementById('app')! })
}
