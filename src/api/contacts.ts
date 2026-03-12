import { tauriInvoke } from "@/utils/invoke";
import type { Contact, PagedResult, QueryParams } from "@/types";

export function listContacts(params: QueryParams): Promise<PagedResult<Contact>> {
  return tauriInvoke<PagedResult<Contact>>("list_contacts", { params });
}

export function getContact(id: string): Promise<Contact> {
  return tauriInvoke<Contact>("get_contact", { id });
}

export function saveContact(data: Partial<Contact>): Promise<string> {
  return tauriInvoke<string>("save_contact", { data });
}

export function deleteContact(id: string): Promise<void> {
  return tauriInvoke<void>("delete_contact", { id });
}
