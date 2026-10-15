import { spawnSync } from 'node:child_process'
import { mkdirSync, copyFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const root = fileURLToPath(new URL('..', import.meta.url))
process.chdir(root)
const args = process.argv.slice(2)
let mode = 'server',
  target =
    process.platform === 'win32' ? 'windows' : process.platform === 'darwin' ? 'macos' : 'linux'
let arch = process.arch,
  debug = false
for (let i = 0; i < args.length; i++) {
  if (args[i] === '--desktop') mode = 'desktop'
  else if (args[i] === '--server') mode = 'server'
  else if (args[i] === '--target') target = args[++i]
  else if (args[i] === '--arch') arch = args[++i]
  else if (args[i] === '--debug') debug = true
  else if (args[i] === '--help') {
    console.log(
      'Usage: node scripts/build.mjs [--server|--desktop] [--target windows|linux|macos] [--arch x64|arm64] [--debug]\nBuild on the target OS. Output: releases/<mode>/<target>-<arch>/',
    )
    process.exit(0)
  } else throw new Error(`Unknown argument: ${args[i]}`)
}
const platforms = { windows: 'win32', linux: 'linux', macos: 'darwin' }
if (!(target in platforms) || !['x64', 'arm64'].includes(arch))
  throw new Error('Use --target windows|linux|macos and --arch x64|arm64')
if (platforms[target] !== process.platform || arch !== process.arch) {
  throw new Error(
    `Build ${target}-${arch} on a matching machine or use the Standalone builds GitHub workflow. This command does not install cross-compilers or webview SDKs.`,
  )
}
function run(command, args) {
  const result = spawnSync(command, args, { stdio: 'inherit', shell: process.platform === 'win32' })
  if (result.error) throw result.error
  if (result.status !== 0) process.exit(result.status || 1)
}
run('npm', ['run', 'build'])
const binary = mode === 'desktop' ? 'rustmvc-desktop' : 'forge'
const features = mode === 'desktop' ? 'desktop' : 'embedded-assets'
run('cargo', [
  'build',
  '--locked',
  '--bin',
  binary,
  '--features',
  features,
  ...(debug ? [] : ['--release']),
])
const suffix = target === 'windows' ? '.exe' : ''
const destination = resolve('releases', mode, `${target}-${arch}`)
mkdirSync(destination, { recursive: true })
const cargoTarget = process.env.CARGO_TARGET_DIR
  ? resolve(process.env.CARGO_TARGET_DIR)
  : resolve('target')
copyFileSync(
  resolve(cargoTarget, debug ? 'debug' : 'release', `${binary}${suffix}`),
  resolve(destination, `${mode === 'desktop' ? 'RustMVC' : 'rustmvc-server'}${suffix}`),
)
copyFileSync('.env.example', resolve(destination, '.env.example'))
copyFileSync('docs/STANDALONE.md', resolve(destination, 'README.md'))
writeFileSync(
  resolve(destination, 'build-info.json'),
  JSON.stringify({ mode, target, arch, profile: debug ? 'debug' : 'release' }, null, 2) + '\n',
)
console.log(`\nBuilt ${mode}: ${destination}`)
