import { createRouter, createWebHistory } from "vue-router";
import DefaultLayout from "@/layouts/DefaultLayout.vue";

const routes = [
  {
    path: "/",
    component: DefaultLayout,
    children: [
      { path: "", name: "Dashboard", component: () => import("@/views/DashboardView.vue"), meta: { title: "仪表盘" } },
      { path: "hosts", name: "Hosts", component: () => import("@/views/HostsView.vue"), meta: { title: "服务器" } },
      { path: "ip-addresses", name: "IpAddresses", component: () => import("@/views/IpAddressesView.vue"), meta: { title: "IP地址" } },
      { path: "applications", name: "Applications", component: () => import("@/views/ApplicationsView.vue"), meta: { title: "应用服务" } },
      { path: "middlewares", name: "Middlewares", component: () => import("@/views/MiddlewaresView.vue"), meta: { title: "中间件" } },
      { path: "nginx-configs", name: "NginxConfigs", component: () => import("@/views/NginxConfigsView.vue"), meta: { title: "负载均衡" } },
      { path: "topology", name: "Topology", component: () => import("@/views/TopologyView.vue"), meta: { title: "拓扑图" } },
      { path: "settings", name: "Settings", component: () => import("@/views/SettingsView.vue"), meta: { title: "系统设置" } },
    ],
  },
];

const router = createRouter({
  history: createWebHistory(),
  routes,
});

export default router;
