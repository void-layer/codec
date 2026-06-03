// @ts-check
import js from '@eslint/js';
import tseslint from 'typescript-eslint';

export default tseslint.config(
  {
    ignores: [
      '**/node_modules/**',
      '**/dist/**',
      '**/pkg/**',
      '**/pkg-node/**',
      '**/pkg-web/**',
      '**/target/**',
      '.changeset/**',
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    // scripts/ are Node.js ESM tooling (not published). Declare Node globals so
    // no-undef doesn't fire on console/process/URL/etc., and relax TS rules
    // that are irrelevant for plain .mjs scripts.
    files: ['packages/*/scripts/**/*.mjs', 'packages/*/scripts/**/*.js'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module',
      globals: {
        console: 'readonly',
        process: 'readonly',
        URL: 'readonly',
      },
    },
    rules: {
      '@typescript-eslint/no-require-imports': 'off',
    },
  },
  {
    files: ['packages/*/src/**/*.ts'],
    languageOptions: {
      ecmaVersion: 2022,
      sourceType: 'module',
    },
    rules: {
      '@typescript-eslint/no-explicit-any': 'error',
      '@typescript-eslint/consistent-type-imports': 'error',
      // Constitution VI — no RPC keys in source
      'no-restricted-syntax': [
        'error',
        {
          selector: "Literal[value=/alch_|alchemyapi\\.io\\/v2\\/|infura\\.io\\/v3\\//]",
          message: 'RPC keys must never appear in @void-layer source (Constitution VI). Server-side only in voidpay.xyz.',
        },
      ],
    },
  },
);
