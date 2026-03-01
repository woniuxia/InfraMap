import { expect, test } from "@playwright/test";

test.beforeEach(async ({ page }) => {
  await page.addInitScript(() => {
    const now = "2026-03-01T00:00:00Z";
    const appRows = [
      {
        id: "app-frontend",
        name: "web-portal",
        type: "frontend",
        env: "prod",
        status: "running",
        is_deleted: 0,
        created_at: now,
        updated_at: now,
      },
      {
        id: "app-backend",
        name: "order-api",
        type: "backend",
        env: "prod",
        status: "running",
        is_deleted: 0,
        created_at: now,
        updated_at: now,
      },
    ];

    const middlewareRows = [
      {
        id: "mw-redis",
        name: "redis-main",
        category: "cache",
        type: "Redis",
        address: "redis://10.0.0.8",
        env: "prod",
        is_deleted: 0,
        created_at: now,
        updated_at: now,
      },
    ];

    const nginxRows = [
      {
        id: "ng-1",
        name: "traffic-lb",
        address: "10.0.0.9",
        env: "prod",
        status: "running",
        is_deleted: 0,
        created_at: now,
        updated_at: now,
      },
    ];

    const paged = <T>(data: T[], pageSize = 999) => ({
      data,
      total: data.length,
      page: 1,
      page_size: pageSize,
    });

    let callbackId = 0;

    (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {
      invoke: async (cmd: string) => {
        if (cmd === "list_applications") {
          return paged(appRows, 10);
        }
        if (cmd === "list_middlewares") {
          return paged(middlewareRows, 999);
        }
        if (cmd === "list_nginx_configs") {
          return paged(nginxRows, 999);
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

test("shows typed labels in call-relation target resources", async ({ page }) => {
  await page.goto("/applications");

  await page.getByRole("button", { name: "新增应用" }).click();
  await page.getByRole("button", { name: "添加关系" }).click();
  await page.getByPlaceholder("选择目标资源").first().click();

  await expect(page.getByText("web-portal（前端）")).toBeVisible();
  await expect(page.getByText("order-api（后端）")).toBeVisible();
  await expect(page.getByText("redis-main（Redis）")).toBeVisible();
  await expect(page.getByText("traffic-lb（负载均衡）")).toBeVisible();
});
