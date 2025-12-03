import { commandBus } from "./ipc";
import type { AppConfig, ProviderConfig, Preset } from "@/types";

export interface ModelInfo {
  id: string;
  name: string;
  ownedBy?: string;
}

export interface IConfigService {
  getConfig(): Promise<AppConfig>;
  setConfig<K extends keyof AppConfig>(key: K, value: AppConfig[K]): Promise<void>;
  resetConfig(): Promise<void>;
  listProviders(): Promise<ProviderConfig[]>;
  addProvider(provider: Omit<ProviderConfig, "id">): Promise<ProviderConfig>;
  updateProvider(id: string, provider: Partial<ProviderConfig>): Promise<void>;
  deleteProvider(id: string): Promise<void>;
  testConnection(providerId: string): Promise<{ success: boolean; error?: string }>;
  fetchModels(providerConfig: ProviderConfig): Promise<ModelInfo[]>;
  listPresets(): Promise<Preset[]>;
  createPreset(preset: Omit<Preset, "id" | "createdAt">): Promise<Preset>;
  updatePreset(id: string, preset: Partial<Preset>): Promise<void>;
  deletePreset(id: string): Promise<void>;
}

class ConfigServiceImpl implements IConfigService {
  async getConfig(): Promise<AppConfig> {
    return await commandBus.dispatch<void, AppConfig>("config:get_all");
  }

  async setConfig<K extends keyof AppConfig>(key: K, value: AppConfig[K]): Promise<void> {
    await commandBus.dispatch("config:set", { key, value });
  }

  async resetConfig(): Promise<void> {
    await commandBus.dispatch("config:reset");
  }

  async listProviders(): Promise<ProviderConfig[]> {
    return await commandBus.dispatch<void, ProviderConfig[]>("llm:list_providers");
  }

  async addProvider(provider: Omit<ProviderConfig, "id">): Promise<ProviderConfig> {
    return await commandBus.dispatch<Omit<ProviderConfig, "id">, ProviderConfig>(
      "llm:add_provider",
      provider,
    );
  }

  async updateProvider(id: string, provider: Partial<ProviderConfig>): Promise<void> {
    await commandBus.dispatch("llm:update_provider", { id, ...provider });
  }

  async deleteProvider(id: string): Promise<void> {
    await commandBus.dispatch("llm:delete_provider", { id });
  }

  async testConnection(providerId: string): Promise<{ success: boolean; error?: string }> {
    return await commandBus.dispatch<
      { providerId: string },
      { success: boolean; error?: string }
    >("llm:test_connection", { providerId });
  }

  async fetchModels(providerConfig: ProviderConfig): Promise<ModelInfo[]> {
    console.log("[ConfigService] fetchModels called with:", { ...providerConfig, apiKey: "***" });
    try {
      const result = await commandBus.dispatch<
        { request: { providerConfig: ProviderConfig } },
        ModelInfo[]
      >("chat:fetch_models", { request: { providerConfig } });
      console.log("[ConfigService] fetchModels result:", result);
      return result;
    } catch (error) {
      console.error("[ConfigService] fetchModels error:", error);
      throw error;
    }
  }

  async listPresets(): Promise<Preset[]> {
    return await commandBus.dispatch<void, Preset[]>("preset:list");
  }

  async createPreset(preset: Omit<Preset, "id" | "createdAt">): Promise<Preset> {
    return await commandBus.dispatch<Omit<Preset, "id" | "createdAt">, Preset>(
      "preset:create",
      preset,
    );
  }

  async updatePreset(id: string, preset: Partial<Preset>): Promise<void> {
    await commandBus.dispatch("preset:update", { id, ...preset });
  }

  async deletePreset(id: string): Promise<void> {
    await commandBus.dispatch("preset:delete", { id });
  }
}

export const configService: IConfigService = new ConfigServiceImpl();
