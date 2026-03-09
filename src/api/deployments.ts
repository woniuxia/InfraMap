import { tauriInvoke } from "@/utils/invoke";
import type {
  Deployment,
  DeploymentResourceType,
  QueryParams,
  PagedResult,
  ResourceDeployContext,
} from "@/types";

export function listDeployments(params: QueryParams): Promise<PagedResult<Deployment>> {
  return tauriInvoke<PagedResult<Deployment>>("list_deployments", { params });
}

export function saveDeployment(data: Partial<Deployment>): Promise<string> {
  return tauriInvoke<string>("save_deployment", { data });
}

export function deleteDeployment(id: string): Promise<void> {
  return tauriInvoke<void>("delete_deployment", { id });
}

export interface ResourceDeployContextOverrides {
  address?: string;
  resourceEnv?: "prod" | "dev" | "test";
}

export function getResourceDeployContext(
  resourceType: DeploymentResourceType,
  resourceId: string,
  overrides?: ResourceDeployContextOverrides,
): Promise<ResourceDeployContext> {
  const payload: Record<string, unknown> = {
    resourceType,
    resourceId,
  };
  if (overrides?.address !== undefined) {
    payload.addressOverride = overrides.address;
  }
  if (overrides?.resourceEnv !== undefined) {
    payload.resourceEnvOverride = overrides.resourceEnv;
  }
  return tauriInvoke<ResourceDeployContext>("get_resource_deploy_context", {
    ...payload,
  });
}
