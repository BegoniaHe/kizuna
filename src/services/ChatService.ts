import { commandBus, createSafeSubscriber } from "./ipc";
import type { Message, MessageChunk, Emotion, ProviderConfig } from "@/types";
import { logger } from "@/utils/logger";

export interface IChatService {
  sendMessage(sessionId: string, content: string, providerConfig?: ProviderConfig): Promise<string>;
  regenerate(sessionId: string, userContent: string, providerConfig?: ProviderConfig): Promise<string>;
  stopGeneration(sessionId: string): Promise<void>;
  getMessages(sessionId: string, page?: number, limit?: number): Promise<Message[]>;
  onMessageChunk(callback: (chunk: MessageChunk) => void): () => void;
  onMessageComplete(
    callback: (data: { sessionId: string; messageId: string; emotion?: Emotion }) => void,
  ): () => void;
  onMessageError(callback: (data: { sessionId: string; error: string }) => void): () => void;
}

class ChatServiceImpl implements IChatService {
  async sendMessage(sessionId: string, content: string, providerConfig?: ProviderConfig): Promise<string> {
    logger.debug(`[ChatService] sendMessage called`, { sessionId, content, providerConfig: providerConfig ? '(configured)' : '(none)' });
    try {
      const result = await commandBus.dispatch<
        { request: { sessionId: string; content: string; providerConfig?: ProviderConfig } },
        { messageId: string }
      >("chat:send_message", { request: { sessionId, content, providerConfig } });
      logger.debug(`[ChatService] sendMessage success`, result);
      return result.messageId;
    } catch (error) {
      console.error(`[ChatService] sendMessage failed`, error);
      throw error;
    }
  }

  async regenerate(sessionId: string, userContent: string, providerConfig?: ProviderConfig): Promise<string> {
    logger.debug(`[ChatService] regenerate called`, { sessionId, userContent, providerConfig: providerConfig ? '(configured)' : '(none)' });
    try {
      const result = await commandBus.dispatch<
        { request: { sessionId: string; userContent: string; providerConfig?: ProviderConfig } },
        { messageId: string }
      >("chat:regenerate", { request: { sessionId, userContent, providerConfig } });
      logger.debug(`[ChatService] regenerate success`, result);
      return result.messageId;
    } catch (error) {
      console.error(`[ChatService] regenerate failed`, error);
      throw error;
    }
  }

  async stopGeneration(sessionId: string): Promise<void> {
    await commandBus.dispatch("chat:stop_generation", { request: { sessionId } });
  }

  async getMessages(sessionId: string, page = 1, limit = 50): Promise<Message[]> {
    const messages = await commandBus.dispatch<
      { request: { sessionId: string; page: number; limit: number } },
      any[]
    >("chat:get_messages", { request: { sessionId, page, limit } });

    // 兼容处理：如果后端返回的是 snake_case，转换为 camelCase
    return messages.map((msg) => ({
      ...msg,
      id: msg.id,
      sessionId: msg.sessionId || msg.session_id,
      role: msg.role,
      content: msg.content,
      tokens: msg.tokens,
      emotion: msg.emotion,
      createdAt: msg.createdAt || msg.created_at || new Date().toISOString(),
    })) as Message[];
  }

  onMessageChunk(callback: (chunk: MessageChunk) => void): () => void {
    logger.debug(`[ChatService] Subscribing to llm:chunk`);
    return createSafeSubscriber<MessageChunk>("llm:chunk", (chunk) => {
      logger.debug(`[ChatService] Received chunk:`, chunk);
      callback(chunk);
    });
  }

  onMessageComplete(
    callback: (data: { sessionId: string; messageId: string; emotion?: Emotion }) => void,
  ): () => void {
    logger.debug(`[ChatService] Subscribing to llm:complete`);
    return createSafeSubscriber<{ sessionId: string; messageId: string; emotion?: Emotion }>(
      "llm:complete",
      (data) => {
        logger.debug(`[ChatService] Received complete:`, data);
        callback(data);
      },
    );
  }

  onMessageError(callback: (data: { sessionId: string; error: string }) => void): () => void {
    logger.debug(`[ChatService] Subscribing to llm:error`);
    return createSafeSubscriber<{ sessionId: string; error: string }>("llm:error", (data) => {
      logger.debug(`[ChatService] Received error:`, data);
      callback(data);
    });
  }
}

export const chatService: IChatService = new ChatServiceImpl();
