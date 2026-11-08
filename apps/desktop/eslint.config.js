import js from '@eslint/js'
import svelte from 'eslint-plugin-svelte'
import tseslint from 'typescript-eslint'
import globals from 'globals'

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  ...svelte.configs['flat/recommended'],
  {
    files: ['**/*.svelte'],
    languageOptions: {
      parserOptions: {
        parser: { ts: tseslint.parser, typescript: tseslint.parser }
      }
    }
  },
  {
    files: ['**/*.{ts,js,svelte}'],
    languageOptions: {
      globals: { ...globals.browser, ...globals.node }
    }
  },
  {
    files: ['**/*.{ts,js,svelte}'],
    rules: {
      'max-lines': [
        'error',
        { max: 200, skipBlankLines: true, skipComments: true }
      ]
    }
  },
  {
    ignores: ['dist/', 'node_modules/', 'src-tauri/']
  }
)
