export interface Message {
  id: string;
  sessionId: string;
  role: "user" | "assistant" | "system";
  content: string;
  tokens?: number;
  emotion?: Emotion;
  createdAt: string;
}

export interface Session {
  id: string;
  title: string;
  presetId?: string;
  modelConfig?: LLMConfig;
  createdAt: string;
  updatedAt: string;
}

export interface LLMConfig {
  provider: string;
  model: string;
  temperature?: number;
  maxTokens?: number;
}

export type Emotion =
  | "neutral"
  | "happy"
  | "sad"
  | "angry"
  | "surprised"
  | "thinking";

export interface MessageChunk {
  sessionId: string;
  content: string;
  tokens?: number;
  /** 口型音素序列 (A/E/I/O/U/N/closed) - 由后端 rust-pinyin 生成 */
  phonemes?: string[];
}

export interface SendMessageRequest {
  sessionId: string;
  content: string;
}

export interface SendMessageResponse {
  messageId: string;
}
