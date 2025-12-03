import React, { useRef, useEffect, useCallback, useState } from "react";
import { useModelStore } from "@/stores";
import { useI18n } from "@/i18n";
import { GLTFRendererAdapter } from "@/renderers/GLTFRenderer";
import { FBXRendererAdapter } from "@/renderers/FBXRenderer";
import { MMDRendererAdapter } from "@/renderers/MMDRenderer";
import type { ModelType } from "@/types";

interface ThreeCanvasProps {
  modelPath: string;
  modelType: ModelType;
}

type ThreeRenderer = GLTFRendererAdapter | FBXRendererAdapter | MMDRendererAdapter;

/** 渲染器能力接口 - 用于解耦 UI 与渲染器实现 */
interface RendererCapabilities {
  /** 是否支持物理模拟 */
  hasPhysics: boolean;
  /** 是否支持表情控制 */
  hasExpressions: boolean;
  /** 是否支持动画播放 */
  hasAnimations: boolean;
}

/** 根据模型类型获取渲染器能力 */
function getRendererCapabilities(modelType: ModelType): RendererCapabilities {
  switch (modelType) {
    case "mmd":
      return { hasPhysics: true, hasExpressions: true, hasAnimations: true };
    case "gltf":
      return { hasPhysics: false, hasExpressions: true, hasAnimations: true };
    case "fbx":
      return { hasPhysics: false, hasExpressions: true, hasAnimations: true };
    default:
      return { hasPhysics: false, hasExpressions: false, hasAnimations: false };
  }
}

export const ThreeCanvas: React.FC<ThreeCanvasProps> = ({ modelPath, modelType }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<ThreeRenderer | null>(null);
  const [isModelLoaded, setIsModelLoaded] = useState(false);
  const isInitializingRef = useRef(false);
  
  const { t } = useI18n();
  const { setLoaded, setLoading, setError, setModelInfo, currentExpression } = useModelStore();
  
  // 渲染器能力 - 用于条件渲染 UI 元素
  const capabilities = getRendererCapabilities(modelType);

  // 用于取消进行中的加载
  const abortControllerRef = useRef<AbortController | null>(null);

  const initRenderer = useCallback(async () => {
    // 取消之前的加载
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    abortControllerRef.current = new AbortController();
    const signal = abortControllerRef.current.signal;

    // 防止重复初始化
    if (isInitializingRef.current) {
      console.log(`[ThreeCanvas] Already initializing, skipping...`);
      return;
    }
    isInitializingRef.current = true;
    
    console.log(`[ThreeCanvas] Initializing ${modelType} renderer for:`, modelPath);
    
    if (!containerRef.current) {
      console.warn("[ThreeCanvas] Container not ready");
      isInitializingRef.current = false;
      return;
    }

    // 清理之前的渲染器
    if (rendererRef.current) {
      rendererRef.current.dispose();
      rendererRef.current = null;
    }

    setLoading(true);

    try {
      // 根据模型类型创建对应的渲染器
      let renderer: ThreeRenderer;
      
      switch (modelType) {
        case "gltf":
          renderer = new GLTFRendererAdapter(containerRef.current);
          break;
        case "fbx":
          renderer = new FBXRendererAdapter(containerRef.current);
          break;
        case "mmd":
          renderer = new MMDRendererAdapter(containerRef.current);
          break;
        default:
          throw new Error(`Unsupported model type: ${modelType}`);
      }

      rendererRef.current = renderer;

      // 监听加载完成事件
      renderer.on("loaded", (metadata) => {
        console.log(`[ThreeCanvas] ${modelType} loaded:`, metadata);
        if (renderer.metadata) {
          setModelInfo({
            type: renderer.metadata.type,
            path: renderer.metadata.path,
            name: renderer.metadata.name,
            expressions: renderer.metadata.expressions,
            motions: renderer.metadata.motions,
          });
        }
        setLoaded(true);
        setIsModelLoaded(true);
      });

      // 监听错误事件
      renderer.on("error", (error) => {
        console.error(`[ThreeCanvas] ${modelType} error:`, error);
        setError(error instanceof Error ? error.message : String(error));
      });

      // 加载模型
      await renderer.load(modelPath);
      
      // 如果加载完成后被取消，直接清理
      if (signal.aborted) {
        renderer.dispose();
        return;
      }
      
      isInitializingRef.current = false;

    } catch (error) {
      // 如果是取消导致的错误，不报错
      if (signal.aborted) {
        return;
      }
      console.error(`[ThreeCanvas] Failed to load ${modelType}:`, error);
      setError(error instanceof Error ? error.message : String(error));
      isInitializingRef.current = false;
    }
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [modelPath, modelType]);

  // 初始化渲染器 - 仅在 modelPath 或 modelType 变化时触发
  useEffect(() => {
    // 重置加载状态
    setIsModelLoaded(false);
    
    // 立即取消之前的加载
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }
    
    // 清理之前的渲染器
    if (rendererRef.current) {
      rendererRef.current.dispose();
      rendererRef.current = null;
    }
    
    // 重置初始化标志，允许新的初始化
    isInitializingRef.current = false;
    
    // 使用 setTimeout 确保在 StrictMode 下的第一次 cleanup 完成后再初始化
    const timeoutId = setTimeout(() => {
      initRenderer();
    }, 0);

    return () => {
      clearTimeout(timeoutId);
      // 取消进行中的加载
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
      isInitializingRef.current = false;
      if (rendererRef.current) {
        rendererRef.current.dispose();
        rendererRef.current = null;
      }
      setIsModelLoaded(false);
    };
  // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [modelPath, modelType]);

  // 响应表情变化
  useEffect(() => {
    if (rendererRef.current && currentExpression) {
      rendererRef.current.expression.setExpression(currentExpression);
    }
  }, [currentExpression]);

  // 响应窗口大小变化
  useEffect(() => {
    const handleResize = () => {
      if (containerRef.current && rendererRef.current) {
        const { clientWidth, clientHeight } = containerRef.current;
        rendererRef.current.resize(clientWidth, clientHeight);
      }
    };

    window.addEventListener("resize", handleResize);
    
    // 使用 ResizeObserver 监听容器大小变化
    const resizeObserver = new ResizeObserver(handleResize);
    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }

    return () => {
      window.removeEventListener("resize", handleResize);
      resizeObserver.disconnect();
    };
  }, []);

  // 重置视角
  const handleResetView = useCallback(() => {
    if (rendererRef.current) {
      rendererRef.current.resetView();
    }
  }, []);

  return (
    <div className="relative w-full h-full">
      {/* 渲染容器 */}
      <div 
        ref={containerRef} 
        className="w-full h-full"
        style={{ minHeight: "300px", touchAction: "none" }}
      />
      
      {/* UI 覆盖层 - 仅在模型加载后显示 */}
      {isModelLoaded && (
        <>
          {/* 重置视角按钮 */}
          <button
            onClick={handleResetView}
            className="absolute bottom-3 right-3 p-2 rounded-lg bg-zinc-800/70 hover:bg-zinc-700/80 text-zinc-300 hover:text-white transition-colors"
            title={t.model.resetView}
          >
            <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
          
          {/* 操作提示 */}
          <div className="absolute bottom-3 left-3 text-xs text-zinc-500 dark:text-zinc-400 space-y-0.5">
            <div>{t.model.controls.zoom}</div>
            <div>{t.model.controls.rotate}</div>
            <div>{t.model.controls.pan}</div>
          </div>
          
          {/* 预留：物理开关插槽 (MMD/VRM) */}
          {capabilities.hasPhysics && (
            <div className="absolute top-3 right-3">
              {/* 后续可添加物理开关组件 */}
            </div>
          )}
        </>
      )}
    </div>
  );
};
