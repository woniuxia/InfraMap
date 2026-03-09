import type { Component } from "vue";

export interface DashboardKpiCard {
  key: string;
  title: string;
  value: number;
  subtitle: string;
  routeName: string;
  icon: Component;
  tone: "primary" | "success" | "warning" | "danger" | "info";
}

export interface DashboardQuickAction {
  key: string;
  title: string;
  desc: string;
  routeName?: string;
  icon: Component;
  priority: "primary" | "secondary";
  badge?: string;
}
