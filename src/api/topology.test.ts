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
import {
  getTopologySnapshotV3,
  getTopologyDrilldownV3,
  getTopologyTaskViewV3,
  getTopologyPathsV3,
  getTopologyImpactV3,
  getTopologyEvidenceV3,
  getTopologyTroubleshootReportV3,
} from '@/api/topologyV3'
import type {
  TopologyV3DrilldownQuery,
  TopologyV3EvidenceQuery,
  TopologyV3ImpactQuery,
  TopologyV3PathsQuery,
  TopologyV3SnapshotQuery,
  TopologyV3TaskViewQuery,
  TopologyV3TroubleshootReportQuery,
} from '@/types'

describe('topology API', () => {
  beforeEach(() => {
    __clearMockHandlers()
    vi.clearAllMocks()
  })

  it('getTopologyGraph should invoke get_topology_graph', async () => {
    const mockGraph = {
      lanes: [
        { id: 'prod', label: '生产', order: 0, node_count: 0, app_count: 0 },
        { id: 'test', label: '测试', order: 1, node_count: 0, app_count: 0 },
        { id: 'dev', label: '开发', order: 2, node_count: 0, app_count: 0 },
      ],
      nodes: [],
      edges: [],
      legend_stats: {
        env_counts: [],
        node_type_counts: [],
        edge_type_counts: [],
        application_service_count: 0,
      },
      layout_hints: {
        lane_order: ['prod', 'test', 'dev'],
        default_collapsed_groups: [],
        high_density_mode: false,
      },
    }
    __setMockHandler('get_topology_graph', () => mockGraph)

    const result = await getTopologyGraph()
    expect(result).toEqual(mockGraph)
  })

  it('findPaths should pass sourceId, targetId, maxResults', async () => {
    const mockResult = { paths: [['A', 'B']], truncated: false }
    __setMockHandler('find_paths', (_cmd, args) => {
      expect(args).toEqual({ sourceId: 'A', targetId: 'B', maxResults: 5 })
      return mockResult
    })

    const result = await findPaths('A', 'B', 5)
    expect(result).toEqual(mockResult)
  })

  it('analyzeImpact should pass nodeId', async () => {
    const mockResult = { affected_nodes: [], total_count: 0, max_depth: 0 }
    __setMockHandler('analyze_impact', (_cmd, args) => {
      expect(args).toEqual({ nodeId: 'C' })
      return mockResult
    })

    const result = await analyzeImpact('C')
    expect(result).toEqual(mockResult)
  })
})

describe('topology V3 API', () => {
  beforeEach(() => {
    __clearMockHandlers()
    vi.clearAllMocks()
  })

  it('getTopologySnapshotV3 should invoke get_topology_snapshot_v3', async () => {
    const query: TopologyV3SnapshotQuery = { env: 'prod', taskView: 'explore', maxDepth: 3 }
    const mockResult = { nodes: [], edges: [], meta: { taskView: 'explore' } }
    __setMockHandler('get_topology_snapshot_v3', (_cmd, args) => {
      expect(args).toEqual({ query })
      return mockResult
    })

    const result = await getTopologySnapshotV3(query)
    expect(result).toEqual(mockResult)
  })

  it('getTopologyDrilldownV3 should invoke get_topology_drilldown_v3', async () => {
    const query: TopologyV3DrilldownQuery = { nodeId: 'node-1', taskView: 'troubleshoot', maxDepth: 4 }
    const mockResult = { centerNodeId: 'node-1', nodes: [], edges: [] }
    __setMockHandler('get_topology_drilldown_v3', (_cmd, args) => {
      expect(args).toEqual({ query })
      return mockResult
    })

    const result = await getTopologyDrilldownV3(query)
    expect(result).toEqual(mockResult)
  })

  it('getTopologyTaskViewV3 should invoke get_topology_task_view_v3', async () => {
    const query: TopologyV3TaskViewQuery = { taskView: 'impact', env: 'prod', maxDepth: 2 }
    const mockResult = {
      taskView: 'impact',
      snapshot: { nodes: [], edges: [], meta: { taskView: 'impact' } },
    }
    __setMockHandler('get_topology_task_view_v3', (_cmd, args) => {
      expect(args).toEqual({ query })
      return mockResult
    })

    const result = await getTopologyTaskViewV3(query)
    expect(result).toEqual(mockResult)
  })

  it('getTopologyPathsV3 should invoke get_topology_paths_v3', async () => {
    const query: TopologyV3PathsQuery = { sourceId: 'A', targetId: 'B', taskView: 'explore', maxDepth: 3, maxResults: 5 }
    const mockResult = { paths: [{ nodeIds: ['A', 'B'] }], truncated: false }
    __setMockHandler('get_topology_paths_v3', (_cmd, args) => {
      expect(args).toEqual({ query })
      return mockResult
    })

    const result = await getTopologyPathsV3(query)
    expect(result).toEqual(mockResult)
  })

  it('getTopologyImpactV3 should invoke get_topology_impact_v3', async () => {
    const query: TopologyV3ImpactQuery = { nodeId: 'C', taskView: 'impact', maxDepth: 3 }
    const mockResult = { affectedNodes: [], totalCount: 0, maxDepth: 0 }
    __setMockHandler('get_topology_impact_v3', (_cmd, args) => {
      expect(args).toEqual({ query })
      return mockResult
    })

    const result = await getTopologyImpactV3(query)
    expect(result).toEqual(mockResult)
  })

  it('getTopologyEvidenceV3 should invoke get_topology_evidence_v3', async () => {
    const query: TopologyV3EvidenceQuery = { nodeId: 'node-1', taskView: 'troubleshoot', maxItems: 20 }
    const mockResult = { items: [], total: 0 }
    __setMockHandler('get_topology_evidence_v3', (_cmd, args) => {
      expect(args).toEqual({ query })
      return mockResult
    })

    const result = await getTopologyEvidenceV3(query)
    expect(result).toEqual(mockResult)
  })

  it('getTopologyTroubleshootReportV3 should invoke get_topology_troubleshoot_report_v3', async () => {
    const query: TopologyV3TroubleshootReportQuery = {
      nodeId: 'node-1',
      taskView: 'troubleshoot',
      evidenceLimit: 10,
    }
    const mockResult = {
      node: { id: 'node-1', name: '订单服务', env: 'prod' },
      summary: {
        inboundEdgeCount: 2,
        outboundEdgeCount: 1,
        deploymentCount: 1,
        recentAuditCount: 1,
        statusSeverity: 'warning',
      },
      upstream: [],
      downstream: [],
      evidence: { items: [], total: 0 },
      insights: [],
    }
    __setMockHandler('get_topology_troubleshoot_report_v3', (_cmd, args) => {
      expect(args).toEqual({ query })
      return mockResult
    })

    const result = await getTopologyTroubleshootReportV3(query)
    expect(result).toEqual(mockResult)
  })
})
