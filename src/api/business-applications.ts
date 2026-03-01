import { tauriInvoke } from "@/utils/invoke";
import type {
  Application,
  AttachServicesResult,
  BusinessApplication,
  BusinessApplicationServices,
  PagedResult,
  QueryParams,
} from "@/types";

export function listBusinessApplications(
  params: QueryParams
): Promise<PagedResult<BusinessApplication>> {
  return tauriInvoke<PagedResult<BusinessApplication>>("list_business_applications", { params });
}

export function getBusinessApplication(id: string): Promise<BusinessApplication> {
  return tauriInvoke<BusinessApplication>("get_business_application", { id });
}

export function saveBusinessApplication(data: Partial<BusinessApplication>): Promise<string> {
  return tauriInvoke<string>("save_business_application", { data });
}

export function softDeleteBusinessApplication(id: string): Promise<void> {
  return tauriInvoke<void>("soft_delete_business_application", { id });
}

export function listUnassignedApplicationServices(
  params: QueryParams
): Promise<PagedResult<Application>> {
  return tauriInvoke<PagedResult<Application>>("list_unassigned_application_services", { params });
}

export function attachServicesToBusinessApplication(
  businessApplicationId: string,
  applicationIds: string[]
): Promise<AttachServicesResult> {
  return tauriInvoke<AttachServicesResult>("attach_services_to_business_application", {
    business_application_id: businessApplicationId,
    application_ids: applicationIds,
  });
}

export function detachServiceFromBusinessApplication(
  businessApplicationId: string,
  applicationId: string
): Promise<void> {
  return tauriInvoke<void>("detach_service_from_business_application", {
    business_application_id: businessApplicationId,
    application_id: applicationId,
  });
}

export function listServicesByBusinessApplication(
  businessApplicationId: string
): Promise<BusinessApplicationServices> {
  return tauriInvoke<BusinessApplicationServices>("list_services_by_business_application", {
    business_application_id: businessApplicationId,
  });
}
