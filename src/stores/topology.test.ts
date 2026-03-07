import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

const { getTopologySnapshotV3Mock, getTopologyTaskViewV3Mock } = vi.hoisted(() => ({
  getTopologySnapshotV3Mock: vi.fn(),
  getTopologyTaskViewV3Mock: vi.fn(),
}))

vi.mock('@/api/topologyV3', () => ({
  getTopologySnapshotV3: getTopologySnapshotV3Mock,
  getTopologyTaskViewV3: getTopologyTaskViewV3Mock,
}))

import { useTopologyStore } from '@/stores/topology'

function createSnapshot() {
  return {
    meta: {
      version: '3.0',
      taskView: 'explore',
      generatedAt: '2026-03-07T00:00:00.000Z',
      nodeCount: 1,
      edgeCount: 0,
    },
    lanes: [{ id: 'prod', label: '生产', order: 0, nodeCount: 1, appCount: 1 }],
    nodes: [
      {
        id: 'node-1',
        name: '订单服务',
        nodeType: 'application',
        groupKind: 'application_service',
        env: 'prod',
        importance: 1,
      },
    ],
    edges: [],
    legendStats: {
      envCounts: [{ env: 'prod', count: 1, appCount: 1 }],
      nodeTypeCounts: [{ kind: 'application', count: 1 }],
      edgeTypeCounts: [],
      applicationServiceCount: 1,
      currentEnv: 'prod',
      externalNodeCount: 0,
      crossEnvEdgeCount: 0,
    },
    layoutHints: {
      laneOrder: ['prod', 'test', 'dev'],
      defaultCollapsedGroups: [],
      highDensityMode: false,
    },
  }
}

describe('useTopologyStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    getTopologySnapshotV3Mock.mockReset()
    getTopologyTaskViewV3Mock.mockReset()
  })

  it('loads explore snapshot from V3 snapshot api', async () => {
    getTopologySnapshotV3Mock.mockResolvedValue(createSnapshot())

    const store = useTopologyStore()
    const result = await store.fetchGraph()

    expect(getTopologySnapshotV3Mock).toHaveBeenCalledWith({ taskView: 'explore', maxDepth: 3 })
    expect(result?.nodes[0]?.node_type).toBe('application')
    expect(store.taskInsights).toEqual([])
  })

  it('does not fall back to snapshot api when task view response misses snapshot', async () => {
    getTopologyTaskViewV3Mock.mockResolvedValue({
      taskView: 'impact',
      insights: [],
      meta: {
        version: '3.0',
        taskView: 'impact',
        generatedAt: '2026-03-07T00:00:00.000Z',
        nodeCount: 0,
        edgeCount: 0,
      },
    })
    getTopologySnapshotV3Mock.mockResolvedValue(createSnapshot())

    const store = useTopologyStore()
    store.setTaskView('impact')
    const result = await store.fetchGraph()

    expect(result).toBeNull()
    expect(getTopologyTaskViewV3Mock).toHaveBeenCalledWith({ taskView: 'impact', maxDepth: 3 })
    expect(getTopologySnapshotV3Mock).not.toHaveBeenCalled()
  })
})
