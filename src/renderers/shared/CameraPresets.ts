// 相机预设配置
// 为不同模型类型提供优化的相机和场景配置

import * as THREE from "three";
import type { SceneConfig, CameraConfig, ControlsConfig, LightingConfig } from "./ThreeSceneSetup";

// ═══════════════════════════════════════════════════════════════════════════
// 预设配置
// ═══════════════════════════════════════════════════════════════════════════

/** VRM 模型相机配置 - 人物模型优化 */
export const VRM_CAMERA_CONFIG: CameraConfig = {
  fov: 30,
  near: 0.1,
  far: 20,
  position: new THREE.Vector3(0, 1.2, 2),
  target: new THREE.Vector3(0, 1, 0),
};

/** VRM 模型控制器配置 */
export const VRM_CONTROLS_CONFIG: ControlsConfig = {
  enableDamping: true,
  dampingFactor: 0.05,
  screenSpacePanning: true,
  minDistance: 0.5,
  maxDistance: 10,
  maxPolarAngle: Math.PI,
};

/** GLTF 模型相机配置 - 通用 3D 模型 */
export const GLTF_CAMERA_CONFIG: CameraConfig = {
  fov: 30,
  near: 0.1,
  far: 100,
  position: new THREE.Vector3(0, 1.5, 3),
  target: new THREE.Vector3(0, 1, 0),
};

/** GLTF 模型控制器配置 */
export const GLTF_CONTROLS_CONFIG: ControlsConfig = {
  enableDamping: true,
  dampingFactor: 0.05,
  screenSpacePanning: true,
  minDistance: 0.5,
  maxDistance: 50,
};

/** FBX 模型相机配置 - 大尺度模型 (Mixamo 等) */
export const FBX_CAMERA_CONFIG: CameraConfig = {
  fov: 45,
  near: 1,
  far: 2000,
  position: new THREE.Vector3(0, 100, 200),
  target: new THREE.Vector3(0, 100, 0),
};

/** FBX 模型控制器配置 */
export const FBX_CONTROLS_CONFIG: ControlsConfig = {
  enableDamping: true,
  dampingFactor: 0.05,
  screenSpacePanning: true,
  minDistance: 10,
  maxDistance: 1000,
};

/** MMD 模型相机配置 - MMD 尺度 */
export const MMD_CAMERA_CONFIG: CameraConfig = {
  fov: 45,
  near: 1,
  far: 2000,
  position: new THREE.Vector3(0, 15, 30),
  target: new THREE.Vector3(0, 10, 0),
};

/** MMD 模型控制器配置 */
export const MMD_CONTROLS_CONFIG: ControlsConfig = {
  enableDamping: true,
  dampingFactor: 0.05,
  screenSpacePanning: true,
  minDistance: 5,
  maxDistance: 100,
};

// ═══════════════════════════════════════════════════════════════════════════
// 光照预设
// ═══════════════════════════════════════════════════════════════════════════

/** 标准人物光照 - VRM/GLTF */
export const CHARACTER_LIGHTING: LightingConfig = {
  ambient: {
    color: 0xffffff,
    intensity: 0.5,
  },
  directional: {
    color: 0xffffff,
    intensity: 1,
    position: new THREE.Vector3(1, 1, 1),
    castShadow: false,
  },
};

/** 完整场景光照 - FBX */
export const FULL_SCENE_LIGHTING: LightingConfig = {
  hemisphere: {
    skyColor: 0xffffff,
    groundColor: 0x444444,
    intensity: 1,
  },
  directional: {
    color: 0xffffff,
    intensity: 1,
    position: new THREE.Vector3(0, 200, 100),
    castShadow: true,
  },
};

/** 柔和光照 - MMD */
export const SOFT_LIGHTING: LightingConfig = {
  ambient: {
    color: 0xffffff,
    intensity: 0.6,
  },
  directional: {
    color: 0xffffff,
    intensity: 0.8,
    position: new THREE.Vector3(1, 1, 1),
    castShadow: false,
  },
};

/** 增强光照 - GLTF (带半球光) */
export const ENHANCED_LIGHTING: LightingConfig = {
  ambient: {
    color: 0xffffff,
    intensity: 0.6,
  },
  directional: {
    color: 0xffffff,
    intensity: 1,
    position: new THREE.Vector3(1, 2, 1),
    castShadow: true,
  },
  hemisphere: {
    skyColor: 0xffffff,
    groundColor: 0x444444,
    intensity: 0.5,
  },
};

// ═══════════════════════════════════════════════════════════════════════════
// 完整预设配置
// ═══════════════════════════════════════════════════════════════════════════

/** VRM 场景完整配置 */
export const VRM_SCENE_CONFIG: SceneConfig = {
  camera: VRM_CAMERA_CONFIG,
  controls: VRM_CONTROLS_CONFIG,
  lighting: CHARACTER_LIGHTING,
  renderer: {
    alpha: true,
    antialias: true,
  },
};

/** GLTF 场景完整配置 */
export const GLTF_SCENE_CONFIG: SceneConfig = {
  camera: GLTF_CAMERA_CONFIG,
  controls: GLTF_CONTROLS_CONFIG,
  lighting: ENHANCED_LIGHTING,
  renderer: {
    alpha: true,
    antialias: true,
  },
};

/** FBX 场景完整配置 */
export const FBX_SCENE_CONFIG: SceneConfig = {
  camera: FBX_CAMERA_CONFIG,
  controls: FBX_CONTROLS_CONFIG,
  lighting: FULL_SCENE_LIGHTING,
  renderer: {
    alpha: true,
    antialias: true,
    shadowMap: true,
  },
};

/** MMD 场景完整配置 */
export const MMD_SCENE_CONFIG: SceneConfig = {
  camera: MMD_CAMERA_CONFIG,
  controls: MMD_CONTROLS_CONFIG,
  lighting: SOFT_LIGHTING,
  renderer: {
    alpha: true,
    antialias: true,
    pixelRatioLimit: 2,
  },
};

// ═══════════════════════════════════════════════════════════════════════════
// 配置获取器
// ═══════════════════════════════════════════════════════════════════════════

import type { ModelType } from "../types";

/** 根据模型类型获取场景配置 */
export function getSceneConfigForModelType(modelType: ModelType): SceneConfig {
  switch (modelType) {
    case "vrm":
      return VRM_SCENE_CONFIG;
    case "gltf":
      return GLTF_SCENE_CONFIG;
    case "fbx":
      return FBX_SCENE_CONFIG;
    case "mmd":
      return MMD_SCENE_CONFIG;
    default:
      return GLTF_SCENE_CONFIG; // 默认使用 GLTF 配置
  }
}
