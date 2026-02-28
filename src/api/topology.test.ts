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
}))

import { getTopologyGraph, findPaths, analyzeImpact } from '@/api/topology'

describe('topology API', () => {
  beforeEach(() => {
    __clearMockHandlers()
    vi.clearAllMocks()
  })

  it('getTopologyGraph should invoke get_topology_graph', async () => {
    const mockGraph = { nodes: [], edges: [], combos: [] }
    __setMockHandler('get_topology_graph', () => mockGraph)

    const result = await getTopologyGraph()
    expect(result).toEqual(mockGraph)
  })

  it('findPaths should pass source_id, target_id, max_results', async () => {
    const mockResult = { paths: [['A', 'B']], truncated: false }
    __setMockHandler('find_paths', (_cmd, args) => {
      expect(args).toEqual({ source_id: 'A', target_id: 'B', max_results: 5 })
      return mockResult
    })

    const result = await findPaths('A', 'B', 5)
    expect(result).toEqual(mockResult)
  })

  it('analyzeImpact should pass node_id', async () => {
    const mockResult = { affected_nodes: [], total_count: 0, max_depth: 0 }
    __setMockHandler('analyze_impact', (_cmd, args) => {
      expect(args).toEqual({ node_id: 'C' })
      return mockResult
    })

    const result = await analyzeImpact('C')
    expect(result).toEqual(mockResult)
  })
})
