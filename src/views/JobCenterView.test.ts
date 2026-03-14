import { defineComponent } from "vue";
import { flushPromises, mount } from "@vue/test-utils";
import { beforeEach, describe, expect, it, vi } from "vitest";
import JobCenterView from "@/views/JobCenterView.vue";
import type { PagedResult, SystemJobDetail, SystemJobSummary } from "@/types";

const { listSystemJobsMock, getSystemJobDetailMock } = vi.hoisted(() => ({
  listSystemJobsMock: vi.fn(),
  getSystemJobDetailMock: vi.fn(),
}));

vi.mock("@/api/system-jobs", () => ({
  listSystemJobs: listSystemJobsMock,
  getSystemJobDetail: getSystemJobDetailMock,
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
  template:
    '<button type="button" :disabled="disabled" @click="$emit(\'click\')"><slot /></button>',
});

const PassThroughStub = defineComponent({
  template: '<div><slot name="header" /><slot /></div>',
});

const ElTagStub = defineComponent({
  name: "ElTag",
  template: "<span><slot /></span>",
});

function createJobsPage(): PagedResult<SystemJobSummary> {
  return {
    data: [
      {
        id: "job-1",
        job_type: "import_applications",
        title: "批量导入应用",
        status: "failed",
        summary: "本次导入共处理 2 行，1 行失败",
        progress_percent: 100,
        retryable: true,
        cancellable: false,
        created_at: "2026-03-07T10:00:00.000Z",
        updated_at: "2026-03-07T10:05:00.000Z",
        finished_at: "2026-03-07T10:05:00.000Z",
      },
      {
        id: "job-2",
        job_type: "integrity_scan",
        title: "完整性扫描",
        status: "running",
        summary: "正在扫描引用关系",
        progress_percent: 45,
        retryable: false,
        cancellable: true,
        created_at: "2026-03-07T11:00:00.000Z",
        updated_at: "2026-03-07T11:02:00.000Z",
      },
    ],
    total: 2,
    page: 1,
    page_size: 20,
  };
}

function createJobDetail(): SystemJobDetail {
  return {
    summary: createJobsPage().data[0],
    error_message: "导入失败：存在重复应用编码",
    payload: {
      file_name: "applications.xlsx",
      operator: "tester",
    },
    result: {
      processed: 2,
      failed: 1,
    },
    import_rows: [
      {
        row_no: 2,
        resource_type: "service",
        name: "应用 A",
        env: "prod",
        status: "failed",
        error_message: "编码重复",
      },
    ],
    import_issues: [
      {
        row_no: 2,
        field_key: "code",
        issue_type: "duplicate",
        code: "APP_CODE_DUPLICATED",
        message: "重复应用编码",
      },
    ],
  };
}

function mountView() {
  return mount(JobCenterView, {
    global: {
      stubs: {
        ElCard: PassThroughStub,
        ElButton: ElButtonStub,
        ElTag: ElTagStub,
      },
    },
  });
}

describe("JobCenterView", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    listSystemJobsMock.mockResolvedValue(createJobsPage());
    getSystemJobDetailMock.mockResolvedValue(createJobDetail());
  });

  it("calls listSystemJobs on mount and renders rows", async () => {
    const wrapper = mountView();
    await flushPromises();

    expect(listSystemJobsMock).toHaveBeenCalledWith({ page: 1, page_size: 20 });
    expect(wrapper.get('[data-testid="jobs-table"]').text()).toContain("批量导入应用");
    expect(wrapper.text()).toContain("完整性扫描");
  });

  it("loads detail on click and renders detail sections", async () => {
    const wrapper = mountView();
    await flushPromises();

    await wrapper.get('[data-testid="detail-job-1"]').trigger("click");
    await flushPromises();

    expect(getSystemJobDetailMock).toHaveBeenCalledWith("job-1");
    expect(wrapper.get('[data-testid="job-detail"]').text()).toContain(
      "导入失败：存在重复应用编码",
    );
    expect(wrapper.get('[data-testid="payload-json"]').text()).toContain(
      '"file_name": "applications.xlsx"',
    );
    expect(wrapper.get('[data-testid="import-rows"]').text()).toContain("应用 A");
    expect(wrapper.get('[data-testid="import-issues"]').text()).toContain("重复应用编码");
  });
});
