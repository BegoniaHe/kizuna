import { create } from "zustand";
import type { WindowMode } from "@/types";

interface UIState {
  windowMode: WindowMode;
  sidebarOpen: boolean;
  sidebarWidth: number;
  modelPanelWidth: number;
  settingsOpen: boolean;
  isAlwaysOnTop: boolean;
  
  // 图片查看器状态
  imageViewer: {
    isOpen: boolean;
    src: string;
    alt?: string;
  };

  setWindowMode: (mode: WindowMode) => void;
  toggleSidebar: () => void;
  setSidebarOpen: (open: boolean) => void;
  setSidebarWidth: (width: number) => void;
  setModelPanelWidth: (width: number) => void;
  toggleSettings: () => void;
  setSettingsOpen: (open: boolean) => void;
  setAlwaysOnTop: (value: boolean) => void;
  
  // 图片查看器操作
  openImageViewer: (src: string, alt?: string) => void;
  closeImageViewer: () => void;
}

export const useUIStore = create<UIState>((set) => ({
  windowMode: "normal",
  sidebarOpen: true,
  sidebarWidth: 280,
  modelPanelWidth: 320,
  settingsOpen: false,
  isAlwaysOnTop: false,
  
  imageViewer: {
    isOpen: false,
    src: "",
    alt: "",
  },

  setWindowMode: (mode) => {
    set({ windowMode: mode });
  },

  toggleSidebar: () => {
    set((state) => ({ sidebarOpen: !state.sidebarOpen }));
  },

  setSidebarOpen: (open) => {
    set({ sidebarOpen: open });
  },

  setSidebarWidth: (width) => {
    set({ sidebarWidth: width });
  },

  setModelPanelWidth: (width) => {
    set({ modelPanelWidth: width });
  },

  toggleSettings: () => {
    set((state) => ({ settingsOpen: !state.settingsOpen }));
  },

  setSettingsOpen: (open) => {
    set({ settingsOpen: open });
  },

  setAlwaysOnTop: (value) => {
    set({ isAlwaysOnTop: value });
  },

  openImageViewer: (src, alt) => {
    console.log("[UIStore] Opening image viewer:", src);
    set({ imageViewer: { isOpen: true, src, alt } });
  },

  closeImageViewer: () => {
    console.log("[UIStore] Closing image viewer");
    set((state) => ({ imageViewer: { ...state.imageViewer, isOpen: false } }));
  },
}));
