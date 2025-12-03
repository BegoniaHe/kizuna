import React, { useRef, useState } from "react";
import { open } from "@tauri-apps/plugin-dialog";
import { useModelStore } from "@/stores";
import { Live2DCanvas } from "./Live2DCanvas";
import { VRMCanvas } from "./VRMCanvas";
import { ThreeCanvas } from "./ThreeCanvas";
import { useI18n } from "@/i18n";

interface ModelViewerProps {
  compact?: boolean;
}

export const ModelViewer: React.FC<ModelViewerProps> = ({ compact = false }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [isDragging, setIsDragging] = useState(false);
  const { modelType, modelPath, isLoading, error, loadModel } = useModelStore();
  const { t } = useI18n();

  const handleOpenFilePicker = async () => {
    console.log("[ModelViewer] Opening file picker...");
    try {
      const selected = await open({
        multiple: false,
        filters: [
          {
            name: "VRM Models",
            extensions: ["vrm"],
          },
          {
            name: "glTF/GLB Models",
            extensions: ["gltf", "glb"],
          },
          {
            name: "FBX Models",
            extensions: ["fbx"],
          },
          {
            name: "MMD Models",
            extensions: ["pmd", "pmx"],
          },
          {
            name: "Live2D Models",
            extensions: ["model.json", "model3.json"],
          },
          {
            name: "All 3D Models",
            extensions: ["vrm", "gltf", "glb", "fbx", "pmd", "pmx", "model.json", "model3.json"],
          },
        ],
      });

      console.log("[ModelViewer] File picker result:", selected);

      if (selected && typeof selected === "string") {
        console.log("[ModelViewer] Loading model from:", selected);
        await loadModel(selected);
      }
    } catch (err) {
      console.error("[ModelViewer] File picker error:", err);
    }
  };

  const handleDrop = async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
    console.log("[ModelViewer] Drop event triggered");
    console.log("[ModelViewer] Files:", e.dataTransfer.files);
    
    const file = e.dataTransfer.files[0];
    if (file) {
      console.log("[ModelViewer] File info:", {
        name: file.name,
        size: file.size,
        type: file.type,
      });
      
      const path = (file as File & { path?: string }).path;
      console.log("[ModelViewer] File path:", path);
      
      if (path) {
        console.log("[ModelViewer] Loading model from path:", path);
        await loadModel(path);
      } else {
        console.warn("[ModelViewer] No file path available - this may be a browser security restriction");
      }
    } else {
      console.warn("[ModelViewer] No file in drop event");
    }
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
  };

  const handleDragEnter = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  };

  const handleDragLeave = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  };

  return (
    <div className="h-full flex flex-col bg-white dark:bg-zinc-900">
      {!compact && (
        <header className="h-16 flex items-center justify-between px-4 border-b border-zinc-200/50 dark:border-zinc-800/50 bg-white/80 dark:bg-zinc-900/80 backdrop-blur-sm">
          <h2 className="text-sm font-medium text-zinc-700 dark:text-zinc-300">
            {modelPath ? t.model.preview : t.model.model}
          </h2>
          {modelType && (
            <span className="text-xs px-2 py-1 rounded-full bg-zinc-200 dark:bg-zinc-800 text-zinc-600 dark:text-zinc-400">
              {modelType.toUpperCase()}
            </span>
          )}
        </header>
      )}
      
      <div
        ref={containerRef}
        className={`
          relative flex-1 flex items-center justify-center overflow-hidden
          ${compact ? "bg-transparent" : ""}
          ${isDragging ? "ring-2 ring-primary-500 ring-inset bg-primary-50/10" : ""}
        `}
        onDrop={handleDrop}
        onDragOver={handleDragOver}
        onDragEnter={handleDragEnter}
        onDragLeave={handleDragLeave}
      >
      {isLoading && (
        <div className="absolute inset-0 flex items-center justify-center bg-black/20 z-10">
          <div className="flex flex-col items-center">
            <svg
              className="animate-spin h-8 w-8 text-primary-500"
              fill="none"
              viewBox="0 0 24 24"
            >
              <circle
                className="opacity-25"
                cx="12"
                cy="12"
                r="10"
                stroke="currentColor"
                strokeWidth="4"
              />
              <path
                className="opacity-75"
                fill="currentColor"
                d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
              />
            </svg>
            <p className="mt-2 text-sm text-zinc-600 dark:text-zinc-400">{t.model.loading}</p>
          </div>
        </div>
      )}

      {error && (
        <div className="absolute inset-0 flex items-center justify-center">
          <div className="text-center p-4">
            <p className="text-red-500">{error}</p>
          </div>
        </div>
      )}

      {!modelPath && !isLoading && (
        <div 
          className="text-center p-8 cursor-pointer hover:bg-zinc-50 dark:hover:bg-zinc-800/50 rounded-lg transition-colors"
          onClick={handleOpenFilePicker}
        >
          <svg
            className="w-16 h-16 mx-auto mb-4 text-zinc-400 dark:text-zinc-500"
            fill="none"
            viewBox="0 0 24 24"
            stroke="currentColor"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1.5}
              d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
            />
          </svg>
          <p className="text-zinc-500 dark:text-zinc-400 text-sm">
            {t.model.dropHint}
          </p>
          <p className="text-zinc-400 dark:text-zinc-500 text-xs mt-1">
            {t.model.supportedFormats}
          </p>
        </div>
      )}

      {modelPath && modelType === "live2d" && (
        <Live2DCanvas modelPath={modelPath} />
      )}

      {modelPath && modelType === "vrm" && (
        <VRMCanvas modelPath={modelPath} />
      )}

      {modelPath && (modelType === "gltf" || modelType === "fbx" || modelType === "mmd") && (
        <ThreeCanvas modelPath={modelPath} modelType={modelType} />
      )}
      </div>
    </div>
  );
};
