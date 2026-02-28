import { ref, reactive } from 'vue'
import type { Ref } from 'vue'
import type { QueryParams, PagedResult } from '@/types'
import { ElMessage, ElMessageBox } from 'element-plus'

interface UseResourceListOptions<T> {
  listFn: (params: QueryParams) => Promise<PagedResult<T>>
  deleteFn: (id: string) => Promise<void>
  entityLabel: string
}

interface QueryUpdate {
  search?: string
  filters?: Record<string, string | undefined | null>
}

export function useResourceList<T extends { id: string }>(options: UseResourceListOptions<T>) {
  const loading = ref(false)
  const data = ref<T[]>([]) as Ref<T[]>
  const total = ref(0)
  let requestToken = 0
  const queryParams = reactive<QueryParams>({
    page: 1,
    page_size: 20,
    search: '',
    filters: {},
  })

  function normalizeText(value: string | undefined | null): string {
    return (value ?? '').trim()
  }

  function normalizeFilters(
    filters: Record<string, string | undefined | null> | undefined
  ): Record<string, string> {
    if (!filters) return {}
    const nextFilters: Record<string, string> = {}
    for (const [key, value] of Object.entries(filters)) {
      const normalized = normalizeText(value)
      if (normalized) {
        nextFilters[key] = normalized
      }
    }
    return nextFilters
  }

  async function fetchData() {
    const currentToken = ++requestToken
    loading.value = true
    try {
      const requestParams: QueryParams = {
        ...queryParams,
        filters: { ...(queryParams.filters ?? {}) },
      }
      const result = await options.listFn(requestParams)
      if (currentToken !== requestToken) return
      data.value = result.data
      total.value = result.total
    } catch {
      // error already shown by tauriInvoke
    } finally {
      if (currentToken !== requestToken) return
      loading.value = false
    }
  }

  function handleSearch(search: string) {
    queryParams.search = normalizeText(search)
    queryParams.page = 1
    fetchData()
  }

  function handleQuery(update: QueryUpdate) {
    if (update.search !== undefined) {
      queryParams.search = normalizeText(update.search)
    }
    if (update.filters !== undefined) {
      queryParams.filters = normalizeFilters(update.filters)
    }
    queryParams.page = 1
    fetchData()
  }

  function handleFilters(filters: Record<string, string | undefined | null>) {
    handleQuery({
      filters: {
        ...(queryParams.filters ?? {}),
        ...filters,
      },
    })
  }

  function resetFilters() {
    handleQuery({ filters: {} })
  }

  function handleFilter(key: string, value: string | undefined | null) {
    handleFilters({ [key]: value })
  }

  function handlePageChange(page: number) {
    queryParams.page = page
    fetchData()
  }

  function handlePageSizeChange(size: number) {
    queryParams.page_size = size
    queryParams.page = 1
    fetchData()
  }

  async function handleDelete(id: string, name?: string) {
    try {
      await ElMessageBox.confirm(
        `确认删除${options.entityLabel} "${name || id}" ?`,
        '确认删除',
        { confirmButtonText: '确定', cancelButtonText: '取消', type: 'warning' }
      )
      await options.deleteFn(id)
      ElMessage.success('删除成功')
      fetchData()
    } catch {
      // cancelled or error (error already shown by tauriInvoke)
    }
  }

  return {
    loading,
    data,
    total,
    queryParams,
    fetchData,
    handleQuery,
    handleSearch,
    handleFilters,
    resetFilters,
    handleFilter,
    handlePageChange,
    handlePageSizeChange,
    handleDelete,
  }
}
