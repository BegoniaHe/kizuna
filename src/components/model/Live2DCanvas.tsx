import React, { useRef, useEffect, useCallback, useState } from "react";
import { Application } from "pixi.js";
import type { Container } from "pixi.js";
import { useModelStore } from "@/stores";
import { useI18n } from "@/i18n";

interface Live2DCanvasProps {
  modelPath: string;
}

export const Live2DCanvas: React.FC<Live2DCanvasProps> = ({ modelPath }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const appRef = useRef<Application | null>(null);
  const modelRef = useRef<unknown>(null);
  
  const { t } = useI18n();
  const { setLoaded, setLoading, setError, setModelInfo } = useModelStore();
  
  // 本地加载状态
  const [isModelLoaded, setIsModelLoaded] = useState(false);

  const initLive2D = useCallback(async () => {
    console.log("[Live2DCanvas] initLive2D called");
    console.log("[Live2DCanvas] modelPath:", modelPath);
    console.log("[Live2DCanvas] containerRef.current:", containerRef.current);
    
    if (!containerRef.current) {
      console.warn("[Live2DCanvas] Container not ready, aborting");
      return;
    }

    setLoading(true);
    console.log("[Live2DCanvas] Starting Live2D load...");

    try {
      // Dynamically import pixi-live2d-display
      console.log("[Live2DCanvas] Importing pixi-live2d-display...");
      const { Live2DModel } = await import("pixi-live2d-display");
      console.log("[Live2DCanvas] pixi-live2d-display imported successfully");

      // Initialize PixiJS Application (PixiJS 7.x API)
      const app = new Application({
        backgroundAlpha: 0,
        resizeTo: containerRef.current,
        resolution: window.devicePixelRatio,
        autoDensity: true,
      });

      containerRef.current.appendChild(app.view as HTMLCanvasElement);
      appRef.current = app;

      // Load the Live2D model
      console.log("[Live2DCanvas] Loading Live2D model from:", modelPath);
      const model = await Live2DModel.from(modelPath, {
        autoInteract: true,
        autoUpdate: true,
      });
      console.log("[Live2DCanvas] Live2D model loaded:", model);

      // Scale and position the model
      // Cast to any to access PixiJS display object properties
      const modelAny = model as any;
      const scale = Math.min(
        containerRef.current.clientWidth / modelAny.width,
        containerRef.current.clientHeight / modelAny.height,
      ) * 0.8;

      modelAny.scale.set(scale);
      modelAny.x = containerRef.current.clientWidth / 2;
      modelAny.y = containerRef.current.clientHeight / 2;
      modelAny.anchor.set(0.5, 0.5);

      // Add model to stage (cast to Container for compatibility)
      app.stage.addChild(model as unknown as Container);
      modelRef.current = model;

      // Get model info
      const internalModel = model.internalModel;
      const motionManager = internalModel.motionManager;
      const definitions = motionManager.definitions;
      const expressionManager = motionManager.expressionManager;

      const expressions = expressionManager?.definitions?.map((d: { name?: string }) => d.name || "unknown") ?? [];
      const motions = Object.entries(definitions || {}).map(([name, defs]) => ({
        name,
        motions: (defs as unknown[]).map((_, i) => ({ index: i })),
      }));

      setModelInfo({
        type: "live2d",
        path: modelPath,
        name: modelPath.split("/").pop() || "Live2D Model",
        expressions,
        motions,
      });

      setLoaded(true);
      setIsModelLoaded(true);
      console.log("[Live2DCanvas] Live2D model loaded successfully!");
    } catch (error) {
      console.error("[Live2DCanvas] Failed to load Live2D model:", error);
      console.error("[Live2DCanvas] Error details:", {
        message: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
      });
      setError(error instanceof Error ? error.message : "Failed to load model");
    }
  }, [modelPath, setLoaded, setLoading, setError, setModelInfo]);

  // 重置视图
  const handleResetView = useCallback(() => {
    if (!containerRef.current || !modelRef.current) return;
    
    // Live2D 模型重置位置和缩放
    const model = modelRef.current as {
      width: number;
      height: number;
      scale: { set: (s: number) => void };
      x: number;
      y: number;
    };
    
    const scale = Math.min(
      containerRef.current.clientWidth / model.width,
      containerRef.current.clientHeight / model.height,
    ) * 0.8;
    
    model.scale.set(scale);
    model.x = containerRef.current.clientWidth / 2;
    model.y = containerRef.current.clientHeight / 2;
  }, []);

  useEffect(() => {
    setIsModelLoaded(false);
    initLive2D();

    return () => {
      if (appRef.current) {
        appRef.current.destroy(true, { children: true });
        appRef.current = null;
      }
      modelRef.current = null;
      setIsModelLoaded(false);
    };
  }, [initLive2D]);

  return (
    <div className="relative w-full h-full">
      {/* 渲染容器 */}
      <div
        ref={containerRef}
        className="w-full h-full"
        style={{ touchAction: "none" }}
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
          
          {/* 操作提示 - Live2D 支持鼠标交互 */}
          <div className="absolute bottom-3 left-3 text-xs text-zinc-500 dark:text-zinc-400 space-y-0.5">
            <div>{t.model.controls.zoom}</div>
            <div>{t.model.controls.rotate}</div>
            <div>{t.model.controls.pan}</div>
          </div>
        </>
      )}
    </div>
  );
};
