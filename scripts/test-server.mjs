import { spawn, spawnSync } from 'node:child_process'
import { mkdtempSync, rmSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
const directory = mkdtempSync(join(tmpdir(), 'rust-mvc-e2e-'))
const env = {
  ...process.env,
  DATABASE_URL: `sqlite://${join(directory, 'test.sqlite')}`,
  APP_HOST: '127.0.0.1',
  APP_PORT: '3100',
  COOKIE_SECURE: 'false',
  SEED_ADMIN_EMAIL: 'admin@example.com',
  SEED_ADMIN_PASSWORD: 'e2e-test-password-123',
}
const setup = spawnSync('cargo', ['run', '--quiet', '--bin', 'forge', '--', 'migrate', '--seed'], {
  env,
  stdio: 'inherit',
})
if (setup.status !== 0) {
  rmSync(directory, { recursive: true, force: true })
  process.exit(setup.status || 1)
}
const server = spawn('cargo', ['run', '--quiet', '--bin', 'forge', '--', 'serve'], {
  env,
  stdio: 'inherit',
})
server.on('exit', (code) => {
  rmSync(directory, { recursive: true, force: true })
  process.exitCode = code || 0
})
process.on('SIGTERM', () => server.kill('SIGTERM'))
process.on('SIGINT', () => server.kill('SIGINT'))
