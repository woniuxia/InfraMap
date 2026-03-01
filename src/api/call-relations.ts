import { tauriInvoke } from "@/utils/invoke";
import type {
  CallRelation,
  PagedResult,
  QueryParams,
  ReplaceResourceCallRelationsParams,
  ReplaceResourceCallRelationsResult,
} from "@/types";

export function listCallRelations(params: QueryParams): Promise<PagedResult<CallRelation>> {
  return tauriInvoke<PagedResult<CallRelation>>("list_call_relations", { params });
}

export function replaceResourceCallRelations(
  params: ReplaceResourceCallRelationsParams
): Promise<ReplaceResourceCallRelationsResult> {
  return tauriInvoke<ReplaceResourceCallRelationsResult>("replace_resource_call_relations", {
    params,
  });
}
