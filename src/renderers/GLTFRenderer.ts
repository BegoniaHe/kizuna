// GLTF/GLB Renderer Adapter
// 通用 glTF/GLB 模型渲染器

import * as THREE from "three";
import { GLTFLoader, type GLTF } from "three/addons/loaders/GLTFLoader.js";
import type { OrbitControls } from "three/addons/controls/OrbitControls.js";
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
import { GLTF_SCENE_CONFIG } from "./shared/CameraPresets";
import { 
  lipSyncController, 
  createVRMLipSyncAdapter,
  type LipSyncTarget 
} from "@/services/LipSyncService";

/** GLTF 渲染器适配器 */
export class GLTFRendererAdapter extends BaseModelRenderer {
  // 场景组件 (通过共享模块管理)
  private sceneComponents: SceneComponents | null = null;
  private sceneDisposer: SceneDisposer | null = null;
  
  // 模型相关
  private model: THREE.Group | null = null;
  private mixer: THREE.AnimationMixer | null = null;
  private animations: Map<string, THREE.AnimationAction> = new Map();
  private currentAction: THREE.AnimationAction | null = null;
  private animationId: number | null = null;
  private morphTargets: Map<string, { mesh: THREE.Mesh; index: number }> = new Map();
  
  // 服装部件缓存
  private outfitParts: Map<string, OutfitPart> = new Map();
  // 外部加载的配件
  private loadedOutfits: Map<string, THREE.Group> = new Map();
  
  // 口型同步适配器
  private lipSyncAdapter: LipSyncTarget | null = null;

  readonly modelType: ModelType = "gltf";

  // ─────────────────────────────────────────────────────────────────────────
  // 表情控制器 (通过 MorphTargets/BlendShapes)
  // ─────────────────────────────────────────────────────────────────────────
  readonly expression: ExpressionController = {
    setExpression: (name: string, weight = 1) => {
      const target = this.morphTargets.get(name.toLowerCase());
      if (target && target.mesh.morphTargetInfluences) {
        // 重置所有
        this.morphTargets.forEach((t) => {
          if (t.mesh.morphTargetInfluences) {
            t.mesh.morphTargetInfluences[t.index] = 0;
          }
        });
        // 设置目标
        target.mesh.morphTargetInfluences[target.index] = weight;
        this.emit("expressionChanged", { expression: name, weight });
      }
    },

    getAvailableExpressions: (): string[] => {
      return Array.from(this.morphTargets.keys());
    },

    resetExpression: () => {
      this.morphTargets.forEach((target) => {
        if (target.mesh.morphTargetInfluences) {
          target.mesh.morphTargetInfluences[target.index] = 0;
        }
      });
    },

    setFromEmotion: (emotion: Emotion) => {
      const available = this.expression.getAvailableExpressions();
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
    playMotion: async (group: string, index?: number) => {
      const actionName = index !== undefined ? `${group}_${index}` : group;
      const action = this.animations.get(actionName) || this.animations.get(group);
      
      if (action) {
        // 淡出当前动作
        if (this.currentAction && this.currentAction !== action) {
          this.currentAction.fadeOut(0.5);
        }
        action.reset().fadeIn(0.5).play();
        this.currentAction = action;
        this.emit("motionStarted", { group, index });
      }
    },

    stopMotion: () => {
      if (this.currentAction) {
        this.currentAction.fadeOut(0.5);
        this.currentAction = null;
      }
      this.emit("motionEnded", {});
    },

    getAvailableMotions: (): MotionGroup[] => {
      const groups: Map<string, MotionGroup> = new Map();
      
      this.animations.forEach((_, name) => {
        const parts = name.split("_");
        const groupName = parts[0];
        
        if (!groups.has(groupName)) {
          groups.set(groupName, { name: groupName, motions: [] });
        }
        
        const group = groups.get(groupName)!;
        group.motions.push({ index: group.motions.length, name });
      });
      
      return Array.from(groups.values());
    },

    playIdleMotion: () => {
      const idleAction = 
        this.animations.get("idle") ||
        this.animations.get("Idle") ||
        Array.from(this.animations.values())[0];
      
      if (idleAction) {
        if (this.currentAction && this.currentAction !== idleAction) {
          this.currentAction.fadeOut(0.5);
        }
        idleAction.reset().fadeIn(0.5).play();
        this.currentAction = idleAction;
      }
    },
    
    /** 加载外部动画文件 */
    loadAnimation: async (animationPath: string): Promise<void> => {
      if (!this.model) {
        throw new Error("Model not loaded");
      }
      
      const assetUrl = toAssetUrl(animationPath);
      const ext = animationPath.toLowerCase().split('.').pop();
      
      let clips: THREE.AnimationClip[] = [];
      
      if (ext === 'gltf' || ext === 'glb') {
        const loader = new GLTFLoader();
        const gltf = await loader.loadAsync(assetUrl);
        clips = gltf.animations;
      } else if (ext === 'fbx') {
        const { FBXLoader } = await import("three/addons/loaders/FBXLoader.js");
        const loader = new FBXLoader();
        const fbx = await loader.loadAsync(assetUrl);
        clips = fbx.animations;
      }
      
      if (clips.length === 0) {
        console.warn(`[GLTFRenderer] No animations found in ${animationPath}`);
        return;
      }
      
      // 确保有 mixer
      if (!this.mixer) {
        this.mixer = new THREE.AnimationMixer(this.model);
      }
      
      const baseName = animationPath.split('/').pop()?.replace(/\.[^.]+$/, '') || 'animation';
      clips.forEach((clip, index) => {
        const name = clip.name || `${baseName}_${index}`;
        const action = this.mixer!.clipAction(clip);
        action.clampWhenFinished = true;
        this.animations.set(name, action);
        console.log(`[GLTFRenderer] Loaded animation: ${name}`);
      });
      
      // 更新元数据
      if (this._metadata) {
        this._metadata.motions = this.motion.getAvailableMotions();
      }
    },
    
    /** 卸载指定动画 */
    unloadAnimation: (name: string): void => {
      const action = this.animations.get(name);
      if (action) {
        action.stop();
        this.mixer?.uncacheAction(action.getClip());
        this.animations.delete(name);
        console.log(`[GLTFRenderer] Unloaded animation: ${name}`);
      }
    },
    
    /** 卸载所有动画 */
    unloadAllAnimations: (): void => {
      this.animations.forEach((action) => {
        action.stop();
        this.mixer?.uncacheAction(action.getClip());
      });
      this.animations.clear();
      this.currentAction = null;
      console.log("[GLTFRenderer] Unloaded all animations");
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
    setEnabled: () => {},
    isEnabled: () => false,
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
      if (!part || !this.model) return;
      
      part.meshNames.forEach(meshName => {
        this.model!.traverse((child) => {
          if (child.name === meshName) {
            child.visible = visible;
          }
        });
      });
      
      part.visible = visible;
    },
    
    showAllParts: (): void => {
      this.outfitParts.forEach((part) => {
        this.outfit.setPartVisibility(part.name, true);
      });
    },
    
    hideAllParts: (): void => {
      this.outfitParts.forEach((part) => {
        this.outfit.setPartVisibility(part.name, false);
      });
    },
    
    /** 加载外部配件模型 */
    loadOutfit: async (outfitPath: string): Promise<void> => {
      if (!this.model || !this.scene) {
        throw new Error("Model not loaded");
      }
      
      const modelDir = getModelDirectory(outfitPath);
      const manager = createTauriLoadingManager(modelDir);
      const loader = new GLTFLoader(manager);
      const assetUrl = toAssetUrl(outfitPath);
      
      const gltf = await loader.loadAsync(assetUrl);
      const outfitScene = gltf.scene;
      
      const outfitName = outfitPath.split('/').pop()?.replace(/\.[^.]+$/, '') || 'outfit';
      const meshNames: string[] = [];
      
      outfitScene.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          const meshName = child.name || `${outfitName}_mesh_${meshNames.length}`;
          child.name = meshName;
          meshNames.push(meshName);
        }
      });
      
      // 添加到模型
      this.model.add(outfitScene);
      this.loadedOutfits.set(outfitName, outfitScene);
      
      // 注册服装部件
      this.outfitParts.set(outfitName, {
        name: outfitName,
        visible: true,
        meshNames,
      });
      
      console.log(`[GLTFRenderer] Loaded outfit: ${outfitName}`);
    },
    
    /** 卸载配件 */
    unloadOutfit: (outfitName: string): void => {
      const outfitScene = this.loadedOutfits.get(outfitName);
      if (outfitScene && this.model) {
        this.model.remove(outfitScene);
        
        // 释放资源
        outfitScene.traverse((child) => {
          if (child instanceof THREE.Mesh) {
            child.geometry.dispose();
            if (Array.isArray(child.material)) {
              child.material.forEach(m => m.dispose());
            } else {
              child.material.dispose();
            }
          }
        });
        
        this.loadedOutfits.delete(outfitName);
        this.outfitParts.delete(outfitName);
        console.log(`[GLTFRenderer] Unloaded outfit: ${outfitName}`);
      }
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
      if (this.model) {
        this.unloadModel();
      }
      
      // 初始化场景（如果需要）
      if (!this.sceneComponents) {
        const { components, dispose } = initializeScene(this.container, GLTF_SCENE_CONFIG);
        this.sceneComponents = components;
        this.sceneDisposer = dispose;
      }

      // 使用共享的 LoadingManager
      const modelDir = getModelDirectory(modelPath);
      const manager = createTauriLoadingManager(modelDir);

      // 加载 GLTF
      const loader = new GLTFLoader(manager);
      const assetUrl = toAssetUrl(modelPath);

      const gltf: GLTF = await loader.loadAsync(assetUrl);
      this.model = gltf.scene;
      this.scene!.add(this.model);
      
      // 旋转模型面向镜头
      this.model.rotation.y = Math.PI;

      // 自动调整相机
      this.fitModelToView();

      // 提取动画
      if (gltf.animations.length > 0) {
        this.mixer = new THREE.AnimationMixer(this.model);
        gltf.animations.forEach((clip, index) => {
          const action = this.mixer!.clipAction(clip);
          this.animations.set(clip.name || `animation_${index}`, action);
        });
      }

      // 提取 MorphTargets
      this.extractMorphTargets();
      
      // 扫描服装部件
      this.scanOutfitParts();
      
      // 设置口型同步（如果模型有口型 BlendShape）
      this.setupLipSync();

      // 设置元数据
      this.setMetadata({
        type: "gltf",
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
    if (this.currentAction) {
      this.currentAction.stop();
      this.currentAction = null;
    }
    
    // 清理动画
    this.animations.forEach((action) => {
      action.stop();
      this.mixer?.uncacheAction(action.getClip());
    });
    this.animations.clear();
    this.mixer = null;
    
    // 清理 morph targets
    this.morphTargets.clear();
    
    // 清理服装
    this.loadedOutfits.clear();
    this.outfitParts.clear();
    
    // 移除模型
    if (this.model && this.scene) {
      this.scene.remove(this.model);
      
      // 释放资源
      this.model.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          child.geometry.dispose();
          if (Array.isArray(child.material)) {
            child.material.forEach(m => m.dispose());
          } else {
            child.material.dispose();
          }
        }
      });
      
      this.model = null;
    }
    
    this._isLoaded = false;
    this._metadata = null;
    
    console.log("[GLTFRenderer] Model unloaded");
  }

  private fitModelToView(): void {
    if (!this.model || !this.camera || !this.controls) return;
    
    // 使用共享工具调整相机
    fitCameraToObject(this.model, this.camera, this.controls, {
      padding: 1.5,
      verticalOffset: 0,
    });
    
    // 调整模型位置（居中）
    const box = new THREE.Box3().setFromObject(this.model);
    const center = box.getCenter(new THREE.Vector3());
    const size = box.getSize(new THREE.Vector3());
    this.model.position.sub(center);
    this.model.position.y += size.y / 2;
    
    // 重新调整相机目标
    this.controls.target.set(0, size.y / 2, 0);
    this.camera.position.set(0, size.y / 2, this.camera.position.z);
    this.controls.update();
  }

  private extractMorphTargets(): void {
    if (!this.model) return;

    this.model.traverse((child) => {
      if (child instanceof THREE.Mesh && child.morphTargetDictionary) {
        Object.entries(child.morphTargetDictionary).forEach(([name, index]) => {
          this.morphTargets.set(name.toLowerCase(), { mesh: child, index });
        });
      }
    });
  }
  
  /** 扫描模型中的服装部件 */
  private scanOutfitParts(): void {
    if (!this.model) return;
    
    this.outfitParts.clear();
    
    const partPatterns = [
      { pattern: /hair/i, category: 'hair' },
      { pattern: /head|face/i, category: 'head' },
      { pattern: /body|torso/i, category: 'body' },
      { pattern: /arm|hand/i, category: 'arms' },
      { pattern: /leg|foot/i, category: 'legs' },
      { pattern: /cloth|shirt|dress|coat|jacket|pants|skirt/i, category: 'clothing' },
      { pattern: /accessory|acc|glasses|hat|bag|jewelry/i, category: 'accessories' },
    ];
    
    const categorizedMeshes: Map<string, string[]> = new Map();
    
    this.model.traverse((child) => {
      if ((child instanceof THREE.Mesh || child instanceof THREE.SkinnedMesh) && child.name) {
        let matched = false;
        for (const { pattern, category } of partPatterns) {
          if (pattern.test(child.name)) {
            if (!categorizedMeshes.has(category)) {
              categorizedMeshes.set(category, []);
            }
            categorizedMeshes.get(category)!.push(child.name);
            matched = true;
            break;
          }
        }
        
        if (!matched) {
          if (!categorizedMeshes.has('other')) {
            categorizedMeshes.set('other', []);
          }
          categorizedMeshes.get('other')!.push(child.name);
        }
      }
    });
    
    categorizedMeshes.forEach((meshNames, category) => {
      this.outfitParts.set(category, {
        name: category,
        visible: true,
        meshNames,
      });
    });
  }
  
  /** 设置口型同步适配器 */
  private setupLipSync(): void {
    const availableExpressions = this.expression.getAvailableExpressions();
    
    // 检查是否有口型相关的 BlendShape
    const hasMouthShapes = availableExpressions.some(e => 
      /aa|ee|ih|oh|ou|mouth|viseme/i.test(e)
    );
    
    if (!hasMouthShapes) {
      console.log("[GLTFRenderer] No mouth BlendShapes found, lip sync disabled");
      return;
    }
    
    // 创建口型适配器
    this.lipSyncAdapter = createVRMLipSyncAdapter(
      (name, weight) => {
        const target = this.morphTargets.get(name.toLowerCase());
        if (target && target.mesh.morphTargetInfluences) {
          target.mesh.morphTargetInfluences[target.index] = weight;
        }
      },
      () => {
        this.expression.resetExpression();
      },
      availableExpressions
    );
    
    // 注册到全局口型控制器
    lipSyncController.setTarget(this.lipSyncAdapter);
    
    console.log("[GLTFRenderer] Lip sync enabled");
  }

  private getModelName(path: string): string {
    const fileName = path.split("/").pop() || path.split("\\").pop() || "GLTF Model";
    return fileName.replace(/\.(gltf|glb)$/i, "");
  }

  private startRenderLoop(): void {
    // 避免重复启动
    if (this.animationId !== null) return;
    
    const animate = () => {
      this.animationId = requestAnimationFrame(animate);
      const delta = this.clock?.getDelta() ?? 0;

      this.controls?.update();
      this.mixer?.update(delta);

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

    // 使用共享清理函数
    this.sceneDisposer?.();
    this.sceneComponents = null;
    this.sceneDisposer = null;

    this.emit("disposed");
  }

  // ─────────────────────────────────────────────────────────────────────────
  // 渲染控制
  // ─────────────────────────────────────────────────────────────────────────
  resize(_width: number, _height: number): void {
    if (this.camera && this.renderer) {
      updateSize(this.container, this.camera, this.renderer);
    }
  }

  setScale(scale: number): void {
    this.model?.scale.setScalar(scale);
  }

  setPosition(x: number, y: number): void {
    if (this.model) {
      this.model.position.x = x;
      this.model.position.y = y;
    }
  }

  resetView(): void {
    if (this.camera && this.controls) {
      this.camera.position.copy(GLTF_SCENE_CONFIG.camera.position);
      this.controls.target.copy(GLTF_SCENE_CONFIG.camera.target);
      this.controls.update();
    }
    this.fitModelToView();
  }
}

/** GLTF 渲染器工厂 */
export class GLTFRendererFactory {
  canHandle(modelPath: string): boolean {
    const lower = modelPath.toLowerCase();
    if (lower.endsWith(".vrm")) return false;
    return lower.endsWith(".gltf") || lower.endsWith(".glb");
  }

  create(container: HTMLElement): GLTFRendererAdapter {
    return new GLTFRendererAdapter(container);
  }
}
