import { describe, expect, it } from "vitest";
import { mount } from "@vue/test-utils";
import TopologyDetailPanel from "@/components/topology/TopologyDetailPanel.vue";
import type {
  TopologyImpactResponse,
  TopologyNode,
  TopologyPathsResponse,
  TopologyTroubleshootReport,
} from "@/types";

const selectedNode: TopologyNode = {
  id: "node-1",
  name: "订单服务",
  node_type: "application",
  env: "prod",
  group_kind: "application_service",
  importance: 1,
  status: "maintenance",
  host_name: "host-a",
  extra: {
    address: "10.0.0.11",
    type: "backend",
  },
};

const troubleshootReport: TopologyTroubleshootReport = {
  node: {
    id: "node-1",
    name: "订单服务",
    nodeType: "application",
    env: "prod",
    status: "maintenance",
  },
  summary: {
    inboundEdgeCount: 2,
    outboundEdgeCount: 3,
    deploymentCount: 1,
    recentAuditCount: 2,
    statusSeverity: "warning",
  },
  upstream: [{ id: "node-up-1", name: "网关服务", nodeType: "application", env: "prod" }],
  downstream: [{ id: "node-down-1", name: "库存服务", nodeType: "application", env: "prod" }],
  evidence: {
    total: 1,
    items: [
      {
        id: "audit-1",
        evidenceType: "audit",
        title: "审计事件 update",
        description: "2026-03-07T12:00:00Z",
        timestamp: "2026-03-07T12:00:00Z",
        source: "audit_logs",
      },
    ],
  },
  insights: [
    {
      kind: "status",
      title: "节点状态异常",
      description: "当前节点处于维护中",
      severity: "warning",
      nodeIds: ["node-1"],
    },
  ],
};

const pathResult: TopologyPathsResponse = {
  paths: [{ nodeIds: ["node-1", "node-up-1"] }],
  truncated: false,
};

const impactResult: TopologyImpactResponse = {
  affectedNodes: [{ id: "node-up-1", name: "网关服务", nodeType: "application", depth: 1 }],
  totalCount: 1,
  maxDepth: 1,
};

function mountPanel(activeTab: "overview" | "evidence" | "path" | "impact") {
  return mount(TopologyDetailPanel, {
    props: {
      visible: true,
      activeTab,
      selectedNode,
      troubleshootReport,
      troubleshootLoading: false,
      pathResult,
      impactResult,
      nodeNameMap: {
        "node-1": "订单服务",
        "node-up-1": "网关服务",
      },
    },
    global: {
      stubs: {
        ElButton: true,
        ElDescriptions: true,
        ElDescriptionsItem: true,
        ElTag: true,
        ElEmpty: true,
        ElStatistic: true,
        ElTabs: false,
        ElTabPane: false,
      },
    },
  });
}

describe("TopologyDetailPanel", () => {
  it("在概览标签中展示排障摘要与动作入口", async () => {
    const wrapper = mountPanel("overview");

    expect(wrapper.text()).toContain("排障工作台");
    expect(wrapper.text()).toContain("入边");
    expect(wrapper.text()).toContain("2");
    expect(wrapper.text()).toContain("出边");
    expect(wrapper.text()).toContain("3");
    expect(wrapper.text()).toContain("发起路径追踪");
    expect(wrapper.text()).toContain("分析影响面");
    expect(wrapper.text()).toContain("编辑资源");
  });

  it("在证据标签中展示证据列表", async () => {
    const wrapper = mountPanel("evidence");

    expect(wrapper.text()).toContain("证据");
    expect(wrapper.text()).toContain("审计事件 update");
    expect(wrapper.text()).toContain("audit_logs");
  });
});
