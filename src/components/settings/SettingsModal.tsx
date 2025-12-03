import React, { useState, useCallback } from "react";
import { Modal, Button, Input } from "@/components/common";
import { useConfigStore } from "@/stores";
import { useI18n } from "@/i18n";
import { configService, type ModelInfo } from "@/services/ConfigService";
import type { ProviderConfig, ProviderType } from "@/types";

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

type SettingsTab = "general" | "llm" | "model" | "shortcuts";

// 默认 provider ID
const DEFAULT_PROVIDER_ID = "default";

// Provider 类型信息
const PROVIDER_TYPES: { id: ProviderType; name: string; description: string; defaultBaseUrl: string }[] = [
  { 
    id: "openai", 
    name: "OpenAI", 
    description: "GPT-4, GPT-3.5",
    defaultBaseUrl: "https://api.openai.com/v1",
  },
  { 
    id: "claude", 
    name: "Claude (Anthropic)", 
    description: "Claude 3.5, Claude 3",
    defaultBaseUrl: "https://api.anthropic.com",
  },
  { 
    id: "ollama", 
    name: "Ollama", 
    description: "Local LLM",
    defaultBaseUrl: "http://localhost:11434",
  },
  { 
    id: "custom", 
    name: "Custom (OpenAI Compatible)", 
    description: "OpenAI API Compatible",
    defaultBaseUrl: "",
  },
];

// 预设模型列表
const PRESET_MODELS: Record<ProviderType, string[]> = {
  openai: ["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-4", "gpt-3.5-turbo"],
  claude: ["claude-sonnet-4-20250514", "claude-3-5-sonnet-20241022", "claude-3-opus-20240229", "claude-3-haiku-20240307"],
  ollama: ["llama3.3", "qwen2.5:32b", "qwen2.5:14b", "qwen2.5-coder", "deepseek-r1", "gemma2"],
  custom: [],
};

export const SettingsModal: React.FC<SettingsModalProps> = ({ isOpen, onClose }) => {
  const [activeTab, setActiveTab] = useState<SettingsTab>("general");
  const [editingProviderId, setEditingProviderId] = useState<string | null>(null);
  const [fetchedModels, setFetchedModels] = useState<ModelInfo[]>([]);
  const [isFetchingModels, setIsFetchingModels] = useState(false);
  const [useCustomModel, setUseCustomModel] = useState(false);
  const { config, setTheme, setLanguage, setAutoStart, updateConfig } = useConfigStore();
  const { t } = useI18n();

  // 获取所有 provider 列表
  const allProviders = Object.values(config.llm.providers);
  
  // 当前选中编辑的 provider（如果正在编辑）或默认 provider
  const activeProviderId = editingProviderId || config.llm.defaultProvider || DEFAULT_PROVIDER_ID;

  // 获取当前 provider 配置，如果没有则使用默认值
  const getCurrentProvider = useCallback((): ProviderConfig => {
    return config.llm.providers[activeProviderId] ?? {
      id: DEFAULT_PROVIDER_ID,
      name: "OpenAI",
      providerType: "openai",
      baseUrl: "",
      apiKey: "",
      models: ["gpt-4", "gpt-3.5-turbo"],
      isDefault: true,
    };
  }, [config.llm.providers, activeProviderId]);

  // 更新 provider 配置
  const updateProvider = useCallback((updates: Partial<ProviderConfig>) => {
    const currentProvider = getCurrentProvider();
    const updatedProvider = { ...currentProvider, ...updates };
    
    updateConfig({
      llm: {
        ...config.llm,
        providers: {
          ...config.llm.providers,
          [activeProviderId]: updatedProvider,
        },
      },
    });
  }, [config.llm, getCurrentProvider, updateConfig, activeProviderId]);

  // 添加新 Provider
  const addProvider = useCallback(() => {
    const newId = `provider_${Date.now()}`;
    const newProvider: ProviderConfig = {
      id: newId,
      name: "New Provider",
      providerType: "openai",
      baseUrl: "https://api.openai.com/v1",
      apiKey: "",
      models: ["gpt-4o"],
      isDefault: false,
    };
    
    updateConfig({
      llm: {
        ...config.llm,
        providers: {
          ...config.llm.providers,
          [newId]: newProvider,
        },
      },
    });
    setEditingProviderId(newId);
  }, [config.llm, updateConfig]);

  // 删除 Provider
  const deleteProvider = useCallback((providerId: string) => {
    if (Object.keys(config.llm.providers).length <= 1) {
      return; // 至少保留一个 provider
    }
    
    const { [providerId]: _, ...remainingProviders } = config.llm.providers;
    const newDefaultProvider = config.llm.defaultProvider === providerId 
      ? Object.keys(remainingProviders)[0] 
      : config.llm.defaultProvider;
    
    updateConfig({
      llm: {
        ...config.llm,
        defaultProvider: newDefaultProvider,
        providers: remainingProviders,
      },
    });
    
    if (editingProviderId === providerId) {
      setEditingProviderId(null);
    }
  }, [config.llm, updateConfig, editingProviderId]);

  // 设置默认 Provider
  const setDefaultProvider = useCallback((providerId: string) => {
    updateConfig({
      llm: {
        ...config.llm,
        defaultProvider: providerId,
      },
    });
  }, [config.llm, updateConfig]);

  // 获取模型列表
  const handleFetchModels = useCallback(async () => {
    const provider = getCurrentProvider();
    console.log("[SettingsModal] handleFetchModels called", { 
      provider: { ...provider, apiKey: provider.apiKey ? "***" : "" },
      baseUrl: provider.baseUrl,
      providerType: provider.providerType 
    });
    
    if (!provider.baseUrl) {
      console.log("[SettingsModal] No baseUrl, aborting");
      return;
    }
    
    if (!provider.apiKey && provider.providerType !== "ollama") {
      console.log("[SettingsModal] No apiKey and not ollama, aborting");
      return;
    }
    
    setIsFetchingModels(true);
    setFetchedModels([]);
    setUseCustomModel(false);
    
    try {
      console.log("[SettingsModal] Calling configService.fetchModels...");
      const models = await configService.fetchModels(provider);
      console.log("[SettingsModal] Fetched models:", models);
      setFetchedModels(models);
    } catch (error) {
      console.error("[SettingsModal] Failed to fetch models:", error);
    } finally {
      setIsFetchingModels(false);
    }
  }, [getCurrentProvider]);

  const currentProvider = getCurrentProvider();

  const tabs: { id: SettingsTab; label: string }[] = [
    { id: "general", label: t.settings.tabs.general },
    { id: "llm", label: t.settings.tabs.llm },
    { id: "model", label: t.settings.tabs.model },
    { id: "shortcuts", label: t.settings.tabs.shortcuts },
  ];

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={t.settings.title} size="3xl">
      <div className="flex gap-6 min-h-[500px]">
        <nav className="w-48 space-y-1">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`
                w-full text-left px-3 py-2 rounded-lg text-sm transition-colors
                ${
                  activeTab === tab.id
                    ? "bg-primary-100 dark:bg-primary-900 text-primary-700 dark:text-primary-300"
                    : "hover:bg-zinc-100 dark:hover:bg-zinc-700 text-zinc-600 dark:text-zinc-400"
                }
              `}
            >
              {tab.label}
            </button>
          ))}
        </nav>

        <div className="flex-1 border-l border-zinc-200 dark:border-zinc-700 pl-6">
          {activeTab === "general" && (
            <div className="space-y-4">
              <h3 className="text-lg font-medium text-zinc-900 dark:text-zinc-100">
                {t.settings.tabs.general}
              </h3>

              <div>
                <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
                  {t.settings.general.theme}
                </label>
                <div className="flex gap-2">
                  {(["light", "dark", "system"] as const).map((theme) => (
                    <Button
                      key={theme}
                      variant={config.general.theme === theme ? "primary" : "secondary"}
                      size="sm"
                      onClick={() => setTheme(theme)}
                    >
                      {theme === "light" ? t.settings.general.themeLight : theme === "dark" ? t.settings.general.themeDark : t.settings.general.themeSystem}
                    </Button>
                  ))}
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
                  {t.settings.general.language}
                </label>
                <select
                  value={config.general.language}
                  onChange={(e) => setLanguage(e.target.value)}
                  className="w-full px-3 py-2 rounded-lg border border-zinc-300 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
                >
                  <option value="zh-CN">简体中文</option>
                  <option value="en-US">English</option>
                  <option value="ja-JP">日本語</option>
                </select>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-sm text-zinc-700 dark:text-zinc-300">{t.settings.general.autoStart}</span>
                <button
                  onClick={() => setAutoStart(!config.general.autoStart)}
                  className={`
                    relative inline-flex h-6 w-11 items-center rounded-full transition-colors
                    ${config.general.autoStart ? "bg-primary-600" : "bg-zinc-300 dark:bg-zinc-600"}
                  `}
                >
                  <span
                    className={`
                      inline-block h-4 w-4 transform rounded-full bg-white transition-transform
                      ${config.general.autoStart ? "translate-x-6" : "translate-x-1"}
                    `}
                  />
                </button>
              </div>
            </div>
          )}

          {activeTab === "llm" && (
            <div className="space-y-4">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-medium text-zinc-900 dark:text-zinc-100">
                  {t.settings.llm.title}
                </h3>
                <Button size="sm" variant="secondary" onClick={addProvider}>
                  + {t.settings.llm.addProvider}
                </Button>
              </div>
              
              {/* Provider 列表 */}
              {allProviders.length > 1 && (
                <div className="space-y-2">
                  <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300">
                    {t.settings.llm.configuredProviders}
                  </label>
                  <div className="space-y-1">
                    {allProviders.map((provider) => (
                      <div 
                        key={provider.id}
                        className={`
                          flex items-center justify-between p-2 rounded-lg border cursor-pointer transition-all
                          ${activeProviderId === provider.id
                            ? "border-primary-500 bg-primary-50 dark:bg-primary-900/30"
                            : "border-zinc-200 dark:border-zinc-700 hover:border-zinc-300"
                          }
                        `}
                        onClick={() => setEditingProviderId(provider.id)}
                      >
                        <div className="flex items-center gap-2">
                          <span className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                            {provider.name}
                          </span>
                          <span className="text-xs text-zinc-500 dark:text-zinc-400">
                            ({PROVIDER_TYPES.find(pt => pt.id === provider.providerType)?.name})
                          </span>
                          {config.llm.defaultProvider === provider.id && (
                            <span className="text-xs px-1.5 py-0.5 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded">
                              {t.settings.llm.default}
                            </span>
                          )}
                        </div>
                        <div className="flex items-center gap-1">
                          {config.llm.defaultProvider !== provider.id && (
                            <button
                              onClick={(e) => {
                                e.stopPropagation();
                                setDefaultProvider(provider.id);
                              }}
                              className="text-xs px-2 py-1 text-zinc-600 dark:text-zinc-400 hover:text-primary-600 dark:hover:text-primary-400"
                            >
                              {t.settings.llm.setDefault}
                            </button>
                          )}
                          {allProviders.length > 1 && (
                            <button
                              onClick={(e) => {
                                e.stopPropagation();
                                deleteProvider(provider.id);
                              }}
                              className="text-xs px-2 py-1 text-red-500 hover:text-red-700"
                            >
                              {t.settings.llm.delete}
                            </button>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                </div>
              )}
              
              {/* 分隔线 */}
              {allProviders.length > 1 && (
                <div className="border-t border-zinc-200 dark:border-zinc-700 pt-4">
                  <h4 className="text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-3">
                    {t.settings.llm.edit}: {getCurrentProvider().name}
                  </h4>
                </div>
              )}
              
              {/* Provider 名称 */}
              <Input
                label={t.settings.llm.providerName}
                placeholder="My API"
                value={getCurrentProvider().name}
                onChange={(e) => updateProvider({ name: e.target.value })}
              />
              
              {/* Provider 类型选择 */}
              <div>
                <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
                  {t.settings.llm.providerType}
                </label>
                <div className="grid grid-cols-2 gap-2">
                  {PROVIDER_TYPES.map((type) => (
                    <button
                      key={type.id}
                      onClick={() => {
                        updateProvider({ 
                          providerType: type.id,
                          baseUrl: type.defaultBaseUrl,
                          models: PRESET_MODELS[type.id].slice(0, 1),
                        });
                      }}
                      className={`
                        p-3 rounded-lg border text-left transition-all
                        ${currentProvider.providerType === type.id
                          ? "border-primary-500 bg-primary-50 dark:bg-primary-900/30"
                          : "border-zinc-200 dark:border-zinc-700 hover:border-zinc-300 dark:hover:border-zinc-600"
                        }
                      `}
                    >
                      <div className="font-medium text-sm text-zinc-900 dark:text-zinc-100">
                        {type.name}
                      </div>
                      <div className="text-xs text-zinc-500 dark:text-zinc-400 mt-0.5">
                        {type.description}
                      </div>
                    </button>
                  ))}
                </div>
              </div>

              <Input
                label={t.settings.llm.apiBaseUrl}
                placeholder={PROVIDER_TYPES.find(pt => pt.id === currentProvider.providerType)?.defaultBaseUrl ?? ""}
                value={currentProvider.baseUrl}
                onChange={(e) => updateProvider({ baseUrl: e.target.value })}
              />

              {/* Ollama 不需要 API Key */}
              {currentProvider.providerType !== "ollama" && (
                <Input
                  label={t.settings.llm.apiKey}
                  type="password"
                  placeholder={currentProvider.providerType === "claude" ? "sk-ant-..." : "sk-..."}
                  value={currentProvider.apiKey}
                  onChange={(e) => updateProvider({ apiKey: e.target.value })}
                />
              )}

              {/* 模型选择 */}
              <div>
                <div className="flex items-center justify-between mb-2">
                  <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300">
                    {t.settings.llm.model}
                  </label>
                  <button
                    onClick={() => {
                      console.log("[SettingsModal] Fetch models button clicked");
                      handleFetchModels();
                    }}
                    disabled={isFetchingModels || (!currentProvider.baseUrl)}
                    className="text-xs px-2 py-1 rounded bg-primary-500 hover:bg-primary-600 disabled:bg-zinc-400 disabled:cursor-not-allowed text-white transition-colors"
                  >
                    {isFetchingModels ? t.settings.llm.fetchingModels : t.settings.llm.fetchModels}
                  </button>
                </div>
                
                {/* 模型选择：自定义模式或选择模式 */}
                {useCustomModel ? (
                  <div className="space-y-2">
                    <Input
                      placeholder="Model name (e.g. gpt-4o, claude-3-opus)"
                      value={currentProvider.models[0] ?? ""}
                      onChange={(e) => updateProvider({ models: [e.target.value] })}
                    />
                    <button
                      onClick={() => setUseCustomModel(false)}
                      className="text-xs text-primary-500 hover:text-primary-600"
                    >
                      ← {t.settings.llm.selectModel}
                    </button>
                  </div>
                ) : (
                  <select
                    value={currentProvider.models[0] ?? ""}
                    onChange={(e) => {
                      if (e.target.value === "__custom__") {
                        setUseCustomModel(true);
                        updateProvider({ models: [""] });
                      } else {
                        updateProvider({ models: [e.target.value] });
                      }
                    }}
                    className="w-full px-3 py-2 rounded-lg border border-zinc-300 dark:border-zinc-700 bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100"
                  >
                    <option value="">{t.settings.llm.selectModel}</option>
                    
                    {/* 获取到的模型 */}
                    {fetchedModels.length > 0 && (
                      <optgroup label={t.settings.llm.models}>
                        {fetchedModels.map((model) => (
                          <option key={model.id} value={model.id}>
                            {model.name}{model.ownedBy ? ` (${model.ownedBy})` : ''}
                          </option>
                        ))}
                      </optgroup>
                    )}
                    
                    {/* 预设模型（如果没有获取到的模型） */}
                    {fetchedModels.length === 0 && PRESET_MODELS[currentProvider.providerType]?.length > 0 && (
                      <optgroup label={t.settings.llm.presetModels}>
                        {PRESET_MODELS[currentProvider.providerType].map((model) => (
                          <option key={model} value={model}>
                            {model}
                          </option>
                        ))}
                      </optgroup>
                    )}
                    
                    {/* 自定义选项 */}
                    <option value="__custom__">{t.settings.llm.customModel}</option>
                  </select>
                )}
              </div>

              {/* 流式响应开关 */}
              <div className="flex items-center justify-between">
                <span className="text-sm text-zinc-700 dark:text-zinc-300">{t.settings.llm.streamResponse}</span>
                <button
                  onClick={() =>
                    updateConfig({
                      llm: { ...config.llm, streamResponse: !config.llm.streamResponse },
                    })
                  }
                  className={`
                    relative inline-flex h-6 w-11 items-center rounded-full transition-colors
                    ${config.llm.streamResponse ? "bg-primary-600" : "bg-zinc-300 dark:bg-zinc-600"}
                  `}
                >
                  <span
                    className={`
                      inline-block h-4 w-4 transform rounded-full bg-white transition-transform
                      ${config.llm.streamResponse ? "translate-x-6" : "translate-x-1"}
                    `}
                  />
                </button>
              </div>
              
              {/* 提示信息 */}
              <div className="p-3 rounded-lg bg-zinc-50 dark:bg-zinc-800/50 text-xs text-zinc-600 dark:text-zinc-400">
                {currentProvider.providerType === "ollama" && (
                  <p>{t.settings.llm.ollamaHelp}</p>
                )}
                {currentProvider.providerType === "claude" && (
                  <p>{t.settings.llm.claudeHelp}</p>
                )}
                {currentProvider.providerType === "openai" && (
                  <p>{t.settings.llm.openaiHelp}</p>
                )}
                {currentProvider.providerType === "custom" && (
                  <p>{t.settings.llm.customHelp}</p>
                )}
              </div>
            </div>
          )}

          {activeTab === "model" && (
            <div className="space-y-4">
              <h3 className="text-lg font-medium text-zinc-900 dark:text-zinc-100">
                {t.settings.model.title}
              </h3>

              <div>
                <label className="block text-sm font-medium text-zinc-700 dark:text-zinc-300 mb-2">
                  {t.settings.model.defaultType}
                </label>
                <div className="flex gap-2">
                  {(["live2d", "vrm"] as const).map((type) => (
                    <Button
                      key={type}
                      variant={config.model.defaultType === type ? "primary" : "secondary"}
                      size="sm"
                      onClick={() =>
                        updateConfig({ model: { ...config.model, defaultType: type } })
                      }
                    >
                      {type === "live2d" ? "Live2D" : "VRM"}
                    </Button>
                  ))}
                </div>
              </div>

              <div className="flex items-center justify-between">
                <span className="text-sm text-zinc-700 dark:text-zinc-300">{t.settings.model.physics}</span>
                <button
                  onClick={() =>
                    updateConfig({
                      model: { ...config.model, physicsEnabled: !config.model.physicsEnabled },
                    })
                  }
                  className={`
                    relative inline-flex h-6 w-11 items-center rounded-full transition-colors
                    ${config.model.physicsEnabled ? "bg-primary-600" : "bg-zinc-300 dark:bg-zinc-600"}
                  `}
                >
                  <span
                    className={`
                      inline-block h-4 w-4 transform rounded-full bg-white transition-transform
                      ${config.model.physicsEnabled ? "translate-x-6" : "translate-x-1"}
                    `}
                  />
                </button>
              </div>
            </div>
          )}

          {activeTab === "shortcuts" && (
            <div className="space-y-4">
              <h3 className="text-lg font-medium text-zinc-900 dark:text-zinc-100">
                {t.settings.shortcuts.title}
              </h3>

              <Input
                label={t.settings.shortcuts.toggleWindow}
                value={config.shortcuts.toggleWindow}
                readOnly
              />

              <Input
                label={t.settings.shortcuts.togglePetMode}
                value={config.shortcuts.togglePetMode}
                readOnly
              />

              <Input
                label={t.settings.shortcuts.newChat}
                value={config.shortcuts.newChat}
                readOnly
              />

              <p className="text-xs text-zinc-500 dark:text-zinc-400">
                {t.settings.shortcuts.comingSoon}
              </p>
            </div>
          )}
        </div>
      </div>

      <div className="flex justify-end gap-2 mt-6 pt-4 border-t border-zinc-200 dark:border-zinc-700">
        <Button variant="secondary" onClick={onClose}>
          {t.common.close}
        </Button>
      </div>
    </Modal>
  );
};
