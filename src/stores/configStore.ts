import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import type { AppConfig, GeneralConfig } from "@/types";

interface ConfigState {
  config: AppConfig;
  isLoaded: boolean;

  setTheme: (theme: GeneralConfig["theme"]) => void;
  setLanguage: (language: string) => void;
  setAutoStart: (autoStart: boolean) => void;
  setLLMProvider: (providerId: string) => void;
  updateConfig: (partial: Partial<AppConfig>) => void;
  resetConfig: () => void;
}

const defaultConfig: AppConfig = {
  general: {
    language: "zh-CN",
    theme: "system",
    autoStart: false,
    minimizeToTray: true,
  },
  window: {
    defaultMode: "normal",
    petModeSize: { width: 300, height: 400 },
    petModePosition: "remember",
  },
  shortcuts: {
    toggleWindow: "CommandOrControl+Shift+K",
    togglePetMode: "CommandOrControl+Shift+P",
    newChat: "CommandOrControl+Shift+N",
  },
  llm: {
    defaultProvider: "",
    streamResponse: true,
    contextLength: 10,
    providers: {},
  },
  model: {
    defaultType: "live2d",
    autoLoadLast: true,
    physicsEnabled: true,
  },
};

export const useConfigStore = create<ConfigState>()(
  persist(
    (set) => ({
      config: defaultConfig,
      isLoaded: false,

      setTheme: (theme) => {
        set((state) => ({
          config: {
            ...state.config,
            general: { ...state.config.general, theme },
          },
        }));
        applyTheme(theme);
      },

      setLanguage: (language) => {
        set((state) => ({
          config: {
            ...state.config,
            general: { ...state.config.general, language },
          },
        }));
      },

      setAutoStart: (autoStart) => {
        set((state) => ({
          config: {
            ...state.config,
            general: { ...state.config.general, autoStart },
          },
        }));
      },

      setLLMProvider: (providerId) => {
        set((state) => ({
          config: {
            ...state.config,
            llm: { ...state.config.llm, defaultProvider: providerId },
          },
        }));
      },

      updateConfig: (partial) => {
        set((state) => ({
          config: { ...state.config, ...partial },
        }));
      },

      resetConfig: () => {
        set({ config: defaultConfig });
      },
    }),
    {
      name: "kizuna-config",
      storage: createJSONStorage(() => localStorage),
      onRehydrateStorage: () => (state) => {
        if (state) {
          state.isLoaded = true;
          applyTheme(state.config.general.theme);
        }
      },
    },
  ),
);

function applyTheme(theme: GeneralConfig["theme"]) {
  const root = document.documentElement;
  if (theme === "system") {
    const isDark = window.matchMedia("(prefers-color-scheme: dark)").matches;
    root.classList.toggle("dark", isDark);
  } else {
    root.classList.toggle("dark", theme === "dark");
  }
}
