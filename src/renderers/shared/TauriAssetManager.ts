// Tauri 资源管理器
// 处理 Tauri 应用中的资源路径转换

import * as THREE from "three";
import { convertFileSrc } from "@tauri-apps/api/core";

/**
 * 标准化路径分隔符
 */
function normalizePath(path: string): string {
  return path.replace(/\\/g, "/");
}

/**
 * 从模型路径获取所在目录
 */
export function getModelDirectory(modelPath: string): string {
  // 先标准化路径
  const normalizedPath = normalizePath(modelPath);
  const lastSlash = normalizedPath.lastIndexOf("/");
  return lastSlash > 0 ? normalizedPath.substring(0, lastSlash + 1) : "";
}

/**
 * 将本地文件路径转换为 Tauri asset URL
 */
export function toAssetUrl(path: string): string {
  // 先标准化路径
  const normalizedPath = normalizePath(path);
  if (normalizedPath.startsWith("/") || normalizedPath.match(/^[A-Za-z]:/)) {
    return convertFileSrc(normalizedPath);
  }
  return normalizedPath;
}

/**
 * 检查 URL 是否已经是有效的远程/特殊 URL
 */
function isValidRemoteUrl(url: string): boolean {
  return (
    url.startsWith("data:") ||
    url.startsWith("blob:") ||
    url.startsWith("http://") ||
    url.startsWith("https://")
  );
}

/**
 * 检查 asset URL 是否包含完整路径
 */
function isCompleteAssetUrl(url: string): boolean {
  return url.startsWith("asset://") && url.includes("%2F");
}

/**
 * 检查是否是仅有文件名的 asset URL (asset://localhost/filename.png)
 */
function isFilenameOnlyAssetUrl(url: string): boolean {
  return url.startsWith("asset://localhost/") && !url.includes("%2F");
}

/**
 * 创建 Tauri 资源 LoadingManager
 * 自动处理相对纹理路径到 Tauri asset:// 协议的转换
 */
export function createTauriLoadingManager(modelDir: string): THREE.LoadingManager {
  const manager = new THREE.LoadingManager();
  
  manager.setURLModifier((url: string) => {
    // 远程 URL 直接返回
    if (isValidRemoteUrl(url)) {
      return url;
    }
    
    // 标准化路径中的反斜杠
    const normalizedUrl = normalizePath(url);
    
    // 仅文件名的 asset URL，需要补全路径
    if (isFilenameOnlyAssetUrl(normalizedUrl)) {
      const filename = normalizedUrl.replace("asset://localhost/", "");
      const absolutePath = modelDir + decodeURIComponent(filename);
      const result = convertFileSrc(normalizePath(absolutePath));
      console.log(`[TauriAssetManager] Filename-only asset URL: ${url} -> ${result}`);
      return result;
    }
    
    // 完整的 asset URL 直接返回
    if (isCompleteAssetUrl(normalizedUrl) || normalizedUrl.startsWith("asset://")) {
      return normalizedUrl;
    }
    
    // 处理本地路径
    let absolutePath: string;
    if (normalizedUrl.startsWith("/") || normalizedUrl.match(/^[A-Za-z]:/)) {
      // 已经是绝对路径
      absolutePath = normalizedUrl;
    } else {
      // 相对路径，拼接模型目录
      absolutePath = modelDir + normalizedUrl;
    }
    
    const result = convertFileSrc(absolutePath);
    console.log(`[TauriAssetManager] URL: ${url} -> ${result}`);
    return result;
  });
  
  return manager;
}

/**
 * 预配置的 LoadingManager 选项
 */
export interface LoadingManagerOptions {
  /** 模型文件路径 */
  modelPath: string;
  /** 加载进度回调 */
  onProgress?: (url: string, loaded: number, total: number) => void;
  /** 加载错误回调 */
  onError?: (url: string) => void;
}

/**
 * 创建完整配置的 Tauri LoadingManager
 */
export function createConfiguredLoadingManager(
  options: LoadingManagerOptions
): THREE.LoadingManager {
  const modelDir = getModelDirectory(options.modelPath);
  const manager = createTauriLoadingManager(modelDir);
  
  if (options.onProgress) {
    manager.onProgress = options.onProgress;
  }
  
  if (options.onError) {
    manager.onError = options.onError;
  }
  
  return manager;
}
