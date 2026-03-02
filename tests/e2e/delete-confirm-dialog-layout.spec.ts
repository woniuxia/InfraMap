import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const now = "2026-03-01T00:00:00Z";
    const ipRows = [
      {
        id: "ip-1",
        ip_address: "10.0.0.8",
        env: "prod",
        is_vip: false,
        real_ips: null,
        tags: null,
        description: "e2e-ip",
        is_deleted: 0,
        created_at: now,
        updated_at: now,
      },
    ];

    const paged = <T>(data: T[], pageSize = 20) => ({
      data,
      total: data.length,
      page: 1,
      page_size: pageSize,
    });

    let callbackId = 0;

    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string) => {
        if (cmd === "list_ip_addresses") {
          return paged(ipRows, 20);
        }
        if (cmd === "list_taxonomy_terms") {
          return [];
        }
        return null;
      },
      transformCallback: () => {
        callbackId += 1;
        return callbackId;
      },
      unregisterCallback: () => undefined,
    };
  });
});

test("delete confirmation dialog should render with stable layout", async ({ page }) => {
  await page.goto("/ip-addresses");

  await page.getByRole("button", { name: "删除" }).first().click();

  const dialog = page.locator(".el-message-box");
  const header = dialog.locator(".el-message-box__header");
  const content = dialog.locator(".el-message-box__content");
  const buttons = dialog.locator(".el-message-box__btns");
  const cancelButton = dialog.getByRole("button", { name: "取消" });
  const confirmButton = dialog.getByRole("button", { name: "确定" });

  await expect(dialog).toBeVisible();
  await expect(cancelButton).toBeVisible();
  await expect(confirmButton).toBeVisible();

  const dialogBox = await dialog.boundingBox();
  const headerBox = await header.boundingBox();
  const contentBox = await content.boundingBox();
  const buttonsBox = await buttons.boundingBox();
  const cancelBox = await cancelButton.boundingBox();
  const confirmBox = await confirmButton.boundingBox();

  expect(dialogBox).not.toBeNull();
  expect(headerBox).not.toBeNull();
  expect(contentBox).not.toBeNull();
  expect(buttonsBox).not.toBeNull();
  expect(cancelBox).not.toBeNull();
  expect(confirmBox).not.toBeNull();

  if (
    dialogBox &&
    headerBox &&
    contentBox &&
    buttonsBox &&
    cancelBox &&
    confirmBox
  ) {
    expect(dialogBox.width).toBeGreaterThan(320);
    expect(headerBox.y + headerBox.height).toBeLessThanOrEqual(contentBox.y + 2);
    expect(contentBox.y + contentBox.height).toBeLessThanOrEqual(buttonsBox.y + 2);
    expect(cancelBox.x + cancelBox.width).toBeLessThanOrEqual(confirmBox.x + 2);
  }
});
