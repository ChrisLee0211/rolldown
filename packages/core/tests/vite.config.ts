import { defineConfig } from 'vitest/config'
export default defineConfig({
  resolve: {
    alias: {
      '@': __dirname,
    },
  },
  test: {
    include: ['./function/index.test.ts'],
  },
})
