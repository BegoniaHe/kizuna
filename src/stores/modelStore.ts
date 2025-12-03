import { logger } from "@/utils/logger";
import { create } from "zustand";
import type { ModelType, ModelInfo, Emotion } from "@/types";
import { toAssetUrl } from "@/renderers/shared/TauriAssetManager";

/**
 * 检测 GLTF/GLB 文件是否包含 VRM 扩展
 * VRM 文件在 JSON 中包含 "VRM" 或 "VRMC_vrm" 扩展
 */
async function detectVRMExtension(path: string): Promise<boolean> {
  try {
    const url = toAssetUrl(path);
    const response = await fetch(url);
    const buffer = await response.arrayBuffer();
    
    // GLB 文件格式: magic(4) + version(4) + length(4) + JSON chunk
    const dataView = new DataView(buffer);
    const magic = dataView.getUint32(0, true);
    
    // 0x46546C67 = "glTF" in little-endian
    if (magic === 0x46546C67) {
      // GLB 格式
      const jsonChunkLength = dataView.getUint32(12, true);
      const jsonChunkType = dataView.getUint32(16, true);
      
      // 0x4E4F534A = "JSON" in little-endian
      if (jsonChunkType === 0x4E4F534A) {
        const jsonBytes = new Uint8Array(buffer, 20, jsonChunkLength);
        const jsonStr = new TextDecoder().decode(jsonBytes);
        
        // 检查是否包含 VRM 扩展
        return jsonStr.includes('"VRM"') || 
               jsonStr.includes('"VRMC_vrm"') ||
               jsonStr.includes('"VRMC_springBone"');
      }
    } else {
      // 可能是 .gltf JSON 文件
      const text = new TextDecoder().decode(buffer);
      return text.includes('"VRM"') || 
             text.includes('"VRMC_vrm"') ||
             text.includes('"VRMC_springBone"');
    }
  } catch (e) {
    logger.warn("[ModelStore] Failed to detect VRM extension:", e);
  }
  return false;
}

interface ModelState {
  modelType: ModelType | null;
  modelPath: string | null;
  isLoading: boolean;
  isLoaded: boolean;
  currentExpression: string;
  currentMotion: string | null;
  modelInfo: ModelInfo | null;
  error: string | null;

  loadModel: (path: string) => Promise<void>;
  unloadModel: () => void;
  setExpression: (expression: string) => void;
  setExpressionFromEmotion: (emotion: Emotion) => void;
  playMotion: (group: string, index?: number) => void;
  setError: (error: string | null) => void;
  setModelInfo: (info: ModelInfo) => void;
  setLoading: (loading: boolean) => void;
  setLoaded: (loaded: boolean) => void;
}

const EMOTION_TO_EXPRESSION: Record<Emotion, string> = {
  neutral: "neutral",
  happy: "smile",
  sad: "sad",
  angry: "angry",
  surprised: "surprised",
  thinking: "thinking",
};

export const useModelStore = create<ModelState>((set) => ({
  modelType: null,
  modelPath: null,
  isLoading: false,
  isLoaded: false,
  currentExpression: "neutral",
  currentMotion: null,
  modelInfo: null,
  error: null,

  loadModel: async (path: string) => {
    logger.debug("[ModelStore] loadModel called with path:", path);
    
    // Detect model type based on file extension
    const lowerPath = path.toLowerCase();
    
    let type: ModelType;
    if (lowerPath.endsWith(".vrm")) {
      type = "vrm";
    } else if (lowerPath.endsWith(".glb") || lowerPath.endsWith(".gltf")) {
      // 检查是否是 VRM 格式的 GLB 文件
      const isVRM = await detectVRMExtension(path);
      type = isVRM ? "vrm" : "gltf";
      if (isVRM) {
        logger.info("[ModelStore] Detected VRM extension in GLB file");
      }
    } else if (lowerPath.endsWith(".fbx")) {
      type = "fbx";
    } else if (lowerPath.endsWith(".pmd") || lowerPath.endsWith(".pmx")) {
      type = "mmd";
    } else if (lowerPath.includes("model.json") || lowerPath.includes("model3.json")) {
      type = "live2d";
    } else {
      // 默认尝试作为 Live2D
      type = "live2d";
    }
    
    logger.debug("[ModelStore] Detected model type:", type);
    
    set({
      modelPath: path,
      modelType: type,
      isLoading: true,
      isLoaded: false,
      error: null,
    });
    
    logger.debug("[ModelStore] State updated, waiting for canvas to load model");
  },

  unloadModel: () => {
    set({
      modelType: null,
      modelPath: null,
      isLoading: false,
      isLoaded: false,
      modelInfo: null,
      currentExpression: "neutral",
      currentMotion: null,
    });
  },

  setExpression: (expression: string) => {
    set({ currentExpression: expression });
  },

  setExpressionFromEmotion: (emotion: Emotion) => {
    const expression = EMOTION_TO_EXPRESSION[emotion] || "neutral";
    set({ currentExpression: expression });
  },

  playMotion: (group: string, index?: number) => {
    set({ currentMotion: `${group}:${index ?? 0}` });
  },

  setError: (error) => {
    set({ error, isLoading: false });
  },

  setModelInfo: (info) => {
    set({ modelInfo: info });
  },

  setLoading: (loading) => {
    set({ isLoading: loading });
  },

  setLoaded: (loaded) => {
    set({ isLoaded: loaded, isLoading: false });
  },
}));
