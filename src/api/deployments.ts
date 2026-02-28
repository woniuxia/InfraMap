import { tauriInvoke } from '@/utils/invoke'
import type {
  Deployment,
  DeploymentResourceType,
  QueryParams,
  PagedResult,
  ResourceDeployContext,
} from '@/types'

export function listDeployments(params: QueryParams): Promise<PagedResult<Deployment>> {
  return tauriInvoke<PagedResult<Deployment>>('list_deployments', { params })
}

export function saveDeployment(data: Partial<Deployment>): Promise<void> {
  return tauriInvoke<void>('save_deployment', { data })
}

export function softDeleteDeployment(id: string): Promise<void> {
  return tauriInvoke<void>('soft_delete_deployment', { id })
}

export function getResourceDeployContext(
  resourceType: DeploymentResourceType,
  resourceId: string
): Promise<ResourceDeployContext> {
  return tauriInvoke<ResourceDeployContext>('get_resource_deploy_context', {
    resource_type: resourceType,
    resource_id: resourceId,
  })
}
