import { defineTestConfig } from '@/utils'
import * as t from 'vitest'

export default defineTestConfig({
  options: {
    shimMissingExports: true,
  },
  exports(exports) {
    t.expect('missingDefault' in exports).toBe(true)
    t.expect(exports.missingDefault).toBe(undefined)
    t.expect('missingNamed' in exports).toBe(true)
    t.expect(exports.missingNamed).toBe(undefined)
  },
})
