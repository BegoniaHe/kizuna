// MMD (MikuMikuDance) Renderer Adapter
// PMD/PMX 模型渲染器 - 增强版

import * as THREE from "three";
import type { OrbitControls } from "three/examples/jsm/controls/OrbitControls.js";
import {
  MMDLoader,
  MMDAnimationHelper,
  VMDLoader,
  createMMDAnimationClip,
  initAmmo,
} from "@moeru/three-mmd";
import type { Emotion } from "@/types";
import {
  BaseModelRenderer,
  type ExpressionController,
  type MotionController,
  type LookAtController,
  type PhysicsController,
  type OutfitController,
  type OutfitPart,
  type MotionGroup,
  type ModelType,
  findExpressionForEmotion,
} from "./types";
import {
  initializeScene,
  fitCameraToObject,
  updateSize,
  type SceneComponents,
  type SceneDisposer,
} from "./shared/ThreeSceneSetup";
import {
  createTauriLoadingManager,
  getModelDirectory,
  toAssetUrl,
} from "./shared/TauriAssetManager";
import { MMD_SCENE_CONFIG } from "./shared/CameraPresets";
import { 
  lipSyncController, 
  createMMDLipSyncAdapter,
  type LipSyncTarget 
} from "@/services/LipSyncService";

// ═══════════════════════════════════════════════════════════════════════════
// Ammo.js 物理引擎管理（单例模式）
// ═══════════════════════════════════════════════════════════════════════════

let ammoInitialized = false;
let ammoInitPromise: Promise<void> | null = null;

async function ensureAmmoInit(): Promise<void> {
  if (ammoInitialized) return;
  if (ammoInitPromise) return ammoInitPromise;

  ammoInitPromise = initAmmo().then(() => {
    ammoInitialized = true;
  });

  return ammoInitPromise;
}

// ═══════════════════════════════════════════════════════════════════════════
// MMD 渲染器
// ═══════════════════════════════════════════════════════════════════════════

/** 已加载的动作信息 */
interface LoadedMotion {
  name: string;
  clip: THREE.AnimationClip;
}

/** MMD 渲染器适配器 */
export class MMDRendererAdapter extends BaseModelRenderer {
  // 场景组件
  private sceneComponents: SceneComponents | null = null;
  private sceneDisposer: SceneDisposer | null = null;
  
  // MMD 特有组件
  private mesh: THREE.SkinnedMesh | null = null;
  private helper: MMDAnimationHelper | null = null;
  private animationId: number | null = null;
  private physicsEnabled = true;
  
  // 动作管理
  private loadedMotions: Map<string, LoadedMotion> = new Map();
  private currentMotionName: string | null = null;
  
  // 服装部件缓存
  private outfitParts: Map<string, OutfitPart> = new Map();
  
  // 口型同步适配器
  private lipSyncAdapter: LipSyncTarget | null = null;

  readonly modelType: ModelType = "mmd";

  // ─────────────────────────────────────────────────────────────────────────
  // 表情控制器 (MMD Morph)
  // ─────────────────────────────────────────────────────────────────────────
  readonly expression: ExpressionController = {
    setExpression: (name: string, weight = 1) => {
      if (!this.mesh?.morphTargetDictionary || !this.mesh.morphTargetInfluences) return;

      const index = this.mesh.morphTargetDictionary[name];
      if (index !== undefined) {
        // 重置所有表情类 morph
        const expressions = this.expression.getAvailableExpressions();
        expressions.forEach((expr) => {
          const idx = this.mesh!.morphTargetDictionary![expr];
          if (idx !== undefined && this.mesh!.morphTargetInfluences) {
            this.mesh!.morphTargetInfluences[idx] = 0;
          }
        });

        this.mesh.morphTargetInfluences[index] = weight;
        this.emit("expressionChanged", { expression: name, weight });
      }
    },

    getAvailableExpressions: (): string[] => {
      if (!this.mesh?.morphTargetDictionary) return [];
      return Object.keys(this.mesh.morphTargetDictionary);
    },

    resetExpression: () => {
      if (!this.mesh?.morphTargetInfluences) return;

      for (let i = 0; i < this.mesh.morphTargetInfluences.length; i++) {
        this.mesh.morphTargetInfluences[i] = 0;
      }
    },

    setFromEmotion: (emotion: Emotion) => {
      const available = this.expression.getAvailableExpressions();
      
      // MMD 表情映射（日文常见名称）
      const mmdEmotionMap: Record<Emotion, string[]> = {
        happy: ["笑い", "にっこり", "smile", "happy", "joy", "にこり"],
        sad: ["悲しい", "泣き", "sad", "cry", "悲しむ"],
        angry: ["怒り", "angry", "mad", "怒る"],
        surprised: ["驚き", "びっくり", "surprised", "驚く"],
        neutral: ["真面目", "normal", "default", "通常"],
        thinking: ["考え", "thinking", "困る"],
      };

      const candidates = mmdEmotionMap[emotion] || [];
      for (const candidate of candidates) {
        const found = available.find(
          (a) => a.toLowerCase().includes(candidate.toLowerCase()) || a.includes(candidate)
        );
        if (found) {
          this.expression.setExpression(found);
          return;
        }
      }

      // 回退到通用匹配
      const matched = findExpressionForEmotion(emotion, available);
      if (matched) {
        this.expression.setExpression(matched);
      }
    },
  };

  // ─────────────────────────────────────────────────────────────────────────
  // 动作控制器
  // ─────────────────────────────────────────────────────────────────────────
  readonly motion: MotionController = {
    playMotion: async (group: string) => {
      const motion = this.loadedMotions.get(group);
      if (!motion || !this.mesh || !this.helper) {
        console.warn(`[MMDRenderer] Motion "${group}" not found`);
        return;
      }
      
      // 重新添加动画
      this.helper.add(this.mesh, {
        animation: motion.clip,
        physics: this.physicsEnabled,
      });
      
      this.currentMotionName = group;
      this.emit("motionStarted", { group });
    },

    stopMotion: () => {
      if (this.mesh && this.helper) {
        // 移除当前模型的动画，保留物理
        this.helper.remove(this.mesh);
        this.helper.add(this.mesh, {
          physics: this.physicsEnabled,
        });
        this.currentMotionName = null;
        this.emit("motionEnded", {});
      }
    },

    getAvailableMotions: (): MotionGroup[] => {
      return Array.from(this.loadedMotions.keys()).map((name, index) => ({
        name,
        motions: [{ index, name }],
      }));
    },

    playIdleMotion: () => {
      // 尝试找到待机动作
      const idleMotion = Array.from(this.loadedMotions.keys()).find(
        name => /idle|待機|stand/i.test(name)
      );
      if (idleMotion) {
        this.motion.playMotion(idleMotion);
      }
    },
    
    /** 加载外部 VMD 动作文件 */
    loadAnimation: async (animationPath: string): Promise<void> => {
      if (!this.mesh || !this.helper) {
        throw new Error("Model not loaded");
      }

      const vmdLoader = new VMDLoader();
      const assetUrl = toAssetUrl(animationPath);

      const vmd = await vmdLoader.loadAsync(assetUrl);
      const clip = createMMDAnimationClip(vmd, this.mesh);

      const motionName = animationPath.split("/").pop()?.replace(/\.vmd$/i, "") || 
                        animationPath.split("\\").pop()?.replace(/\.vmd$/i, "") || 
                        `motion_${this.loadedMotions.size}`;
      
      this.loadedMotions.set(motionName, {
        name: motionName,
        clip,
      });
      
      console.log(`[MMDRenderer] Loaded animation: ${motionName}`);
      
      // 更新元数据
      if (this._metadata) {
        this._metadata.motions = this.motion.getAvailableMotions();
      }
    },
    
    /** 卸载指定动画 */
    unloadAnimation: (name: string): void => {
      if (this.currentMotionName === name) {
        this.motion.stopMotion();
      }
      this.loadedMotions.delete(name);
      console.log(`[MMDRenderer] Unloaded animation: ${name}`);
    },
    
    /** 卸载所有动画 */
    unloadAllAnimations: (): void => {
      this.motion.stopMotion();
      this.loadedMotions.clear();
      console.log("[MMDRenderer] Unloaded all animations");
    },
  };

  // ─────────────────────────────────────────────────────────────────────────
  // 可选控制器
  // ─────────────────────────────────────────────────────────────────────────
  lookAt: LookAtController = {
    lookAt: () => {},
    setAutoLookAt: () => {},
    resetLookAt: () => {},
  };

  physics: PhysicsController = {
    setEnabled: (enabled: boolean) => {
      this.physicsEnabled = enabled;
      if (this.helper) {
        this.helper.enabled.physics = enabled;
      }
    },
    isEnabled: () => this.physicsEnabled,
  };
  
  // ─────────────────────────────────────────────────────────────────────────
  // 服装控制器
  // ─────────────────────────────────────────────────────────────────────────
  outfit: OutfitController = {
    getAvailableParts: (): OutfitPart[] => {
      return Array.from(this.outfitParts.values());
    },
    
    setPartVisibility: (partName: string, visible: boolean): void => {
      const part = this.outfitParts.get(partName);
      if (!part || !this.mesh) return;
      
      // MMD 模型通常不支持按 mesh 名称切换，但我们可以尝试通过材质组来控制
      // 这是一个有限的实现，因为 MMD 模型结构与其他格式不同
      console.warn(`[MMDRenderer] Outfit visibility control is limited for MMD models`);
      part.visible = visible;
    },
    
    showAllParts: (): void => {
      this.outfitParts.forEach((part) => {
        part.visible = true;
      });
    },
    
    hideAllParts: (): void => {
      this.outfitParts.forEach((part) => {
        part.visible = false;
      });
    },
  };

  // ─────────────────────────────────────────────────────────────────────────
  // 便捷访问器
  // ─────────────────────────────────────────────────────────────────────────
  private get scene(): THREE.Scene | null {
    return this.sceneComponents?.scene ?? null;
  }
  
  private get camera(): THREE.PerspectiveCamera | null {
    return this.sceneComponents?.camera ?? null;
  }
  
  private get renderer(): THREE.WebGLRenderer | null {
    return this.sceneComponents?.renderer ?? null;
  }
  
  private get controls(): OrbitControls | null {
    return this.sceneComponents?.controls ?? null;
  }
  
  private get clock(): THREE.Clock | null {
    return this.sceneComponents?.clock ?? null;
  }

  // ─────────────────────────────────────────────────────────────────────────
  // 生命周期
  // ─────────────────────────────────────────────────────────────────────────
  async load(modelPath: string): Promise<void> {
    try {
      // 如果已有模型，先卸载
      if (this.mesh) {
        this.unloadModel();
      }
      
      // 初始化 Ammo.js 物理引擎
      await ensureAmmoInit();

      // 初始化场景（如果需要）
      if (!this.sceneComponents) {
        const { components, dispose } = initializeScene(this.container, MMD_SCENE_CONFIG);
        this.sceneComponents = components;
        this.sceneDisposer = dispose;
      }

      // 创建 MMDAnimationHelper
      this.helper = new MMDAnimationHelper({ afterglow: 2.0 });

      // 使用共享的 LoadingManager
      const modelDir = getModelDirectory(modelPath);
      const manager = createTauriLoadingManager(modelDir);

      // 加载 MMD 模型
      const loader = new MMDLoader(manager);
      const assetUrl = toAssetUrl(modelPath);

      this.mesh = await loader.loadAsync(assetUrl);
      this.scene!.add(this.mesh);

      // 添加到 helper（启用物理）
      this.helper.add(this.mesh, {
        physics: this.physicsEnabled,
      });

      // 自动调整相机
      this.fitModelToView();
      
      // 扫描服装部件（有限支持）
      this.scanOutfitParts();
      
      // 设置口型同步适配器
      this.setupLipSync();

      // 设置元数据
      this.setMetadata({
        type: "mmd",
        path: modelPath,
        name: this.getModelName(modelPath),
        expressions: this.expression.getAvailableExpressions(),
        motions: this.motion.getAvailableMotions(),
      });

      // 启动渲染循环
      this.startRenderLoop();

      this.setLoaded(true);
    } catch (error) {
      this.emit("error", error);
      throw error;
    }
  }
  
  /** 卸载模型但保留渲染器 */
  unloadModel(): void {
    // 停止口型同步
    lipSyncController.setTarget(null);
    this.lipSyncAdapter = null;
    
    // 停止动画
    this.motion.stopMotion();
    
    // 清理动画
    this.loadedMotions.clear();
    this.currentMotionName = null;
    
    // 清理服装
    this.outfitParts.clear();
    
    // 移除模型
    if (this.mesh && this.scene) {
      // 从 helper 中移除
      if (this.helper) {
        this.helper.remove(this.mesh);
      }
      
      this.scene.remove(this.mesh);
      
      // 释放资源
      this.mesh.geometry.dispose();
      if (Array.isArray(this.mesh.material)) {
        this.mesh.material.forEach(m => m.dispose());
      } else {
        this.mesh.material.dispose();
      }
      
      this.mesh = null;
    }
    
    // 销毁 helper（会在下次 load 时重建）
    this.helper = null;
    
    this._isLoaded = false;
    this._metadata = null;
    
    console.log("[MMDRenderer] Model unloaded");
  }
  
  /** 扫描模型的材质组作为"服装部件"（有限支持） */
  private scanOutfitParts(): void {
    if (!this.mesh) return;
    
    this.outfitParts.clear();
    
    // MMD 模型的 mesh 通常是单个 SkinnedMesh，可以通过材质来区分部件
    const materials = Array.isArray(this.mesh.material) 
      ? this.mesh.material 
      : [this.mesh.material];
    
    materials.forEach((material, index) => {
      const name = material.name || `material_${index}`;
      this.outfitParts.set(name, {
        name,
        visible: true,
        meshNames: [name],
      });
    });
  }
  
  /** 设置口型同步适配器 */
  private setupLipSync(): void {
    if (!this.mesh) return;
    
    // 创建 MMD 口型适配器
    this.lipSyncAdapter = createMMDLipSyncAdapter(this.mesh);
    
    // 注册到全局口型控制器
    lipSyncController.setTarget(this.lipSyncAdapter);
    
    console.log("[MMDRenderer] Lip sync enabled");
  }

  private fitModelToView(): void {
    if (!this.mesh || !this.camera || !this.controls) return;
    
    fitCameraToObject(this.mesh, this.camera, this.controls, {
      padding: 1.2,
      verticalOffset: 0,
    });
  }

  private getModelName(path: string): string {
    const fileName = path.split("/").pop() || path.split("\\").pop() || "MMD Model";
    return fileName.replace(/\.(pmd|pmx)$/i, "");
  }

  private startRenderLoop(): void {
    // 避免重复启动
    if (this.animationId !== null) return;
    
    const animate = () => {
      this.animationId = requestAnimationFrame(animate);
      const delta = this.clock?.getDelta() ?? 0;

      this.controls?.update();

      // 更新 MMD helper（物理、动画）
      if (this.helper) {
        this.helper.update(delta);
      }

      if (this.renderer && this.scene && this.camera) {
        this.renderer.render(this.scene, this.camera);
      }
    };
    animate();
  }

  dispose(): void {
    // 先卸载模型
    this.unloadModel();
    
    // 停止渲染循环
    if (this.animationId !== null) {
      cancelAnimationFrame(this.animationId);
      this.animationId = null;
    }

    this.sceneDisposer?.();
    this.sceneComponents = null;
    this.sceneDisposer = null;

    this.emit("disposed");
  }

  resize(_width: number, _height: number): void {
    if (this.camera && this.renderer) {
      updateSize(this.container, this.camera, this.renderer);
    }
  }

  setScale(scale: number): void {
    this.mesh?.scale.setScalar(scale);
  }

  setPosition(x: number, y: number): void {
    if (this.mesh) {
      this.mesh.position.x = x;
      this.mesh.position.y = y;
    }
  }

  resetView(): void {
    this.fitModelToView();
  }
}

/** MMD 渲染器工厂 */
export class MMDRendererFactory {
  canHandle(modelPath: string): boolean {
    const lower = modelPath.toLowerCase();
    return lower.endsWith(".pmd") || lower.endsWith(".pmx");
  }

  create(container: HTMLElement): MMDRendererAdapter {
    return new MMDRendererAdapter(container);
  }
}
