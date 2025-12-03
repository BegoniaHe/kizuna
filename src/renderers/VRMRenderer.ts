// VRM Renderer Adapter
// 封装 @pixiv/three-vrm 提供统一的渲染接口

import * as THREE from "three";
import { GLTFLoader } from "three/addons/loaders/GLTFLoader.js";
import type { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { VRMLoaderPlugin, VRM } from "@pixiv/three-vrm";
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
  updateSize,
  type SceneComponents,
  type SceneDisposer,
} from "./shared/ThreeSceneSetup";
import { toAssetUrl, createTauriLoadingManager, getModelDirectory } from "./shared/TauriAssetManager";
import { VRM_SCENE_CONFIG } from "./shared/CameraPresets";
import { 
  lipSyncController, 
  createVRMLipSyncAdapter,
  type LipSyncTarget 
} from "@/services/LipSyncService";

/** VRM 渲染器适配器 */
export class VRMRendererAdapter extends BaseModelRenderer {
  // 场景组件
  private sceneComponents: SceneComponents | null = null;
  private sceneDisposer: SceneDisposer | null = null;
  
  // VRM 特有组件
  private vrm: VRM | null = null;
  private mixer: THREE.AnimationMixer | null = null;
  private animations: Map<string, THREE.AnimationAction> = new Map();
  private currentAction: THREE.AnimationAction | null = null;
  private animationId: number | null = null;
  private physicsEnabled = true;
  private lookAtTarget: THREE.Object3D | null = null;
  
  // 服装部件缓存
  private outfitParts: Map<string, OutfitPart> = new Map();
  
  // 口型同步适配器
  private lipSyncAdapter: LipSyncTarget | null = null;
  
  readonly modelType: ModelType = "vrm";
  
  // ─────────────────────────────────────────────────────────────────────────
  // 表情控制器
  // ─────────────────────────────────────────────────────────────────────────
  readonly expression: ExpressionController = {
    setExpression: (name: string, weight = 1) => {
      if (!this.vrm?.expressionManager) return;
      
      const expressionManager = this.vrm.expressionManager;
      Object.keys(expressionManager.expressionMap).forEach(key => {
        expressionManager.setValue(key, 0);
      });
      
      expressionManager.setValue(name, weight);
      this.emit("expressionChanged", { expression: name, weight });
    },
    
    getAvailableExpressions: (): string[] => {
      if (!this.vrm?.expressionManager) return [];
      return Object.keys(this.vrm.expressionManager.expressionMap);
    },
    
    resetExpression: () => {
      if (!this.vrm?.expressionManager) return;
      
      const expressionManager = this.vrm.expressionManager;
      Object.keys(expressionManager.expressionMap).forEach(key => {
        expressionManager.setValue(key, 0);
      });
      
      if (expressionManager.expressionMap["neutral"]) {
        expressionManager.setValue("neutral", 1);
      }
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
  // 动作控制器 (支持外部动画加载)
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
      } else {
        console.warn(`[VRMRenderer] Animation "${actionName}" not found. Load animations first.`);
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
        this.animations.get("待機") ||
        Array.from(this.animations.values())[0];
      
      if (idleAction) {
        if (this.currentAction && this.currentAction !== idleAction) {
          this.currentAction.fadeOut(0.5);
        }
        idleAction.reset().fadeIn(0.5).play();
        this.currentAction = idleAction;
      }
    },
    
    /** 加载外部动画文件 (支持 GLTF/GLB/FBX/BVH) */
    loadAnimation: async (animationPath: string): Promise<void> => {
      if (!this.vrm) {
        throw new Error("Model not loaded");
      }
      
      const assetUrl = toAssetUrl(animationPath);
      const ext = animationPath.toLowerCase().split('.').pop();
      
      let clips: THREE.AnimationClip[] = [];
      
      if (ext === 'gltf' || ext === 'glb' || ext === 'vrma') {
        // 加载 GLTF/GLB/VRMA 动画
        const loader = new GLTFLoader();
        const gltf = await loader.loadAsync(assetUrl);
        clips = gltf.animations;
      } else if (ext === 'fbx') {
        // 加载 FBX 动画
        const { FBXLoader } = await import("three/addons/loaders/FBXLoader.js");
        const loader = new FBXLoader();
        const fbx = await loader.loadAsync(assetUrl);
        clips = fbx.animations;
      } else if (ext === 'bvh') {
        // 加载 BVH 动画
        const { BVHLoader } = await import("three/addons/loaders/BVHLoader.js");
        const loader = new BVHLoader();
        const bvh = await loader.loadAsync(assetUrl);
        clips = [bvh.clip];
      }
      
      if (clips.length === 0) {
        console.warn(`[VRMRenderer] No animations found in ${animationPath}`);
        return;
      }
      
      // 确保有 mixer
      if (!this.mixer) {
        this.mixer = new THREE.AnimationMixer(this.vrm.scene);
      }
      
      // 添加动画到 mixer
      const baseName = animationPath.split('/').pop()?.replace(/\.[^.]+$/, '') || 'animation';
      clips.forEach((clip, index) => {
        const name = clip.name || `${baseName}_${index}`;
        
        // 重定向动画到 VRM 骨骼 (如果需要)
        const retargetedClip = this.retargetAnimation(clip);
        
        const action = this.mixer!.clipAction(retargetedClip);
        action.clampWhenFinished = true;
        this.animations.set(name, action);
        console.log(`[VRMRenderer] Loaded animation: ${name}`);
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
        console.log(`[VRMRenderer] Unloaded animation: ${name}`);
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
      console.log("[VRMRenderer] Unloaded all animations");
    },
  };
  
  // ─────────────────────────────────────────────────────────────────────────
  // 视线控制器
  // ─────────────────────────────────────────────────────────────────────────
  lookAt: LookAtController = {
    lookAt: (x: number, y: number) => {
      if (this.vrm?.lookAt) {
        if (!this.lookAtTarget) {
          this.lookAtTarget = new THREE.Object3D();
          this.scene?.add(this.lookAtTarget);
        }
        this.lookAtTarget.position.set(
          (x - 0.5) * 2,
          (0.5 - y) * 2 + 1,
          1
        );
        this.vrm.lookAt.target = this.lookAtTarget;
      }
    },
    
    setAutoLookAt: (enabled: boolean) => {
      if (this.vrm?.lookAt) {
        this.vrm.lookAt.autoUpdate = enabled;
      }
    },
    
    resetLookAt: () => {
      if (this.vrm?.lookAt && this.lookAtTarget) {
        this.lookAtTarget.position.set(0, 1, 1);
        this.vrm.lookAt.target = this.lookAtTarget;
      }
    },
  };
  
  // ─────────────────────────────────────────────────────────────────────────
  // 物理控制器
  // ─────────────────────────────────────────────────────────────────────────
  physics: PhysicsController = {
    setEnabled: (enabled: boolean) => {
      this.physicsEnabled = enabled;
    },
    isEnabled: (): boolean => this.physicsEnabled,
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
      if (!part || !this.vrm) return;
      
      part.meshNames.forEach(meshName => {
        this.vrm!.scene.traverse((child) => {
          if (child.name === meshName && child instanceof THREE.Mesh) {
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
      if (!this.vrm || !this.scene) {
        throw new Error("Model not loaded");
      }
      
      const assetUrl = toAssetUrl(outfitPath);
      const loader = new GLTFLoader();
      const gltf = await loader.loadAsync(assetUrl);
      
      const outfitName = outfitPath.split('/').pop()?.replace(/\.[^.]+$/, '') || 'outfit';
      const meshNames: string[] = [];
      
      // 遍历配件模型的所有 mesh
      gltf.scene.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          meshNames.push(child.name || `${outfitName}_mesh_${meshNames.length}`);
        }
      });
      
      // 尝试绑定到 VRM 的骨骼系统
      // 注意：这需要配件模型有匹配的骨骼结构
      this.vrm.scene.add(gltf.scene);
      
      // 注册服装部件
      this.outfitParts.set(outfitName, {
        name: outfitName,
        visible: true,
        meshNames,
      });
      
      console.log(`[VRMRenderer] Loaded outfit: ${outfitName}`);
    },
    
    /** 卸载配件 */
    unloadOutfit: (outfitName: string): void => {
      const part = this.outfitParts.get(outfitName);
      if (!part || !this.vrm) return;
      
      // 移除相关 mesh
      const toRemove: THREE.Object3D[] = [];
      this.vrm.scene.traverse((child) => {
        if (part.meshNames.includes(child.name)) {
          toRemove.push(child);
        }
      });
      
      toRemove.forEach(obj => {
        obj.parent?.remove(obj);
        if (obj instanceof THREE.Mesh) {
          obj.geometry.dispose();
          if (Array.isArray(obj.material)) {
            obj.material.forEach(m => m.dispose());
          } else {
            obj.material.dispose();
          }
        }
      });
      
      this.outfitParts.delete(outfitName);
      console.log(`[VRMRenderer] Unloaded outfit: ${outfitName}`);
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
  // 动画重定向
  // ─────────────────────────────────────────────────────────────────────────
  private retargetAnimation(clip: THREE.AnimationClip): THREE.AnimationClip {
    // VRM 使用标准骨骼命名，这里做简单的名称映射
    // 对于 Mixamo 动画，需要将骨骼名称映射到 VRM 标准
    const mixamoToVRM: Record<string, string> = {
      'mixamorigHips': 'hips',
      'mixamorigSpine': 'spine',
      'mixamorigSpine1': 'chest',
      'mixamorigSpine2': 'upperChest',
      'mixamorigNeck': 'neck',
      'mixamorigHead': 'head',
      'mixamorigLeftShoulder': 'leftShoulder',
      'mixamorigLeftArm': 'leftUpperArm',
      'mixamorigLeftForeArm': 'leftLowerArm',
      'mixamorigLeftHand': 'leftHand',
      'mixamorigRightShoulder': 'rightShoulder',
      'mixamorigRightArm': 'rightUpperArm',
      'mixamorigRightForeArm': 'rightLowerArm',
      'mixamorigRightHand': 'rightHand',
      'mixamorigLeftUpLeg': 'leftUpperLeg',
      'mixamorigLeftLeg': 'leftLowerLeg',
      'mixamorigLeftFoot': 'leftFoot',
      'mixamorigLeftToeBase': 'leftToes',
      'mixamorigRightUpLeg': 'rightUpperLeg',
      'mixamorigRightLeg': 'rightLowerLeg',
      'mixamorigRightFoot': 'rightFoot',
      'mixamorigRightToeBase': 'rightToes',
    };
    
    const tracks = clip.tracks.map(track => {
      // 解析轨道名称 (格式: boneName.property)
      const parts = track.name.split('.');
      const boneName = parts[0];
      const property = parts.slice(1).join('.');
      
      // 尝试映射骨骼名称
      const vrmBoneName = mixamoToVRM[boneName] || boneName;
      const newName = `${vrmBoneName}.${property}`;
      
      // 创建新轨道
      const TrackConstructor = track.constructor as new (
        name: string,
        times: Float32Array,
        values: Float32Array
      ) => THREE.KeyframeTrack;
      
      return new TrackConstructor(
        newName,
        track.times as Float32Array,
        track.values as Float32Array
      );
    });
    
    return new THREE.AnimationClip(clip.name, clip.duration, tracks);
  }
  
  // ─────────────────────────────────────────────────────────────────────────
  // 生命周期
  // ─────────────────────────────────────────────────────────────────────────
  async load(modelPath: string): Promise<void> {
    try {
      // 如果已有模型，先卸载
      if (this.vrm) {
        this.unloadModel();
      }
      
      // 初始化场景（如果需要）
      if (!this.sceneComponents) {
        const { components, dispose } = initializeScene(this.container, VRM_SCENE_CONFIG);
        this.sceneComponents = components;
        this.sceneDisposer = dispose;
      }
      
      // 获取模型目录用于解析纹理路径
      const modelDir = getModelDirectory(modelPath);
      
      // 使用 Tauri LoadingManager 来正确处理纹理路径
      const loadingManager = createTauriLoadingManager(modelDir);
      
      // 加载 VRM
      const loader = new GLTFLoader(loadingManager);
      loader.register((parser) => new VRMLoaderPlugin(parser));
      
      const assetUrl = toAssetUrl(modelPath);
      const gltf = await loader.loadAsync(assetUrl);
      this.vrm = gltf.userData.vrm as VRM;
      
      this.scene!.add(this.vrm.scene);
      this.vrm.scene.rotation.y = Math.PI;
      
      // 创建动画混合器
      this.mixer = new THREE.AnimationMixer(this.vrm.scene);
      
      // 提取模型嵌入的动画
      if (gltf.animations.length > 0) {
        gltf.animations.forEach((clip, index) => {
          const action = this.mixer!.clipAction(clip);
          this.animations.set(clip.name || `embedded_${index}`, action);
        });
      }
      
      // 扫描并缓存服装部件
      this.scanOutfitParts();
      
      // 设置口型同步
      this.setupLipSync();
      
      // 设置元数据
      this.setMetadata({
        type: "vrm",
        path: modelPath,
        name: this.getVRMName(),
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
    
    // 清理服装缓存
    this.outfitParts.clear();
    
    // 移除 VRM 模型
    if (this.vrm && this.scene) {
      this.scene.remove(this.vrm.scene);
      
      // 释放 VRM 资源
      this.vrm.scene.traverse((child) => {
        if (child instanceof THREE.Mesh) {
          child.geometry.dispose();
          if (Array.isArray(child.material)) {
            child.material.forEach(m => m.dispose());
          } else {
            child.material.dispose();
          }
        }
      });
      
      this.vrm = null;
    }
    
    // 移除视线目标
    if (this.lookAtTarget && this.scene) {
      this.scene.remove(this.lookAtTarget);
      this.lookAtTarget = null;
    }
    
    this._isLoaded = false;
    this._metadata = null;
    
    console.log("[VRMRenderer] Model unloaded");
  }
  
  /** 扫描模型中的服装部件 */
  private scanOutfitParts(): void {
    if (!this.vrm) return;
    
    this.outfitParts.clear();
    
    // 常见的服装部件名称模式
    const partPatterns = [
      { pattern: /hair/i, category: 'hair' },
      { pattern: /head|face/i, category: 'head' },
      { pattern: /body|torso/i, category: 'body' },
      { pattern: /arm|hand/i, category: 'arms' },
      { pattern: /leg|foot/i, category: 'legs' },
      { pattern: /cloth|shirt|dress|coat|jacket/i, category: 'clothing' },
      { pattern: /accessory|acc|glasses|hat|bag/i, category: 'accessories' },
    ];
    
    const categorizedMeshes: Map<string, string[]> = new Map();
    
    this.vrm.scene.traverse((child) => {
      if (child instanceof THREE.Mesh && child.name) {
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
        
        // 未分类的 mesh
        if (!matched) {
          if (!categorizedMeshes.has('other')) {
            categorizedMeshes.set('other', []);
          }
          categorizedMeshes.get('other')!.push(child.name);
        }
      }
    });
    
    // 创建服装部件
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
    if (!this.vrm?.expressionManager) {
      console.warn("[VRMRenderer] No expressionManager, lip sync disabled");
      return;
    }
    
    const availableExpressions = this.expression.getAvailableExpressions();
    console.log("[VRMRenderer] All available expressions:", availableExpressions);
    
    // 创建 VRM 口型适配器
    this.lipSyncAdapter = createVRMLipSyncAdapter(
      (name, weight) => {
        console.log(`[VRMRenderer] Setting expression: ${name} = ${weight}`);
        this.vrm?.expressionManager?.setValue(name, weight);
      },
      () => {
        this.expression.resetExpression();
      },
      availableExpressions
    );
    
    // 注册到全局口型控制器
    lipSyncController.setTarget(this.lipSyncAdapter);
    
    const mouthShapes = availableExpressions.filter(e => /aa|ee|ih|oh|ou/i.test(e));
    console.log("[VRMRenderer] Lip sync enabled, matched mouth shapes:", mouthShapes);
    
    if (mouthShapes.length === 0) {
      console.warn("[VRMRenderer] No mouth shape expressions found! Looking for aa, ee, ih, oh, ou");
    }
  }
  
  private getVRMName(): string {
    if (!this.vrm?.meta) return "VRM Model";
    const meta = this.vrm.meta as { name?: string };
    return meta.name || "VRM Model";
  }
  
  private startRenderLoop(): void {
    // 避免重复启动
    if (this.animationId !== null) return;
    
    const animate = () => {
      this.animationId = requestAnimationFrame(animate);
      const delta = this.clock?.getDelta() ?? 0;
      
      this.controls?.update();
      
      // 更新动画混合器
      if (this.mixer) {
        this.mixer.update(delta);
      }
      
      // 更新 VRM (物理、表情等)
      if (this.vrm && this.physicsEnabled) {
        this.vrm.update(delta);
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
    
    // 销毁场景
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
    if (this.vrm) {
      this.vrm.scene.scale.setScalar(scale);
    }
  }
  
  setPosition(x: number, y: number): void {
    if (this.vrm) {
      this.vrm.scene.position.x = x;
      this.vrm.scene.position.y = y;
    }
  }
  
  resetView(): void {
    if (this.camera && this.controls) {
      this.camera.position.copy(VRM_SCENE_CONFIG.camera.position);
      this.controls.target.copy(VRM_SCENE_CONFIG.camera.target);
      this.controls.update();
    }
    
    if (this.vrm) {
      this.vrm.scene.scale.setScalar(1);
      this.vrm.scene.position.set(0, 0, 0);
    }
  }
}

/** VRM 渲染器工厂 */
export class VRMRendererFactory {
  canHandle(modelPath: string): boolean {
    return modelPath.endsWith(".vrm") || modelPath.endsWith(".glb");
  }
  
  create(container: HTMLElement): VRMRendererAdapter {
    return new VRMRendererAdapter(container);
  }
}
