import { describe, expect, it } from "vitest";
import {
  SYSTEM_STATUS_OPTIONS,
  DEPLOY_MODE_OPTIONS,
  ENV_OPTIONS,
  STATUS_OPTIONS,
  TOPOLOGY_ENV_OPTIONS,
} from "@/constants/options";

describe("shared options", () => {
  it("exports environment options in stable order", () => {
    expect(ENV_OPTIONS).toEqual([
      { label: "生产", value: "prod" },
      { label: "开发", value: "dev" },
      { label: "测试", value: "test" },
    ]);
  });

  it("exports status options for running resources", () => {
    expect(STATUS_OPTIONS).toEqual([
      { label: "运行中", value: "running" },
      { label: "已停止", value: "stopped" },
      { label: "维护中", value: "maintenance" },
    ]);
  });

  it("exports deployment mode options", () => {
    expect(DEPLOY_MODE_OPTIONS).toEqual([
      { label: "物理机", value: "physical" },
      { label: "虚拟机", value: "vm" },
      { label: "Docker", value: "docker" },
      { label: "Kubernetes", value: "k8s" },
      { label: "Serverless", value: "serverless" },
      { label: "其他", value: "other" },
    ]);
  });

  it("exports topology environment options in topology view order", () => {
    expect(TOPOLOGY_ENV_OPTIONS).toEqual([
      { label: "生产", value: "prod" },
      { label: "测试", value: "test" },
      { label: "开发", value: "dev" },
    ]);
  });

  it("exports system status options", () => {
    expect(SYSTEM_STATUS_OPTIONS).toEqual([
      { label: "激活", value: "active" },
      { label: "停用", value: "inactive" },
    ]);
  });
});
