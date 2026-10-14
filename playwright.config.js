import { defineConfig } from '@playwright/test'
export default defineConfig({
  testDir: './tests/browser',
  workers: 1,
  use: { baseURL: 'http://127.0.0.1:3100', trace: 'retain-on-failure' },
  webServer: {
    command: 'node scripts/test-server.mjs',
    url: 'http://127.0.0.1:3100/api/health',
    timeout: 180000,
    reuseExistingServer: false,
  },
})
