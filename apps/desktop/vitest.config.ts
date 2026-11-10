import { defineConfig } from 'vitest/config'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],
  test: {
    // Vitest turns a stylesheet into an EMPTY module unless it is listed here,
    // and that includes a `?raw` import: the theme tests read the two stylesheets
    // as text, and an empty one would pass every check they make. Listing only
    // these two keeps every other stylesheet as cheap as before.
    css: { include: [/\/(theme|app)\.css/] },
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.{js,ts}']
  },
  // Run Svelte components in the browser build even under Vitest,
  // otherwise `mount(...)` resolves to the server variant.
  resolve: process.env.VITEST ? { conditions: ['browser'] } : undefined
})
