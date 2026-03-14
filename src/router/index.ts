import { createRouter, createWebHistory } from "vue-router";
import DefaultLayout from "@/layouts/DefaultLayout.vue";

const routes = [
  {
    path: "/",
    component: DefaultLayout,
    children: [
      {
        path: "",
        name: "Dashboard",
        component: () => import("@/views/DashboardView.vue"),
        meta: { title: "仪表盘" },
      },
      {
        path: "hosts",
        name: "Hosts",
        component: () => import("@/views/HostsView.vue"),
        meta: { title: "服务器" },
      },
      {
        path: "ip-addresses",
        name: "IpAddresses",
        component: () => import("@/views/IpAddressesView.vue"),
        meta: { title: "IP地址" },
      },
      {
        path: "systems",
        name: "Systems",
        component: () => import("@/views/SystemsView.vue"),
        meta: { title: "系统" },
      },
      {
        path: "services",
        name: "Services",
        component: () => import("@/views/ServicesView.vue"),
        meta: { title: "服务" },
      },
      {
        path: "middlewares",
        name: "Middlewares",
        component: () => import("@/views/MiddlewaresView.vue"),
        meta: { title: "中间件" },
      },
      {
        path: "nginx-configs",
        name: "NginxConfigs",
        component: () => import("@/views/NginxConfigsView.vue"),
        meta: { title: "网关" },
      },
      {
        path: "contacts",
        name: "Contacts",
        component: () => import("@/views/ContactsView.vue"),
        meta: { title: "联系人" },
      },
      {
        path: "import-workbench",
        name: "ImportWorkbench",
        component: () => import("@/views/ImportWorkbenchView.vue"),
        meta: { title: "批量录入" },
      },
      {
        path: "jobs",
        name: "Jobs",
        component: () => import("@/views/JobCenterView.vue"),
        meta: { title: "任务中心" },
      },
      {
        path: "integrity-center",
        name: "IntegrityCenter",
        component: () => import("@/views/IntegrityCenterView.vue"),
        meta: { title: "数据健康" },
      },
      {
        path: "topology",
        name: "Topology",
        component: () => import("@/views/TopologyView.vue"),
        meta: { title: "拓扑图" },
      },
      {
        path: "settings",
        name: "Settings",
        component: () => import("@/views/SettingsView.vue"),
        meta: { title: "系统设置" },
      },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
