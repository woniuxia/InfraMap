import { defineStore } from "pinia";
import { ref, watch } from "vue";
import type { TopologyEnv } from "@/types";

const STORAGE_KEY = "inframap.global.env";
const DEFAULT_ENV: TopologyEnv = "prod";

function loadInitialEnv(): TopologyEnv {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored === "prod" || stored === "test" || stored === "dev") {
      return stored;
    }
  } catch {
    // localStorage not available
  }
  return DEFAULT_ENV;
}

export const useEnvStore = defineStore("env", () => {
  const currentEnv = ref<TopologyEnv>(loadInitialEnv());

  watch(currentEnv, (newEnv) => {
    try {
      localStorage.setItem(STORAGE_KEY, newEnv);
    } catch {
      // localStorage not available
    }
  });

  function setEnv(env: TopologyEnv) {
    if (currentEnv.value === env) return;
    currentEnv.value = env;
  }

  return { currentEnv, setEnv };
});
