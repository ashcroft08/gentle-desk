import { defineConfig } from 'vitest/config'
import { svelte } from '@sveltejs/vite-plugin-svelte'

export default defineConfig({
  plugins: [svelte()],
  test: {
    environment: 'jsdom',
    include: ['src/**/*.{test,spec}.{js,ts}']
  },
  // Run Svelte components in the browser build even under Vitest,
  // otherwise `mount(...)` resolves to the server variant.
  resolve: process.env.VITEST ? { conditions: ['browser'] } : undefined
})
