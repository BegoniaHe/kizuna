export interface AppConfig {
  general: GeneralConfig;
  window: WindowConfig;
  shortcuts: ShortcutConfig;
  llm: LLMSettings;
  model: ModelConfig;
}

export interface GeneralConfig {
  language: string;
  theme: "light" | "dark" | "system";
  autoStart: boolean;
  minimizeToTray: boolean;
}

export interface WindowConfig {
  defaultMode: WindowMode;
  petModeSize: { width: number; height: number };
  petModePosition: "remember" | { x: number; y: number };
}

export type WindowMode = "normal" | "pet" | "compact" | "fullscreen";

export interface ShortcutConfig {
  toggleWindow: string;
  togglePetMode: string;
  newChat: string;
}

export interface LLMSettings {
  defaultProvider: string;
  streamResponse: boolean;
  contextLength: number;
  providers: Record<string, ProviderConfig>;
}

/** LLM 提供商类型 */
export type ProviderType = "openai" | "claude" | "ollama" | "custom";

export interface ProviderConfig {
  id: string;
  name: string;
  providerType: ProviderType;
  baseUrl: string;
  apiKey: string;
  models: string[];
  isDefault: boolean;
}

export interface ModelConfig {
  defaultType: "live2d" | "vrm";
  autoLoadLast: boolean;
  physicsEnabled: boolean;
  defaultPath?: string;
}

export interface Preset {
  id: string;
  name: string;
  avatar?: string;
  systemPrompt: string;
  modelType: "live2d" | "vrm";
  modelPath: string;
  defaultExpression: string;
  emotionMapping: Record<string, EmotionMapping>;
  createdAt: string;
}

export interface EmotionMapping {
  expression: string;
  motion?: string;
}
