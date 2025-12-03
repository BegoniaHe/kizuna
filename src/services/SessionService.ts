import { commandBus } from "./ipc";
import type { Session } from "@/types";

export interface ISessionService {
  createSession(presetId?: string): Promise<Session>;
  listSessions(page?: number, limit?: number): Promise<Session[]>;
  getSession(id: string): Promise<Session>;
  deleteSession(id: string): Promise<void>;
  renameSession(id: string, title: string): Promise<void>;
}

class SessionServiceImpl implements ISessionService {
  async createSession(presetId?: string): Promise<Session> {
    return await commandBus.dispatch<{ request: { presetId?: string } }, Session>(
      "session:create",
      { request: { presetId } },
    );
  }

  async listSessions(page = 1, limit = 20): Promise<Session[]> {
    const result = await commandBus.dispatch<
      { request: { page: number; limit: number } },
      { sessions: Session[]; total: number }
    >("session:list", { request: { page, limit } });
    return result.sessions;
  }

  async getSession(id: string): Promise<Session> {
    return await commandBus.dispatch<{ request: { id: string } }, Session>(
      "session:get",
      { request: { id } },
    );
  }

  async deleteSession(id: string): Promise<void> {
    await commandBus.dispatch("session:delete", { request: { id } });
  }

  async renameSession(id: string, title: string): Promise<void> {
    await commandBus.dispatch("session:rename", { request: { id, title } });
  }
}

export const sessionService: ISessionService = new SessionServiceImpl();
