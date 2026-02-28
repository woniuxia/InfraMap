import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { TopologyGraph } from '@/types'
import { getTopologyGraph as fetchTopologyGraph } from '@/api/topology'

const CACHE_TTL_MS = 30_000 // 30 seconds

export const useTopologyStore = defineStore('topology', () => {
  const graphData = ref<TopologyGraph | null>(null)
  const lastFetchTime = ref<number>(0)
  const loading = ref(false)

  // Index node names for fast search
  const nodeNameIndex = computed(() => {
    const map = new Map<string, string[]>()
    if (!graphData.value) return map

    for (const node of graphData.value.nodes) {
      const words = node.name.toLowerCase().split(/[\s\-_./]+/)
      for (const word of words) {
        if (!word) continue
        const ids = map.get(word) || []
        ids.push(node.id)
        map.set(word, ids)
      }
    }
    return map
  })

  async function fetchGraph(forceRefresh = false): Promise<TopologyGraph | null> {
    const now = Date.now()
    if (!forceRefresh && graphData.value && now - lastFetchTime.value < CACHE_TTL_MS) {
      return graphData.value
    }

    loading.value = true
    try {
      graphData.value = await fetchTopologyGraph()
      lastFetchTime.value = Date.now()
      return graphData.value
    } catch {
      return null
    } finally {
      loading.value = false
    }
  }

  function invalidateCache() {
    lastFetchTime.value = 0
  }

  return {
    graphData,
    lastFetchTime,
    loading,
    nodeNameIndex,
    fetchGraph,
    invalidateCache,
  }
})
