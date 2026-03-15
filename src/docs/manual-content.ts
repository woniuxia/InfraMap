export interface ManualSection {
  id: string;
  title: string;
  icon?: string;
  children?: ManualSection[];
  component?: string;
}

export const manualSections: ManualSection[] = [
  {
    id: "overview",
    title: "产品概述",
    icon: "InfoFilled",
    children: [
      {
        id: "overview-intro",
        title: "什么是 InfraMap",
        component: "OverviewIntro",
      },
      {
        id: "overview-features",
        title: "核心功能",
        component: "OverviewFeatures",
      },
    ],
  },
  {
    id: "topology",
    title: "拓扑图使用指南",
    icon: "Share",
    children: [
      {
        id: "topology-basics",
        title: "基础操作",
        component: "TopologyBasics",
      },
      {
        id: "topology-task-views",
        title: "任务视图",
        component: "TopologyTaskViews",
      },
      {
        id: "topology-layouts",
        title: "布局模式",
        component: "TopologyLayouts",
      },
      {
        id: "topology-analysis",
        title: "分析功能",
        component: "TopologyAnalysis",
      },
      {
        id: "topology-filters",
        title: "筛选与搜索",
        component: "TopologyFilters",
      },
    ],
  },
  {
    id: "resources",
    title: "资源管理",
    icon: "FolderOpened",
    children: [
      {
        id: "resources-hosts",
        title: "服务器管理",
        component: "ResourcesHosts",
      },
      {
        id: "resources-systems",
        title: "系统管理",
        component: "ResourcesSystems",
      },
      {
        id: "resources-services",
        title: "服务管理",
        component: "ResourcesServices",
      },
      {
        id: "resources-middlewares",
        title: "中间件管理",
        component: "ResourcesMiddlewares",
      },
      {
        id: "resources-nginx",
        title: "网关管理",
        component: "ResourcesNginx",
      },
      {
        id: "resources-contacts",
        title: "联系人管理",
        component: "ResourcesContacts",
      },
      {
        id: "resources-ip",
        title: "IP地址管理",
        component: "ResourcesIP",
      },
    ],
  },
  {
    id: "tools",
    title: "工具箱",
    icon: "Tools",
    children: [
      {
        id: "tools-import",
        title: "批量录入",
        component: "ToolsImport",
      },
      {
        id: "tools-jobs",
        title: "任务中心",
        component: "ToolsJobs",
      },
      {
        id: "tools-integrity",
        title: "数据健康",
        component: "ToolsIntegrity",
      },
    ],
  },
  {
    id: "settings",
    title: "系统设置",
    icon: "Setting",
    children: [
      {
        id: "settings-storage",
        title: "存储路径",
        component: "SettingsStorage",
      },
      {
        id: "settings-backup",
        title: "备份策略",
        component: "SettingsBackup",
      },
      {
        id: "settings-snapshot",
        title: "快照导入导出",
        component: "SettingsSnapshot",
      },
    ],
  },
  {
    id: "faq",
    title: "常见问题",
    icon: "QuestionFilled",
    component: "FAQ",
  },
];

// 扁平化所有章节,便于查找
export function flattenSections(sections: ManualSection[]): ManualSection[] {
  const result: ManualSection[] = [];
  sections.forEach((section) => {
    result.push(section);
    if (section.children) {
      result.push(...flattenSections(section.children));
    }
  });
  return result;
}

// 根据 ID 查找章节
export function findSectionById(id: string): ManualSection | undefined {
  return flattenSections(manualSections).find((s) => s.id === id);
}
