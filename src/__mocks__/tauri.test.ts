import { beforeEach, describe, expect, it } from 'vitest'
import { __clearMockHandlers, __setMockHandler, invoke } from '@/__mocks__/tauri'

describe('tauri mock invoke arg guard', () => {
  beforeEach(() => {
    __clearMockHandlers()
  })

  it('allows camelCase top-level args', async () => {
    __setMockHandler('get_resource_deploy_context', (_cmd, args) => args)

    const result = await invoke<Record<string, unknown>>('get_resource_deploy_context', {
      resourceType: 'middleware',
      resourceId: 'mw-1',
    })

    expect(result).toEqual({
      resourceType: 'middleware',
      resourceId: 'mw-1',
    })
  })

  it('allows nested snake_case payload fields', async () => {
    __setMockHandler('list_deployments', (_cmd, args) => args)

    const result = await invoke<Record<string, unknown>>('list_deployments', {
      params: {
        page_size: 20,
        filters: {
          resource_id: 'mw-1',
          resource_type: 'middleware',
        },
      },
    })

    expect(result).toEqual({
      params: {
        page_size: 20,
        filters: {
          resource_id: 'mw-1',
          resource_type: 'middleware',
        },
      },
    })
  })

  it('rejects snake_case top-level args', async () => {
    __setMockHandler('get_resource_deploy_context', () => undefined)

    await expect(
      invoke('get_resource_deploy_context', {
        resource_type: 'middleware',
        resource_id: 'mw-1',
      })
    ).rejects.toThrow('snake_case top-level args')
    await expect(
      invoke('get_resource_deploy_context', {
        resource_type: 'middleware',
      })
    ).rejects.toThrow('resource_type')
  })

  it('allows calls without args', async () => {
    __setMockHandler('get_topology_graph', (_cmd, args) => args ?? null)

    const result = await invoke<null>('get_topology_graph')

    expect(result).toBeNull()
  })
})
