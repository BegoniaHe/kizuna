// Renderers - 统一的模型渲染层
// 提供多种 3D 模型格式的抽象接口

export * from "./types";
export { Live2DRendererAdapter, Live2DRendererFactory } from "./Live2DRenderer";
export { VRMRendererAdapter, VRMRendererFactory } from "./VRMRenderer";
export { GLTFRendererAdapter, GLTFRendererFactory } from "./GLTFRenderer";
export { FBXRendererAdapter, FBXRendererFactory } from "./FBXRenderer";
export { MMDRendererAdapter, MMDRendererFactory } from "./MMDRenderer";

import { RendererRegistry } from "./types";
import { Live2DRendererFactory } from "./Live2DRenderer";
import { VRMRendererFactory } from "./VRMRenderer";
import { GLTFRendererFactory } from "./GLTFRenderer";
import { FBXRendererFactory } from "./FBXRenderer";
import { MMDRendererFactory } from "./MMDRenderer";

/** 创建默认的渲染器注册表 */
export function createDefaultRendererRegistry(): RendererRegistry {
  const registry = new RendererRegistry();
  // 注册顺序很重要：VRM 优先于 GLTF
  registry.register(new Live2DRendererFactory());
  registry.register(new VRMRendererFactory());
  registry.register(new GLTFRendererFactory());
  registry.register(new FBXRendererFactory());
  registry.register(new MMDRendererFactory());
  return registry;
}

/** 默认渲染器注册表单例 */
let defaultRegistry: RendererRegistry | null = null;

export function getDefaultRendererRegistry(): RendererRegistry {
  if (!defaultRegistry) {
    defaultRegistry = createDefaultRendererRegistry();
  }
  return defaultRegistry;
}
