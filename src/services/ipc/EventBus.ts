import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type EventCallback<T> = (payload: T) => void;

class EventBus {
  private listeners: Map<string, UnlistenFn[]> = new Map();

  async subscribe<T>(event: string, callback: EventCallback<T>): Promise<() => void> {
    const unlisten = await listen<T>(event, (e) => {
      callback(e.payload);
    });

    const existing = this.listeners.get(event) ?? [];
    existing.push(unlisten);
    this.listeners.set(event, existing);

    return () => {
      unlisten();
      const updated = this.listeners.get(event)?.filter((fn) => fn !== unlisten) ?? [];
      this.listeners.set(event, updated);
    };
  }

  unsubscribeAll(event?: string): void {
    if (event) {
      const listeners = this.listeners.get(event) ?? [];
      listeners.forEach((unlisten) => unlisten());
      this.listeners.delete(event);
    } else {
      this.listeners.forEach((listeners) => {
        listeners.forEach((unlisten) => unlisten());
      });
      this.listeners.clear();
    }
  }
}

export const eventBus = new EventBus();
