import { ref, reactive } from 'vue'
import type { Ref } from 'vue'
import type { QueryParams, PagedResult } from '@/types'
import { ElMessage, ElMessageBox } from 'element-plus'

interface UseResourceListOptions<T> {
  listFn: (params: QueryParams) => Promise<PagedResult<T>>
  deleteFn: (id: string) => Promise<void>
  entityLabel: string
}

export function useResourceList<T extends { id: string }>(options: UseResourceListOptions<T>) {
  const loading = ref(false)
  const data = ref<T[]>([]) as Ref<T[]>
  const total = ref(0)
  const queryParams = reactive<QueryParams>({
    page: 1,
    page_size: 20,
    search: '',
    filters: {},
  })

  async function fetchData() {
    loading.value = true
    try {
      const result = await options.listFn({ ...queryParams })
      data.value = result.data
      total.value = result.total
    } catch {
      // error already shown by tauriInvoke
    } finally {
      loading.value = false
    }
  }

  function handleSearch(search: string) {
    queryParams.search = search
    queryParams.page = 1
    fetchData()
  }

  function handleFilter(key: string, value: string) {
    if (!queryParams.filters) queryParams.filters = {}
    if (value) {
      queryParams.filters[key] = value
    } else {
      delete queryParams.filters[key]
    }
    queryParams.page = 1
    fetchData()
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
    handleSearch,
    handleFilter,
    handlePageChange,
    handlePageSizeChange,
    handleDelete,
  }
}
