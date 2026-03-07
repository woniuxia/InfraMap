import { defineComponent } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ElMessage } from "element-plus";
import IntegrityCenterView from "@/views/IntegrityCenterView.vue";
import type { IntegrityRepairResult, IntegrityReport } from "@/types";

const { scanIntegrityMock, repairIntegrityFindingsMock, pushMock } = vi.hoisted(() => ({
  scanIntegrityMock: vi.fn(),
  repairIntegrityFindingsMock: vi.fn(),
  pushMock: vi.fn(),
}));

vi.mock("vue-router", () => ({
  useRouter: () => ({
    push: pushMock,
  }),
}));

vi.mock("@/api/integrity", () => ({
  scanIntegrity: scanIntegrityMock,
  repairIntegrityFindings: repairIntegrityFindingsMock,
}));

vi.mock("element-plus", () => ({
  ElMessage: {
    success: vi.fn(),
    warning: vi.fn(),
  },
}));

const ElButtonStub = defineComponent({
  name: "ElButton",
  props: {
    disabled: {
      type: Boolean,
      default: false,
    },
  },
  emits: ["click"],
  template: '<button type="button" :disabled="disabled" @click="$emit(\'click\')"><slot /></button>',
});

const PassThroughStub = defineComponent({
  template: '<div><slot name="header" /><slot /></div>',
});

const ElTagStub = defineComponent({
  name: "ElTag",
  template: "<span><slot /></span>",
});

function createReport(): IntegrityReport {
  return {
    job_id: "job-integrity-1",
    summary: {
      total: 2,
      critical: 1,
      warning: 1,
      info: 0,
      repairable: 1,
      generated_at: "2026-03-07T10:00:00.000Z",
    },
    findings: [
      {
        id: "orphan-deployment:dep-1",
        key: "orphan_deployment",
        category: "deployment",
        severity: "critical",
        title: "孤儿部署关系",
        description: "部署记录指向了不存在的应用。",
        resource_name: "应用 A",
        target_route: "Deployments",
        target_filters: { status: "orphan" },
        repair_supported: true,
      },
      {
        id: "missing-binding:host-1",
        key: "missing_binding",
        category: "host",
        severity: "warning",
        title: "缺失绑定关系",
        description: "主机与中间件绑定不完整。",
        resource_name: "主机 01",
        target_route: "Hosts",
        target_filters: { env: "prod" },
        repair_supported: false,
      },
    ],
  };
}

function createRepairResult(): IntegrityRepairResult {
  return {
    job_id: "job-integrity-2",
    backup_filename: "backup_pre_integrity_repair_20260307.db",
    repaired_count: 1,
    skipped_count: 0,
    report: {
      job_id: "job-integrity-2",
      summary: {
        total: 1,
        critical: 0,
        warning: 1,
        info: 0,
        repairable: 0,
        generated_at: "2026-03-07T10:05:00.000Z",
      },
      findings: [
        {
          id: "missing-binding:host-1",
          key: "missing_binding",
          category: "host",
          severity: "warning",
          title: "缺失绑定关系",
          description: "主机与中间件绑定不完整。",
          resource_name: "主机 01",
          target_route: "Hosts",
          target_filters: { env: "prod" },
          repair_supported: false,
        },
      ],
    },
  };
}

function mountView() {
  return mount(IntegrityCenterView, {
    global: {
      stubs: {
        ElCard: PassThroughStub,
        ElButton: ElButtonStub,
        ElTag: ElTagStub,
      },
    },
  });
}

describe("IntegrityCenterView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    pushMock.mockReset();
    scanIntegrityMock.mockResolvedValue(createReport());
    repairIntegrityFindingsMock.mockResolvedValue(createRepairResult());
  });

  it("calls scanIntegrity on mount and renders findings", async () => {
    const wrapper = mountView();
    await flushPromises();

    expect(scanIntegrityMock).toHaveBeenCalledTimes(1);
    expect(wrapper.get('[data-testid="findings-list"]').text()).toContain("孤儿部署关系");
    expect(wrapper.text()).toContain("当前可自动修复 1 项");
  });

  it("repairs selected findings", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="select-orphan-deployment:dep-1"]').setValue(true);
    await wrapper.get('[data-testid="repair-selected"]').trigger("click");
    await flushPromises();

    expect(repairIntegrityFindingsMock).toHaveBeenCalledWith({
      finding_ids: ["orphan-deployment:dep-1"],
    });
    expect(ElMessage.success).toHaveBeenCalledWith("已修复 1 项问题");
    expect(wrapper.text()).not.toContain("孤儿部署关系");
  });

  it("pushes to target route when opening finding target", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="open-orphan-deployment:dep-1"]').trigger("click");

    expect(pushMock).toHaveBeenCalledWith({
      name: "Deployments",
      query: { status: "orphan" },
    });
  });
});
