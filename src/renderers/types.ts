// Model Renderer - 统一渲染器抽象层
// 为 Live2D 和 VRM 提供统一的接口

import type { Emotion } from "@/types";

/** 模型类型 */
export type ModelType = "live2d" | "vrm" | "gltf" | "fbx" | "mmd";

/** 动作优先级 */
export enum MotionPriority {
  None = 0,
  Idle = 1,
  Normal = 2,
  Force = 3,
}

/** 动作信息 */
export interface MotionInfo {
  index: number;
  name?: string;
}

/** 动作组信息 */
export interface MotionGroup {
  name: string;
  motions: MotionInfo[];
}

/** 模型元信息 */
export interface ModelMetadata {
  type: ModelType;
  name: string;
  path: string;
  expressions: string[];
  motions: MotionGroup[];
}

/** 渲染器事件类型 */
export type RendererEventType = 
  | "loaded" 
  | "error" 
  | "expressionChanged" 
  | "motionStarted" 
  | "motionEnded"
  | "disposed";

/** 渲染器事件处理函数 */
export type RendererEventHandler = (data?: unknown) => void;

/** 取消订阅函数 */
export type Unsubscribe = () => void;

// ═══════════════════════════════════════════════════════════════════════════
// 控制器接口
// ═══════════════════════════════════════════════════════════════════════════

/** 表情控制器接口 */
export interface ExpressionController {
  /** 设置表情 */
  setExpression(name: string, weight?: number): void;
  
  /** 获取可用表情列表 */
  getAvailableExpressions(): string[];
  
  /** 重置为默认表情 */
  resetExpression(): void;
  
  /** 根据情感设置表情 */
  setFromEmotion(emotion: Emotion): void;
}

/** 动作控制器接口 */
export interface MotionController {
  /** 播放动作 */
  playMotion(group: string, index?: number, priority?: MotionPriority): Promise<void>;
  
  /** 停止当前动作 */
  stopMotion(): void;
  
  /** 获取可用动作组列表 */
  getAvailableMotions(): MotionGroup[];
  
  /** 播放随机空闲动作 */
  playIdleMotion(): void;
  
  /** 加载外部动画文件 */
  loadAnimation?(animationPath: string): Promise<void>;
  
  /** 卸载指定动画 */
  unloadAnimation?(name: string): void;
  
  /** 卸载所有已加载的动画 */
  unloadAllAnimations?(): void;
}

/** 服装/部件信息 */
export interface OutfitPart {
  name: string;
  visible: boolean;
  meshNames: string[];
}

/** 服装控制器接口 */
export interface OutfitController {
  /** 获取所有可用的服装部件 */
  getAvailableParts(): OutfitPart[];
  
  /** 设置部件可见性 */
  setPartVisibility(partName: string, visible: boolean): void;
  
  /** 显示所有部件 */
  showAllParts(): void;
  
  /** 隐藏所有部件 */
  hideAllParts(): void;
  
  /** 加载外部服装/配件模型 */
  loadOutfit?(outfitPath: string): Promise<void>;
  
  /** 卸载服装/配件 */
  unloadOutfit?(outfitName: string): void;
}

/** 视线控制器接口 */
export interface LookAtController {
  /** 看向指定坐标 */
  lookAt(x: number, y: number): void;
  
  /** 启用/禁用自动跟踪鼠标 */
  setAutoLookAt(enabled: boolean): void;
  
  /** 重置视线 */
  resetLookAt(): void;
}

/** 物理控制器接口 */
export interface PhysicsController {
  /** 启用/禁用物理模拟 */
  setEnabled(enabled: boolean): void;
  
  /** 获取当前状态 */
  isEnabled(): boolean;
}

// ═══════════════════════════════════════════════════════════════════════════
// 渲染器端口定义
// ═══════════════════════════════════════════════════════════════════════════

/** 模型渲染器端口 - 统一的模型渲染抽象 */
export interface ModelRendererPort {
  // ─────────────────────────────────────────────────────────────────────────
  // 生命周期
  // ─────────────────────────────────────────────────────────────────────────
  
  /** 加载模型 */
  load(modelPath: string): Promise<void>;
  
  /** 卸载当前模型（保留渲染器实例） */
  unloadModel(): void;
  
  /** 销毁渲染器和模型 */
  dispose(): void;
  
  /** 是否已加载 */
  readonly isLoaded: boolean;
  
  // ─────────────────────────────────────────────────────────────────────────
  // 渲染控制
  // ─────────────────────────────────────────────────────────────────────────
  
  /** 调整大小 */
  resize(width: number, height: number): void;
  
  /** 设置缩放 */
  setScale(scale: number): void;
  
  /** 设置位置 */
  setPosition(x: number, y: number): void;
  
  /** 重置视图 */
  resetView(): void;
  
  // ─────────────────────────────────────────────────────────────────────────
  // 功能控制器 (可选能力)
  // ─────────────────────────────────────────────────────────────────────────
  
  /** 表情控制器 */
  readonly expression: ExpressionController;
  
  /** 动作控制器 */
  readonly motion: MotionController;
  
  /** 视线控制器 (可选) */
  readonly lookAt?: LookAtController;
  
  /** 物理控制器 (可选) */
  readonly physics?: PhysicsController;
  
  /** 服装控制器 (可选) */
  readonly outfit?: OutfitController;
  
  // ─────────────────────────────────────────────────────────────────────────
  // 事件
  // ─────────────────────────────────────────────────────────────────────────
  
  /** 订阅事件 */
  on(event: RendererEventType, handler: RendererEventHandler): Unsubscribe;
  
  /** 触发事件 */
  emit(event: RendererEventType, data?: unknown): void;
  
  // ─────────────────────────────────────────────────────────────────────────
  // 元数据
  // ─────────────────────────────────────────────────────────────────────────
  
  /** 模型类型 */
  readonly modelType: ModelType;
  
  /** 模型元信息 */
  readonly metadata: ModelMetadata | null;
}

// ═══════════════════════════════════════════════════════════════════════════
// 渲染器工厂
// ═══════════════════════════════════════════════════════════════════════════

/** 渲染器工厂接口 */
export interface RendererFactory {
  /** 创建渲染器 */
  create(container: HTMLElement): ModelRendererPort;
  
  /** 检查是否支持该模型 */
  canHandle(modelPath: string): boolean;
}

/** 渲染器注册表 - 管理多个渲染器工厂 */
export class RendererRegistry {
  private factories: RendererFactory[] = [];
  
  /** 注册一个渲染器工厂 */
  register(factory: RendererFactory): void {
    this.factories.push(factory);
  }
  
  /** 根据模型路径获取合适的工厂 */
  getFactory(modelPath: string): RendererFactory | null {
    return this.factories.find(f => f.canHandle(modelPath)) ?? null;
  }
  
  /** 创建渲染器 */
  createRenderer(modelPath: string, container: HTMLElement): ModelRendererPort | null {
    const factory = this.getFactory(modelPath);
    return factory?.create(container) ?? null;
  }
  
  /** 检查是否支持该模型 */
  canHandle(modelPath: string): boolean {
    return this.factories.some(f => f.canHandle(modelPath));
  }
}

// ═══════════════════════════════════════════════════════════════════════════
// 基础渲染器实现 (抽象类)
// ═══════════════════════════════════════════════════════════════════════════

/** 基础渲染器 - 提供公共功能 */
export abstract class BaseModelRenderer implements ModelRendererPort {
  protected container: HTMLElement;
  protected _isLoaded = false;
  protected _metadata: ModelMetadata | null = null;
  protected eventHandlers: Map<RendererEventType, Set<RendererEventHandler>> = new Map();
  
  // 可选控制器 - 子类可以覆盖
  lookAt?: LookAtController;
  physics?: PhysicsController;
  outfit?: OutfitController;
  
  constructor(container: HTMLElement) {
    this.container = container;
  }
  
  // 抽象方法 - 子类必须实现
  abstract load(modelPath: string): Promise<void>;
  abstract unloadModel(): void;
  abstract dispose(): void;
  abstract resize(width: number, height: number): void;
  abstract setScale(scale: number): void;
  abstract setPosition(x: number, y: number): void;
  abstract resetView(): void;
  abstract get modelType(): ModelType;
  abstract get expression(): ExpressionController;
  abstract get motion(): MotionController;
  
  get isLoaded(): boolean {
    return this._isLoaded;
  }
  
  get metadata(): ModelMetadata | null {
    return this._metadata;
  }
  
  // 事件系统
  on(event: RendererEventType, handler: RendererEventHandler): Unsubscribe {
    if (!this.eventHandlers.has(event)) {
      this.eventHandlers.set(event, new Set());
    }
    this.eventHandlers.get(event)!.add(handler);
    
    return () => {
      this.eventHandlers.get(event)?.delete(handler);
    };
  }
  
  emit(event: RendererEventType, data?: unknown): void {
    this.eventHandlers.get(event)?.forEach(handler => {
      try {
        handler(data);
      } catch (e) {
        console.error(`[BaseModelRenderer] Error in event handler for ${event}:`, e);
      }
    });
  }
  
  // 受保护的辅助方法
  protected setMetadata(metadata: ModelMetadata): void {
    this._metadata = metadata;
  }
  
  protected setLoaded(loaded: boolean): void {
    this._isLoaded = loaded;
    if (loaded) {
      this.emit("loaded", this._metadata);
    }
  }
}

// ═══════════════════════════════════════════════════════════════════════════
// 情感到表情的映射
// ═══════════════════════════════════════════════════════════════════════════

/** 情感到表情的默认映射 */
export const EMOTION_EXPRESSION_MAP: Record<Emotion, string[]> = {
  happy: ["happy", "smile", "joy", "aa"],
  sad: ["sad", "cry", "sorrow"],
  angry: ["angry", "mad", "furious"],
  surprised: ["surprised", "shock", "amazed", "oh"],
  neutral: ["neutral", "normal", "default", "idle"],
  thinking: ["thinking", "hmm", "curious"],
};

/** 根据情感找到最匹配的表情 */
export function findExpressionForEmotion(
  emotion: Emotion,
  availableExpressions: string[]
): string | null {
  const candidates = EMOTION_EXPRESSION_MAP[emotion] || [];
  const lowerExpressions = availableExpressions.map(e => e.toLowerCase());
  
  for (const candidate of candidates) {
    const index = lowerExpressions.findIndex(e => 
      e.includes(candidate) || candidate.includes(e)
    );
    if (index !== -1) {
      return availableExpressions[index];
    }
  }
  
  return null;
}
