// Three.js 场景设置工具
// 提供场景、相机、控制器、光照的标准化初始化

import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";

// ═══════════════════════════════════════════════════════════════════════════
// 配置接口
// ═══════════════════════════════════════════════════════════════════════════

/** 相机配置 */
export interface CameraConfig {
  fov: number;
  near: number;
  far: number;
  position: THREE.Vector3;
  target: THREE.Vector3;
}

/** 控制器配置 */
export interface ControlsConfig {
  enableDamping: boolean;
  dampingFactor: number;
  screenSpacePanning: boolean;
  minDistance: number;
  maxDistance: number;
  maxPolarAngle?: number;
}

/** 光照配置 */
export interface LightingConfig {
  ambient?: {
    color: number;
    intensity: number;
  };
  directional?: {
    color: number;
    intensity: number;
    position: THREE.Vector3;
    castShadow?: boolean;
  };
  hemisphere?: {
    skyColor: number;
    groundColor: number;
    intensity: number;
  };
}

/** 渲染器配置 */
export interface RendererConfig {
  alpha?: boolean;
  antialias?: boolean;
  shadowMap?: boolean;
  pixelRatioLimit?: number;
}

/** 完整场景配置 */
export interface SceneConfig {
  camera: CameraConfig;
  controls: ControlsConfig;
  lighting: LightingConfig;
  renderer?: RendererConfig;
}

// ═══════════════════════════════════════════════════════════════════════════
// 场景组件
// ═══════════════════════════════════════════════════════════════════════════

/** Three.js 场景组件集合 */
export interface SceneComponents {
  scene: THREE.Scene;
  camera: THREE.PerspectiveCamera;
  renderer: THREE.WebGLRenderer;
  controls: OrbitControls;
  clock: THREE.Clock;
}

/** 场景销毁清理函数 */
export type SceneDisposer = () => void;

// ═══════════════════════════════════════════════════════════════════════════
// 工厂函数
// ═══════════════════════════════════════════════════════════════════════════

/**
 * 创建 Three.js 场景
 */
export function createScene(): THREE.Scene {
  const scene = new THREE.Scene();
  scene.background = null; // 透明背景
  return scene;
}

/**
 * 创建透视相机
 */
export function createCamera(
  container: HTMLElement,
  config: CameraConfig
): THREE.PerspectiveCamera {
  const aspect = container.clientWidth / container.clientHeight;
  const camera = new THREE.PerspectiveCamera(
    config.fov,
    aspect,
    config.near,
    config.far
  );
  camera.position.copy(config.position);
  return camera;
}

/**
 * 创建 WebGL 渲染器
 */
export function createRenderer(
  container: HTMLElement,
  config: RendererConfig = {}
): THREE.WebGLRenderer {
  const renderer = new THREE.WebGLRenderer({
    alpha: config.alpha ?? true,
    antialias: config.antialias ?? true,
  });
  
  renderer.setSize(container.clientWidth, container.clientHeight);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio, config.pixelRatioLimit ?? 2));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  
  if (config.shadowMap) {
    renderer.shadowMap.enabled = true;
  }
  
  container.appendChild(renderer.domElement);
  return renderer;
}

/**
 * 创建轨道控制器
 */
export function createControls(
  camera: THREE.PerspectiveCamera,
  renderer: THREE.WebGLRenderer,
  config: ControlsConfig,
  target: THREE.Vector3
): OrbitControls {
  const controls = new OrbitControls(camera, renderer.domElement);
  
  controls.target.copy(target);
  controls.enableDamping = config.enableDamping;
  controls.dampingFactor = config.dampingFactor;
  controls.screenSpacePanning = config.screenSpacePanning;
  controls.minDistance = config.minDistance;
  controls.maxDistance = config.maxDistance;
  
  if (config.maxPolarAngle !== undefined) {
    controls.maxPolarAngle = config.maxPolarAngle;
  }
  
  controls.update();
  return controls;
}

/**
 * 设置场景光照
 */
export function setupLighting(scene: THREE.Scene, config: LightingConfig): void {
  // 环境光
  if (config.ambient) {
    const ambient = new THREE.AmbientLight(
      config.ambient.color,
      config.ambient.intensity
    );
    scene.add(ambient);
  }
  
  // 平行光
  if (config.directional) {
    const directional = new THREE.DirectionalLight(
      config.directional.color,
      config.directional.intensity
    );
    directional.position.copy(config.directional.position);
    
    if (config.directional.castShadow) {
      directional.castShadow = true;
      directional.shadow.camera.top = 180;
      directional.shadow.camera.bottom = -100;
      directional.shadow.camera.left = -120;
      directional.shadow.camera.right = 120;
    }
    
    scene.add(directional);
  }
  
  // 半球光
  if (config.hemisphere) {
    const hemisphere = new THREE.HemisphereLight(
      config.hemisphere.skyColor,
      config.hemisphere.groundColor,
      config.hemisphere.intensity
    );
    scene.add(hemisphere);
  }
}

// ═══════════════════════════════════════════════════════════════════════════
// 完整场景初始化
// ═══════════════════════════════════════════════════════════════════════════

/**
 * 初始化完整的 Three.js 场景
 * 返回场景组件和清理函数
 */
export function initializeScene(
  container: HTMLElement,
  config: SceneConfig
): { components: SceneComponents; dispose: SceneDisposer } {
  // 创建各组件
  const scene = createScene();
  const camera = createCamera(container, config.camera);
  const renderer = createRenderer(container, config.renderer);
  const controls = createControls(camera, renderer, config.controls, config.camera.target);
  const clock = new THREE.Clock();
  
  // 设置光照
  setupLighting(scene, config.lighting);
  
  // 组件集合
  const components: SceneComponents = {
    scene,
    camera,
    renderer,
    controls,
    clock,
  };
  
  // 清理函数
  const dispose: SceneDisposer = () => {
    controls.dispose();
    renderer.dispose();
    renderer.domElement.remove();
  };
  
  return { components, dispose };
}

// ═══════════════════════════════════════════════════════════════════════════
// 工具函数
// ═══════════════════════════════════════════════════════════════════════════

/**
 * 计算模型包围盒并返回适合的相机距离
 */
export function calculateFitDistance(
  object: THREE.Object3D,
  camera: THREE.PerspectiveCamera,
  padding = 1.5
): { center: THREE.Vector3; size: THREE.Vector3; distance: number } {
  const box = new THREE.Box3().setFromObject(object);
  const center = box.getCenter(new THREE.Vector3());
  const size = box.getSize(new THREE.Vector3());
  const maxDim = Math.max(size.x, size.y, size.z);
  
  const fov = camera.fov * (Math.PI / 180);
  const distance = (maxDim / (2 * Math.tan(fov / 2))) * padding;
  
  return { center, size, distance };
}

/**
 * 自动调整相机以适应模型
 */
export function fitCameraToObject(
  object: THREE.Object3D,
  camera: THREE.PerspectiveCamera,
  controls: OrbitControls,
  options: {
    padding?: number;
    verticalOffset?: number;
  } = {}
): void {
  const { center, size, distance } = calculateFitDistance(
    object,
    camera,
    options.padding ?? 1.5
  );
  
  const verticalOffset = options.verticalOffset ?? size.y * 0.1;
  
  camera.position.set(center.x, center.y + verticalOffset, center.z + distance);
  controls.target.copy(center);
  controls.update();
}

/**
 * 更新相机和渲染器尺寸
 */
export function updateSize(
  container: HTMLElement,
  camera: THREE.PerspectiveCamera,
  renderer: THREE.WebGLRenderer
): void {
  const { clientWidth, clientHeight } = container;
  camera.aspect = clientWidth / clientHeight;
  camera.updateProjectionMatrix();
  renderer.setSize(clientWidth, clientHeight);
}
