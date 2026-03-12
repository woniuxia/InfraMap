import { beforeEach, describe, expect, it, vi } from "vitest";
import { __clearMockHandlers, __setMockHandler } from "@/__mocks__/tauri";
import { deleteContact, getContact, listContacts, saveContact } from "@/api/contacts";

vi.mock("element-plus", () => ({
  ElMessage: {
    error: vi.fn(),
    success: vi.fn(),
    info: vi.fn(),
    warning: vi.fn(),
  },
}));

describe("contacts API", () => {
  beforeEach(() => {
    __clearMockHandlers();
    vi.clearAllMocks();
  });

  it("listContacts should invoke list_contacts with params", async () => {
    __setMockHandler("list_contacts", (_cmd, args) => {
      expect(args).toEqual({
        params: {
          page: 1,
          page_size: 20,
          search: "alice",
        },
      });
      return { data: [], total: 0, page: 1, page_size: 20 };
    });

    const result = await listContacts({
      page: 1,
      page_size: 20,
      search: "alice",
    });
    expect(result.total).toBe(0);
  });

  it("getContact should invoke get_contact with id", async () => {
    __setMockHandler("get_contact", (_cmd, args) => {
      expect(args).toEqual({ id: "contact-1" });
      return {
        id: "contact-1",
        name: "Alice",
        phone: "13800000000",
        email: "alice@example.com",
        remark: "owner",
        created_at: "",
        updated_at: "",
      };
    });

    const result = await getContact("contact-1");
    expect(result.id).toBe("contact-1");
  });

  it("saveContact should invoke save_contact and return id", async () => {
    __setMockHandler("save_contact", (_cmd, args) => {
      expect(args).toEqual({
        data: {
          id: "",
          name: "Bob",
          phone: "13900000000",
          email: undefined,
          remark: "SRE",
        },
      });
      return "contact-2";
    });

    const id = await saveContact({
      id: "",
      name: "Bob",
      phone: "13900000000",
      email: undefined,
      remark: "SRE",
    });
    expect(id).toBe("contact-2");
  });

  it("deleteContact should invoke delete_contact with id", async () => {
    __setMockHandler("delete_contact", (_cmd, args) => {
      expect(args).toEqual({ id: "contact-3" });
      return undefined;
    });

    await deleteContact("contact-3");
  });
});
