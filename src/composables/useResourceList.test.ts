import { describe, it, expect, vi, beforeEach } from 'vitest'
import { __setMockHandler, __clearMockHandlers } from '@/__mocks__/tauri'

// Mock element-plus
vi.mock('element-plus', () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
  ElMessageBox: {
    confirm: vi.fn().mockResolvedValue(true),
  },
}))

import { useResourceList } from '@/composables/useResourceList'
import { ElMessage, ElMessageBox } from 'element-plus'

function createMockListFn(data: Array<{ id: string }> = [], total = 0) {
  return vi.fn().mockResolvedValue({ data, total, page: 1, page_size: 20 })
}

describe('useResourceList', () => {
  beforeEach(() => {
    __clearMockHandlers()
    vi.clearAllMocks()
  })

  it('fetchData should populate data and total', async () => {
    const mockData = [{ id: '1' }, { id: '2' }]
    const listFn = createMockListFn(mockData, 2)
    const deleteFn = vi.fn()
    const { data, total, fetchData } = useResourceList({
      listFn,
      deleteFn,
      entityLabel: 'Test',
    })

    await fetchData()
    expect(data.value).toEqual(mockData)
    expect(total.value).toBe(2)
    expect(listFn).toHaveBeenCalled()
  })

  it('handleSearch should reset page to 1', async () => {
    const listFn = createMockListFn()
    const deleteFn = vi.fn()
    const { queryParams, handleSearch } = useResourceList({
      listFn,
      deleteFn,
      entityLabel: 'Test',
    })

    queryParams.page = 5
    handleSearch('test query')
    expect(queryParams.page).toBe(1)
    expect(queryParams.search).toBe('test query')
  })

  it('handlePageChange should update page', async () => {
    const listFn = createMockListFn()
    const deleteFn = vi.fn()
    const { queryParams, handlePageChange } = useResourceList({
      listFn,
      deleteFn,
      entityLabel: 'Test',
    })

    handlePageChange(3)
    expect(queryParams.page).toBe(3)
  })

  it('handleFilter should set filter and reset page', async () => {
    const listFn = createMockListFn()
    const deleteFn = vi.fn()
    const { queryParams, handleFilter } = useResourceList({
      listFn,
      deleteFn,
      entityLabel: 'Test',
    })

    queryParams.page = 3
    handleFilter('status', 'running')
    expect(queryParams.page).toBe(1)
    expect(queryParams.filters!['status']).toBe('running')
  })

  it('handleFilters should merge and clear filters in one request', async () => {
    const listFn = createMockListFn()
    const deleteFn = vi.fn()
    const { queryParams, handleFilters } = useResourceList({
      listFn,
      deleteFn,
      entityLabel: 'Test',
    })

    handleFilters({ status: 'running', env: 'prod' })
    expect(queryParams.filters).toEqual({ status: 'running', env: 'prod' })

    handleFilters({ status: '' })
    expect(queryParams.filters).toEqual({ env: 'prod' })
  })

  it('resetFilters should clear all filters and keep search unchanged', async () => {
    const listFn = createMockListFn()
    const deleteFn = vi.fn()
    const { queryParams, handleSearch, handleFilters, resetFilters } = useResourceList({
      listFn,
      deleteFn,
      entityLabel: 'Test',
    })

    handleSearch('app')
    handleFilters({ status: 'running', env: 'prod' })
    resetFilters()

    expect(queryParams.filters).toEqual({})
    expect(queryParams.search).toBe('app')
  })

  it('fetchData should only apply latest request result', async () => {
    const listFn = vi
      .fn()
      .mockImplementationOnce(
        () =>
          new Promise((resolve) =>
            setTimeout(() => resolve({ data: [{ id: 'old' }], total: 1, page: 1, page_size: 20 }), 20)
          )
      )
      .mockResolvedValueOnce({ data: [{ id: 'new' }], total: 1, page: 1, page_size: 20 })

    const deleteFn = vi.fn()
    const { data, fetchData } = useResourceList({
      listFn,
      deleteFn,
      entityLabel: 'Test',
    })

    const first = fetchData()
    const second = fetchData()
    await Promise.all([first, second])

    expect(data.value).toEqual([{ id: 'new' }])
  })

  it('handlePageSizeChange should update page_size and reset page', async () => {
    const listFn = createMockListFn()
    const deleteFn = vi.fn()
    const { queryParams, handlePageSizeChange } = useResourceList({
      listFn,
      deleteFn,
      entityLabel: 'Test',
    })

    queryParams.page = 3
    handlePageSizeChange(50)
    expect(queryParams.page_size).toBe(50)
    expect(queryParams.page).toBe(1)
  })

  it('handleDelete should call deleteFn and refresh after confirmation', async () => {
    const listFn = createMockListFn()
    const deleteFn = vi.fn().mockResolvedValue(undefined)
    const { handleDelete } = useResourceList({
      listFn,
      deleteFn,
      entityLabel: 'Host',
    })

    await handleDelete('1', 'server1')
    expect(ElMessageBox.confirm).toHaveBeenCalled()
    expect(deleteFn).toHaveBeenCalledWith('1')
    expect(ElMessage.success).toHaveBeenCalledWith('删除成功')
  })

  it('handleDelete should not call deleteFn when confirmation is cancelled', async () => {
    vi.mocked(ElMessageBox.confirm).mockRejectedValueOnce(new Error('cancel'))

    const listFn = createMockListFn()
    const deleteFn = vi.fn().mockResolvedValue(undefined)
    const { handleDelete } = useResourceList({
      listFn,
      deleteFn,
      entityLabel: 'Host',
    })

    await handleDelete('1', 'server1')

    expect(ElMessageBox.confirm).toHaveBeenCalled()
    expect(deleteFn).not.toHaveBeenCalled()
    expect(ElMessage.success).not.toHaveBeenCalled()
  })
})
