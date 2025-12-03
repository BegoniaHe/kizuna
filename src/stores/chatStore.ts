import { logger } from "@/utils/logger";
import { create } from "zustand";
import type { Message, Session, Emotion, ProviderConfig } from "@/types";
import { chatService } from "@/services/ChatService";
import { useConfigStore } from "./configStore";

// Token 使用信息
export interface TokenUsage {
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
}

interface ChatState {
  currentSession: Session | null;
  messages: Message[];
  isGenerating: boolean;
  streamContent: string;
  error: string | null;
  lastTokenUsage: TokenUsage | null;

  setCurrentSession: (session: Session | null) => Promise<void>;
  sendMessage: (content: string) => Promise<void>;
  stopGeneration: () => void;
  appendStreamContent: (content: string) => void;
  finalizeMessage: (messageId: string, emotion?: Emotion, tokens?: number) => void;
  clearMessages: () => void;
  setError: (error: string | null) => void;
  // 新增操作
  editMessage: (messageId: string, newContent: string) => void;
  deleteMessage: (messageId: string) => void;
  deleteMessagesFrom: (messageId: string) => void;
  regenerateFrom: (messageId: string) => Promise<void>;
  setTokenUsage: (usage: TokenUsage | null) => void;
}

/**
 * 从 configStore 获取当前的 LLM 配置
 */
function getLLMConfig(): ProviderConfig | undefined {
  const configState = useConfigStore.getState();
  const { llm } = configState.config;
  const providerId = llm.defaultProvider || "default";
  const provider = llm.providers[providerId];
  
  logger.debug(`[getLLMConfig] providerId: ${providerId}`);
  logger.debug(`[getLLMConfig] provider:`, provider);
  logger.debug(`[getLLMConfig] llm.providers:`, llm.providers);
  
  if (provider && provider.apiKey && provider.baseUrl) {
    logger.debug(`[getLLMConfig] Returning config:`, { ...provider, apiKey: '***' });
    return provider;
  }
  
  logger.debug(`[getLLMConfig] No valid config found`);
  return undefined;
}

export const useChatStore = create<ChatState>((set, get) => ({
  currentSession: null,
  messages: [],
  isGenerating: false,
  streamContent: "",
  error: null,
  lastTokenUsage: null,

  setCurrentSession: async (session) => {
    set({ currentSession: session, messages: [], streamContent: "", error: null });
    
    // 加载会话历史消息
    if (session) {
      try {
        logger.debug(`[ChatStore] Loading messages for session: ${session.id}`);
        const messages = await chatService.getMessages(session.id);
        logger.debug(`[ChatStore] Loaded ${messages.length} messages`);
        set({ messages });
      } catch (error) {
        logger.error(`[ChatStore] Failed to load messages:`, error);
        set({ error: error instanceof Error ? error.message : "Failed to load messages" });
      }
    }
  },

  sendMessage: async (content: string) => {
    logger.debug(`[ChatStore] sendMessage called`, { content });
    const { currentSession } = get();
    logger.debug(`[ChatStore] currentSession:`, currentSession);
    
    if (!currentSession) {
      console.error(`[ChatStore] No active session!`);
      set({ error: "No active session" });
      return;
    }

    const userMessage: Message = {
      id: crypto.randomUUID(),
      sessionId: currentSession.id,
      role: "user",
      content,
      createdAt: new Date().toISOString(),
    };
    logger.debug(`[ChatStore] Created user message:`, userMessage);

    set((state) => ({
      messages: [...state.messages, userMessage],
      isGenerating: true,
      streamContent: "",
      error: null,
    }));

    try {
      // 获取 LLM 配置
      const providerConfig = getLLMConfig();
      logger.debug(`[ChatStore] Provider config:`, providerConfig ? '(configured)' : '(none)');
      
      logger.debug(`[ChatStore] Calling chatService.sendMessage...`);
      await chatService.sendMessage(currentSession.id, content, providerConfig);
      logger.debug(`[ChatStore] chatService.sendMessage completed`);
    } catch (error) {
      console.error(`[ChatStore] sendMessage error:`, error);
      set({
        error: error instanceof Error ? error.message : "Failed to send message",
        isGenerating: false,
      });
    }
  },

  stopGeneration: () => {
    const { currentSession } = get();
    if (currentSession) {
      chatService.stopGeneration(currentSession.id);
    }
    set({ isGenerating: false });
  },

  appendStreamContent: (content: string) => {
    set((state) => ({
      streamContent: state.streamContent + content,
    }));
  },

  finalizeMessage: (messageId: string, emotion?: Emotion, tokens?: number) => {
    const { streamContent, currentSession } = get();
    if (!currentSession) return;

    const assistantMessage: Message = {
      id: messageId,
      sessionId: currentSession.id,
      role: "assistant",
      content: streamContent,
      tokens,
      emotion,
      createdAt: new Date().toISOString(),
    };

    // 估算 token 使用量（如果没有从后端获取）
    const outputTokens = tokens ?? Math.ceil(streamContent.length / 4);
    const messages = get().messages;
    const lastUserMsg = [...messages].reverse().find(m => m.role === "user");
    const inputTokens = lastUserMsg ? Math.ceil(lastUserMsg.content.length / 4) : 0;

    set((state) => ({
      messages: [...state.messages, assistantMessage],
      streamContent: "",
      isGenerating: false,
      lastTokenUsage: {
        inputTokens,
        outputTokens,
        totalTokens: inputTokens + outputTokens,
      },
    }));
  },

  clearMessages: () => {
    set({ messages: [], streamContent: "", lastTokenUsage: null });
  },

  setError: (error) => {
    set({ error });
  },

  // 编辑消息
  editMessage: (messageId: string, newContent: string) => {
    set((state) => ({
      messages: state.messages.map((msg) =>
        msg.id === messageId ? { ...msg, content: newContent } : msg
      ),
    }));
  },

  // 删除单条消息
  deleteMessage: (messageId: string) => {
    set((state) => ({
      messages: state.messages.filter((msg) => msg.id !== messageId),
    }));
  },

  // 删除从某条消息开始的所有后续消息（包含该消息）
  deleteMessagesFrom: (messageId: string) => {
    set((state) => {
      const index = state.messages.findIndex((msg) => msg.id === messageId);
      if (index === -1) return state;
      return {
        messages: state.messages.slice(0, index),
      };
    });
  },

  // 从某条消息重新生成（不创建新的用户消息）
  regenerateFrom: async (messageId: string) => {
    const { messages, currentSession } = get();
    if (!currentSession) return;

    const index = messages.findIndex((msg) => msg.id === messageId);
    if (index === -1) return;

    const targetMessage = messages[index];
    
    // 找到上一条用户消息
    let userMessageIndex = index;
    if (targetMessage.role === "assistant") {
      // 如果是 AI 消息，找它之前的用户消息
      for (let i = index - 1; i >= 0; i--) {
        if (messages[i].role === "user") {
          userMessageIndex = i;
          break;
        }
      }
    }
    
    const userMessage = messages[userMessageIndex];
    if (userMessage.role !== "user") return;

    // 删除从用户消息之后的所有消息（保留用户消息）
    const remainingMessages = messages.slice(0, userMessageIndex + 1);
    
    set({
      messages: remainingMessages,
      isGenerating: true,
      streamContent: "",
      error: null,
    });

    try {
      const providerConfig = getLLMConfig();
      // 调用 regenerate（不会在后端创建新的用户消息）
      await chatService.regenerate(currentSession.id, userMessage.content, providerConfig);
    } catch (error) {
      set({
        error: error instanceof Error ? error.message : "Failed to regenerate",
        isGenerating: false,
      });
    }
  },

  setTokenUsage: (usage: TokenUsage | null) => {
    set({ lastTokenUsage: usage });
  },
}));
