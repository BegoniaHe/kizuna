import { commandBus, createSafeSubscriber } from "./ipc";
import type { WindowMode } from "@/types";

export interface WindowInfo {
  label: string;
  mode: string;
  isVisible: boolean;
  isFocused: boolean;
  width: number;
  height: number;
}

export interface CreateWindowOptions {
  label: string;
  title?: string;
  width?: number;
  height?: number;
  mode?: "normal" | "pet" | "compact";
}

export interface IWindowService {
  togglePetMode(): Promise<{ isPetMode: boolean }>;
  setAlwaysOnTop(value: boolean): Promise<void>;
  startDragging(): Promise<void>;
  createWindow(options: CreateWindowOptions): Promise<WindowInfo>;
  listWindows(): Promise<WindowInfo[]>;
  closeWindow(label: string): Promise<void>;
  onModeChanged(callback: (data: { mode: WindowMode }) => void): () => void;
}

class WindowServiceImpl implements IWindowService {
  async togglePetMode(): Promise<{ isPetMode: boolean }> {
    return await commandBus.dispatch<void, { isPetMode: boolean }>("window:toggle_pet_mode");
  }

  async setAlwaysOnTop(value: boolean): Promise<void> {
    await commandBus.dispatch("window:set_always_on_top", { request: { value } });
  }

  async startDragging(): Promise<void> {
    await commandBus.dispatch("window:start_dragging");
  }

  async createWindow(options: CreateWindowOptions): Promise<WindowInfo> {
    return await commandBus.dispatch<CreateWindowOptions, WindowInfo>("window:create", options);
  }

  async listWindows(): Promise<WindowInfo[]> {
    return await commandBus.dispatch<void, WindowInfo[]>("window:list");
  }

  async closeWindow(label: string): Promise<void> {
    await commandBus.dispatch("window:close", label);
  }

  onModeChanged(callback: (data: { mode: WindowMode }) => void): () => void {
    return createSafeSubscriber<{ mode: WindowMode }>("window:mode_changed", callback);
  }
}

export const windowService: IWindowService = new WindowServiceImpl();
