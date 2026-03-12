import { ref, watch, computed } from "vue";

type ViewMode = "card" | "list";

interface UseViewToggleOptions {
  storageKey: string;
  defaultView?: ViewMode;
}

export function useViewToggle(options: UseViewToggleOptions) {
  const { storageKey, defaultView = "card" } = options;

  const storedView = localStorage.getItem(storageKey) as ViewMode | null;
  const currentView = ref<ViewMode>(storedView || defaultView);

  watch(currentView, (newView) => {
    localStorage.setItem(storageKey, newView);
  });

  function setView(view: ViewMode) {
    currentView.value = view;
  }

  function toggleView() {
    currentView.value = currentView.value === "card" ? "list" : "card";
  }

  return {
    currentView,
    setView,
    toggleView,
    isCardView: computed(() => currentView.value === "card"),
    isListView: computed(() => currentView.value === "list"),
  };
}
