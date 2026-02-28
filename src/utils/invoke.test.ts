import { describe, it, expect, vi, beforeEach } from 'vitest'
import { __setMockHandler, __clearMockHandlers } from '@/__mocks__/tauri'

// Mock element-plus ElMessage
vi.mock('element-plus', () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
  ElMessageBox: {
    confirm: vi.fn(),
  },
}))

import { tauriInvoke } from '@/utils/invoke'
import { ElMessage } from 'element-plus'

describe('tauriInvoke', () => {
  beforeEach(() => {
    __clearMockHandlers()
    vi.clearAllMocks()
  })

  it('should return data on successful invoke', async () => {
    __setMockHandler('test_cmd', () => ({ result: 'ok' }))
    const data = await tauriInvoke<{ result: string }>('test_cmd')
    expect(data).toEqual({ result: 'ok' })
  })

  it('should show error message and throw on failed invoke', async () => {
    __setMockHandler('fail_cmd', () => {
      throw 'Something went wrong'
    })

    await expect(tauriInvoke('fail_cmd')).rejects.toThrow('Something went wrong')
    expect(ElMessage.error).toHaveBeenCalledWith('Something went wrong')
  })

  it('should handle non-string errors', async () => {
    __setMockHandler('fail_cmd2', () => {
      throw { code: 500 }
    })

    await expect(tauriInvoke('fail_cmd2')).rejects.toThrow('Unknown error')
    expect(ElMessage.error).toHaveBeenCalledWith('Unknown error')
  })
})
