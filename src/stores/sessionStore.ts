import { create } from "zustand";
import type { Session } from "@/types";
import { sessionService } from "@/services/SessionService";

interface SessionState {
  sessions: Session[];
  isLoading: boolean;
  error: string | null;

  loadSessions: () => Promise<void>;
  createSession: (presetId?: string) => Promise<Session>;
  deleteSession: (id: string) => Promise<void>;
  renameSession: (id: string, title: string) => Promise<void>;
}

export const useSessionStore = create<SessionState>((set) => ({
  sessions: [],
  isLoading: false,
  error: null,

  loadSessions: async () => {
    set({ isLoading: true, error: null });
    try {
      const sessions = await sessionService.listSessions();
      set({ sessions, isLoading: false });
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to load sessions",
        isLoading: false,
      });
    }
  },

  createSession: async (presetId?: string) => {
    try {
      const session = await sessionService.createSession(presetId);
      set((state) => ({
        sessions: [session, ...state.sessions],
      }));
      return session;
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to create session",
      });
      throw error;
    }
  },

  deleteSession: async (id: string) => {
    try {
      await sessionService.deleteSession(id);
      set((state) => ({
        sessions: state.sessions.filter((s) => s.id !== id),
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to delete session",
      });
    }
  },

  renameSession: async (id: string, title: string) => {
    try {
      await sessionService.renameSession(id, title);
      set((state) => ({
        sessions: state.sessions.map((s) =>
          s.id === id ? { ...s, title } : s,
        ),
      }));
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to rename session",
      });
    }
  },
}));
