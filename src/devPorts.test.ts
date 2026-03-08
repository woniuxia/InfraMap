import { readFileSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import { describe, expect, it } from 'vitest'

const currentDir = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(currentDir, '..')

function readViteConfigText() {
  return readFileSync(path.join(repoRoot, 'vite.config.ts'), 'utf8')
}

describe('dev port configuration', () => {
  it('keeps Vite and Tauri dev ports aligned', () => {
    const viteConfigText = readViteConfigText()
    const tauriConfig = JSON.parse(
      readFileSync(path.join(repoRoot, 'src-tauri', 'tauri.conf.json'), 'utf8')
    ) as {
      build?: {
        devUrl?: string
      }
    }

    expect(viteConfigText).toContain('strictPort: true')
    expect(viteConfigText).toContain('port: 15420')
    expect(viteConfigText).toContain('port: 15421')
    expect(tauriConfig.build?.devUrl).toBe('http://localhost:15420')
  })
})
