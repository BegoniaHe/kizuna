// FBX Renderer Adapter
// FBX 模型渲染器 - 增强版

import * as THREE from "three";
import { FBXLoader } from "three/addons/loaders/FBXLoader.js";
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
import { FBX_SCENE_CONFIG } from "./shared/CameraPresets";
import { 
  lipSyncController, 
  createVRMLipSyncAdapter,
  type LipSyncTarget 
} from "@/services/LipSyncService";

/** FBX 渲染器适配器 */
export class FBXRendererAdapter extends BaseModelRenderer {
  // 场景组件
  private sceneComponents: SceneComponents | null = null;
  private sceneDisposer: SceneDisposer | null = null;
  
  // 模型相关
  private model: THREE.Group | null = null;
  private mixer: THREE.AnimationMixer | null = null;
  private animations: Map<string, THREE.AnimationAction> = new Map();
  private currentAction: THREE.AnimationAction | null = null;
  private animationId: number | null = null;
  private morphTargets: Map<string, { mesh: THREE.SkinnedMesh; index: number }> = new Map();
  
  // 服装部件缓存
  private outfitParts: Map<string, OutfitPart> = new Map();
  // 外部加载的配件
  private loadedOutfits: Map<string, THREE.Group> = new Map();
  
  // 口型同步适配器
  private lipSyncAdapter: LipSyncTarget | null = null;

  readonly modelType: ModelType = "fbx";

  // ─────────────────────────────────────────────────────────────────────────
  // 表情控制器
  // ─────────────────────────────────────────────────────────────────────────
  readonly expression: ExpressionController = {
    setExpression: (name: string, weight = 1) => {
      const target = this.morphTargets.get(name.toLowerCase());
      if (target && target.mesh.morphTargetInfluences) {
        this.morphTargets.forEach((t) => {
          if (t.mesh.morphTargetInfluences) {
            t.mesh.morphTargetInfluences[t.index] = 0;
          }
        });
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
        const parts = name.split("|");
        const groupName = parts.length > 1 ? parts[0] : "default";
        const motionName = parts.length > 1 ? parts[1] : name;

        if (!groups.has(groupName)) {
          groups.set(groupName, { name: groupName, motions: [] });
        }

        const group = groups.get(groupName)!;
        group.motions.push({ index: group.motions.length, name: motionName });
      });

      return Array.from(groups.values());
    },

    playIdleMotion: () => {
      const idleAction =
        this.animations.get("idle") ||
        this.animations.get("Idle") ||
        this.animations.get("Take 001") ||
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
      
      const modelDir = getModelDirectory(animationPath);
      const manager = createTauriLoadingManager(modelDir);
      const assetUrl = toAssetUrl(animationPath);
      const ext = animationPath.toLowerCase().split('.').pop();
      
      let clips: THREE.AnimationClip[] = [];
      
      if (ext === 'fbx') {
        const loader = new FBXLoader(manager);
        const fbx = await loader.loadAsync(assetUrl);
        clips = fbx.animations;
      } else if (ext === 'gltf' || ext === 'glb') {
        const { GLTFLoader } = await import("three/addons/loaders/GLTFLoader.js");
        const loader = new GLTFLoader(manager);
        const gltf = await loader.loadAsync(assetUrl);
        clips = gltf.animations;
      }
      
      if (clips.length === 0) {
        console.warn(`[FBXRenderer] No animations found in ${animationPath}`);
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
        console.log(`[FBXRenderer] Loaded animation: ${name}`);
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
        console.log(`[FBXRenderer] Unloaded animation: ${name}`);
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
      console.log("[FBXRenderer] Unloaded all animations");
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
      const loader = new FBXLoader(manager);
      const assetUrl = toAssetUrl(outfitPath);
      
      const fbx = await loader.loadAsync(assetUrl);
      
      const outfitName = outfitPath.split('/').pop()?.replace(/\.[^.]+$/, '') || 'outfit';
      const meshNames: string[] = [];
      
      fbx.traverse((child) => {
        if (child instanceof THREE.Mesh || child instanceof THREE.SkinnedMesh) {
          const meshName = child.name || `${outfitName}_mesh_${meshNames.length}`;
          child.name = meshName;
          meshNames.push(meshName);
        }
      });
      
      // 添加到模型
      this.model.add(fbx);
      this.loadedOutfits.set(outfitName, fbx);
      
      // 注册服装部件
      this.outfitParts.set(outfitName, {
        name: outfitName,
        visible: true,
        meshNames,
      });
      
      console.log(`[FBXRenderer] Loaded outfit: ${outfitName}`);
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
        console.log(`[FBXRenderer] Unloaded outfit: ${outfitName}`);
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
      
      // 初始化场景
      if (!this.sceneComponents) {
        const { components, dispose } = initializeScene(this.container, FBX_SCENE_CONFIG);
        this.sceneComponents = components;
        this.sceneDisposer = dispose;
      }

      // 使用共享的 LoadingManager
      const modelDir = getModelDirectory(modelPath);
      const manager = createTauriLoadingManager(modelDir);

      // 设置默认 LoadingManager，确保所有内部加载器都使用它
      THREE.DefaultLoadingManager.setURLModifier((url: string) => {
        // 跳过已处理的 URL
        if (url.startsWith("asset://") && url.includes("%2F")) {
          return url;
        }
        if (url.startsWith("data:") || url.startsWith("blob:") || 
            url.startsWith("http://") || url.startsWith("https://")) {
          return url;
        }
        
        // 处理仅文件名的 asset URL
        if (url.startsWith("asset://localhost/") && !url.includes("%2F")) {
          const filename = url.replace("asset://localhost/", "");
          const absolutePath = modelDir + decodeURIComponent(filename);
          const converted = toAssetUrl(absolutePath);
          console.log(`[FBXRenderer] DefaultManager fixing: ${url} -> ${converted}`);
          return converted;
        }
        
        // 处理本地路径
        let absolutePath: string;
        if (url.startsWith("/") || url.match(/^[A-Za-z]:/)) {
          absolutePath = url;
        } else {
          absolutePath = modelDir + url;
        }
        const converted = toAssetUrl(absolutePath);
        console.log(`[FBXRenderer] DefaultManager fixing: ${url} -> ${converted}`);
        return converted;
      });

      // 加载 FBX
      const loader = new FBXLoader(manager);
      const assetUrl = toAssetUrl(modelPath);

      const fbx = await loader.loadAsync(assetUrl);
      this.model = fbx;

      // 启用阴影并修复纹理路径
      const textureLoader = new THREE.TextureLoader(manager);
      fbx.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          child.castShadow = true;
          child.receiveShadow = true;
          
          // 修复材质纹理路径
          const materials = Array.isArray(child.material) ? child.material : [child.material];
          materials.forEach((material) => {
            if (material instanceof THREE.MeshStandardMaterial || 
                material instanceof THREE.MeshPhongMaterial ||
                material instanceof THREE.MeshBasicMaterial) {
              this.fixMaterialTextures(material, modelDir, textureLoader);
            }
          });
        }
      });

      // 检查是否已被销毁
      if (!this.sceneComponents) {
        console.warn('[FBXRenderer] Scene disposed during loading, skipping...');
        return;
      }

      this.scene!.add(fbx);
      
      // 旋转模型面向镜头 (与 VRM 保持一致)
      fbx.rotation.y = Math.PI;

      // 自动调整相机
      this.fitModelToView();

      // 提取动画
      if (fbx.animations.length > 0) {
        this.mixer = new THREE.AnimationMixer(fbx);
        fbx.animations.forEach((clip, index) => {
          const action = this.mixer!.clipAction(clip);
          this.animations.set(clip.name || `animation_${index}`, action);
        });

        // 自动播放第一个动画
        const firstAction = Array.from(this.animations.values())[0];
        if (firstAction) {
          firstAction.play();
          this.currentAction = firstAction;
        }
      }

      // 提取 MorphTargets
      this.extractMorphTargets();
      
      // 扫描服装部件
      this.scanOutfitParts();
      
      // 设置口型同步
      this.setupLipSync();

      // 设置元数据
      this.setMetadata({
        type: "fbx",
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
    
    // 清理 DefaultLoadingManager 的 URLModifier
    THREE.DefaultLoadingManager.setURLModifier((url) => url);
    
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
    
    console.log("[FBXRenderer] Model unloaded");
  }

  private fitModelToView(): void {
    if (!this.model || !this.camera || !this.controls) return;
    
    // 计算模型边界
    const box = new THREE.Box3().setFromObject(this.model);
    const size = box.getSize(new THREE.Vector3());
    const center = box.getCenter(new THREE.Vector3());
    
    console.log(`[FBXRenderer] Model size: ${size.x.toFixed(2)}, ${size.y.toFixed(2)}, ${size.z.toFixed(2)}`);
    console.log(`[FBXRenderer] Model center: ${center.x.toFixed(2)}, ${center.y.toFixed(2)}, ${center.z.toFixed(2)}`);
    
    // 根据模型大小动态调整 padding
    const maxDim = Math.max(size.x, size.y, size.z);
    // 使用较小的 padding 让模型更大
    const padding = maxDim < 10 ? 1.2 : 1.5;
    
    fitCameraToObject(this.model, this.camera, this.controls, {
      padding,
      verticalOffset: 0,
    });
    
    console.log(`[FBXRenderer] Camera position: ${this.camera.position.x.toFixed(2)}, ${this.camera.position.y.toFixed(2)}, ${this.camera.position.z.toFixed(2)}`);
  }

  private extractMorphTargets(): void {
    if (!this.model) return;

    this.model.traverse((child) => {
      if (child instanceof THREE.SkinnedMesh && child.morphTargetDictionary) {
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
      console.log("[FBXRenderer] No mouth BlendShapes found, lip sync disabled");
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
    
    console.log("[FBXRenderer] Lip sync enabled");
  }

  private getModelName(path: string): string {
    const fileName = path.split("/").pop() || path.split("\\").pop() || "FBX Model";
    return fileName.replace(/\.fbx$/i, "");
  }

  /**
   * 修复材质中的纹理路径
   * FBX 可能存储绝对路径的纹理引用，需要转换为 Tauri asset URL
   */
  private fixMaterialTextures(
    material: THREE.MeshStandardMaterial | THREE.MeshPhongMaterial | THREE.MeshBasicMaterial,
    modelDir: string,
    textureLoader: THREE.TextureLoader
  ): void {
    const textureProps = ['map', 'normalMap', 'roughnessMap', 'metalnessMap', 'aoMap', 'emissiveMap', 'bumpMap', 'specularMap'] as const;
    
    for (const prop of textureProps) {
      const texture = (material as unknown as Record<string, THREE.Texture | null>)[prop];
      if (!texture) continue;
      
      // 获取纹理源 - 多种方式尝试
      let src: string | undefined;
      
      // 方式1: texture.source.data.src (HTMLImageElement)
      if (texture.source?.data?.src) {
        src = texture.source.data.src;
      }
      // 方式2: texture.image.src
      else if (texture.image?.src) {
        src = texture.image.src;
      }
      // 方式3: texture.userData.url
      else if (texture.userData?.url) {
        src = texture.userData.url;
      }
      
      if (!src) continue;
      
      console.log(`[FBXRenderer] Checking texture: ${prop} = ${src}`);
      
      // 如果是本地路径，重新加载
      if (src.startsWith('/') || src.match(/^[A-Za-z]:/) || !src.startsWith('asset://')) {
        const fileName = src.split('/').pop() || src.split('\\').pop() || '';
        const fixedPath = modelDir + fileName;
        
        console.log(`[FBXRenderer] Fixing texture: ${fileName} -> ${fixedPath}`);
        
        // 使用 LoadingManager 重新加载纹理
        const newTexture = textureLoader.load(fixedPath);
        newTexture.colorSpace = THREE.SRGBColorSpace;
        newTexture.flipY = texture.flipY;
        newTexture.wrapS = texture.wrapS;
        newTexture.wrapT = texture.wrapT;
        (material as unknown as Record<string, THREE.Texture>)[prop] = newTexture;
        material.needsUpdate = true;
      }
    }
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
    this.model?.scale.setScalar(scale);
  }

  setPosition(x: number, y: number): void {
    if (this.model) {
      this.model.position.x = x;
      this.model.position.y = y;
    }
  }

  resetView(): void {
    this.fitModelToView();
  }
}

/** FBX 渲染器工厂 */
export class FBXRendererFactory {
  canHandle(modelPath: string): boolean {
    return modelPath.toLowerCase().endsWith(".fbx");
  }

  create(container: HTMLElement): FBXRendererAdapter {
    return new FBXRendererAdapter(container);
  }
}
