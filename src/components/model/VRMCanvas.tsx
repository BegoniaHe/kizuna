import React, { useRef, useEffect, useCallback, useState } from "react";
import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { VRMLoaderPlugin, VRM } from "@pixiv/three-vrm";
import { convertFileSrc } from "@tauri-apps/api/core";
import { useModelStore } from "@/stores";
import { useI18n } from "@/i18n";
import { createTauriLoadingManager, getModelDirectory } from "@/renderers/shared/TauriAssetManager";

interface VRMCanvasProps {
  modelPath: string;
}

export const VRMCanvas: React.FC<VRMCanvasProps> = ({ modelPath }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const rendererRef = useRef<THREE.WebGLRenderer | null>(null);
  const sceneRef = useRef<THREE.Scene | null>(null);
  const cameraRef = useRef<THREE.PerspectiveCamera | null>(null);
  const controlsRef = useRef<OrbitControls | null>(null);
  const vrmRef = useRef<VRM | null>(null);
  const clockRef = useRef(new THREE.Clock());
  const animationIdRef = useRef<number | null>(null);
  
  const { t } = useI18n();
  const { setLoaded, setLoading, setError, setModelInfo, currentExpression } = useModelStore();
  
  // 本地加载状态 - 用于控制 UI 显示
  const [isModelLoaded, setIsModelLoaded] = useState(false);

  // Camera control state for reset
  const [cameraState] = useState({
    position: new THREE.Vector3(0, 1.2, 2),
    target: new THREE.Vector3(0, 1, 0),
  });

  const resetCamera = useCallback(() => {
    if (cameraRef.current && controlsRef.current) {
      cameraRef.current.position.copy(cameraState.position);
      controlsRef.current.target.copy(cameraState.target);
      controlsRef.current.update();
    }
  }, [cameraState]);

  const initVRM = useCallback(async () => {
    console.log("[VRMCanvas] initVRM called");
    console.log("[VRMCanvas] modelPath:", modelPath);
    console.log("[VRMCanvas] containerRef.current:", containerRef.current);
    
    if (!containerRef.current) {
      console.warn("[VRMCanvas] Container not ready, aborting");
      return;
    }

    setLoading(true);
    console.log("[VRMCanvas] Starting VRM load...");

    try {
      // Create scene
      const scene = new THREE.Scene();
      sceneRef.current = scene;
      console.log("[VRMCanvas] Scene created");

      // Create camera
      const camera = new THREE.PerspectiveCamera(
        30,
        containerRef.current.clientWidth / containerRef.current.clientHeight,
        0.1,
        20,
      );
      camera.position.copy(cameraState.position);
      cameraRef.current = camera;

      // Create renderer
      const renderer = new THREE.WebGLRenderer({
        alpha: true,
        antialias: true,
        powerPreference: "high-performance",
      });
      renderer.setSize(containerRef.current.clientWidth, containerRef.current.clientHeight);
      renderer.setPixelRatio(window.devicePixelRatio);
      // 设置输出颜色空间
      renderer.outputColorSpace = THREE.SRGBColorSpace;
      containerRef.current.appendChild(renderer.domElement);
      rendererRef.current = renderer;

      // Add OrbitControls for zoom, pan, rotate
      const controls = new OrbitControls(camera, renderer.domElement);
      controls.target.copy(cameraState.target);
      controls.enableDamping = true;
      controls.dampingFactor = 0.05;
      controls.screenSpacePanning = true;
      controls.minDistance = 0.5;
      controls.maxDistance = 10;
      controls.maxPolarAngle = Math.PI;
      controls.update();
      controlsRef.current = controls;

      // Add lighting
      const directionalLight = new THREE.DirectionalLight(0xffffff, 1);
      directionalLight.position.set(1, 1, 1);
      scene.add(directionalLight);
      scene.add(new THREE.AmbientLight(0xffffff, 0.5));

      // Load VRM
      console.log("[VRMCanvas] Creating GLTFLoader...");
      
      // 获取模型目录用于解析纹理路径
      const modelDir = getModelDirectory(modelPath);
      console.log("[VRMCanvas] Model directory:", modelDir);
      
      // 使用 Tauri LoadingManager 来正确处理纹理路径
      const loadingManager = createTauriLoadingManager(modelDir);
      
      // 添加错误处理
      loadingManager.onError = (url) => {
        console.error("[VRMCanvas] Failed to load resource:", url);
      };
      
      loadingManager.onProgress = (url, loaded, total) => {
        console.log(`[VRMCanvas] Loading: ${url} (${loaded}/${total})`);
      };
      
      const loader = new GLTFLoader(loadingManager);
      loader.register((parser) => new VRMLoaderPlugin(parser));

      // Convert local file path to asset URL for Tauri
      const assetUrl = modelPath.startsWith("/") || modelPath.match(/^[A-Za-z]:/)
        ? convertFileSrc(modelPath)
        : modelPath;
      console.log("[VRMCanvas] Loading model from:", modelPath);
      console.log("[VRMCanvas] Asset URL:", assetUrl);
      
      let gltf;
      try {
        gltf = await loader.loadAsync(assetUrl);
      } catch (loadError) {
        console.error("[VRMCanvas] GLTFLoader error:", loadError);
        throw loadError;
      }
      console.log("[VRMCanvas] GLTF loaded:", gltf);
      console.log("[VRMCanvas] userData:", gltf.userData);
      
      const vrm = gltf.userData.vrm as VRM;
      console.log("[VRMCanvas] VRM extracted:", vrm);
      vrmRef.current = vrm;

      scene.add(vrm.scene);
      vrm.scene.rotation.y = Math.PI;

      // Get model info
      const expressions = vrm.expressionManager
        ? Object.keys(vrm.expressionManager.expressionMap)
        : [];

      setModelInfo({
        type: "vrm",
        path: modelPath,
        name: vrm.meta?.metaVersion === "0" 
          ? (vrm.meta as { name?: string }).name || "VRM Model"
          : (vrm.meta as { name?: string }).name || "VRM Model",
        expressions,
        motions: [],
      });

      // Start render loop
      const animate = () => {
        animationIdRef.current = requestAnimationFrame(animate);
        const delta = clockRef.current.getDelta();

        if (controlsRef.current) {
          controlsRef.current.update();
        }

        if (vrmRef.current) {
          vrmRef.current.update(delta);
        }

        if (rendererRef.current && sceneRef.current && cameraRef.current) {
          rendererRef.current.render(sceneRef.current, cameraRef.current);
        }
      };
      animate();

      setLoaded(true);
      setIsModelLoaded(true);
      console.log("[VRMCanvas] VRM model loaded successfully!");
    } catch (error) {
      console.error("[VRMCanvas] Failed to load VRM model:", error);
      console.error("[VRMCanvas] Error details:", {
        message: error instanceof Error ? error.message : String(error),
        stack: error instanceof Error ? error.stack : undefined,
      });
      setError(error instanceof Error ? error.message : "Failed to load model");
    }
  }, [modelPath, setLoaded, setLoading, setError, setModelInfo, cameraState]);

  useEffect(() => {
    setIsModelLoaded(false);
    initVRM();

    return () => {
      if (animationIdRef.current) {
        cancelAnimationFrame(animationIdRef.current);
      }
      if (controlsRef.current) {
        controlsRef.current.dispose();
      }
      if (rendererRef.current) {
        rendererRef.current.dispose();
        rendererRef.current.domElement.remove();
      }
      if (vrmRef.current) {
        vrmRef.current.scene.traverse((obj) => {
          if (obj instanceof THREE.Mesh) {
            obj.geometry?.dispose();
            if (Array.isArray(obj.material)) {
              obj.material.forEach((m) => m.dispose());
            } else {
              obj.material?.dispose();
            }
          }
        });
      }
    };
  }, [initVRM]);

  useEffect(() => {
    if (vrmRef.current?.expressionManager) {
      // Reset all expressions
      Object.keys(vrmRef.current.expressionManager.expressionMap).forEach((name) => {
        vrmRef.current?.expressionManager?.setValue(name, 0);
      });
      // Set current expression
      vrmRef.current.expressionManager.setValue(currentExpression, 1);
    }
  }, [currentExpression]);

  // Handle resize
  useEffect(() => {
    const handleResize = () => {
      if (!containerRef.current || !rendererRef.current || !cameraRef.current) return;

      const width = containerRef.current.clientWidth;
      const height = containerRef.current.clientHeight;

      cameraRef.current.aspect = width / height;
      cameraRef.current.updateProjectionMatrix();
      rendererRef.current.setSize(width, height);
    };

    window.addEventListener("resize", handleResize);
    
    // Also observe container size changes
    const resizeObserver = new ResizeObserver(handleResize);
    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }
    
    return () => {
      window.removeEventListener("resize", handleResize);
      resizeObserver.disconnect();
    };
  }, []);

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
            onClick={resetCamera}
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
          
          {/* 预留：VRM 物理开关插槽 */}
          <div className="absolute top-3 right-3">
            {/* 后续可添加物理开关、表情选择等组件 */}
          </div>
        </>
      )}
    </div>
  );
};
