// Live2D Renderer Adapter
// 封装 pixi-live2d-display 提供统一的渲染接口 - 增强版

import { Application, Container } from "pixi.js";
import type { Live2DModel } from "pixi-live2d-display";
import type { Emotion } from "@/types";
import {
  BaseModelRenderer,
  type ExpressionController,
  type MotionController,
  type LookAtController,
  type MotionGroup,
  type ModelType,
  MotionPriority,
  findExpressionForEmotion,
} from "./types";

// Live2D Model 内部类型
interface Live2DMotionManager {
  definitions: Record<string, unknown[]>;
  expressionManager?: {
    definitions?: Array<{ name?: string }>;
  };
  stopAllMotions(): void;
}

interface Live2DInternalModel {
  motionManager: Live2DMotionManager;
}

/** Live2D 渲染器适配器 */
export class Live2DRendererAdapter extends BaseModelRenderer {
  private app: Application | null = null;
  private model: Live2DModel | null = null;
  private currentScale = 1;
  private autoInteractEnabled = true;
  
  readonly modelType: ModelType = "live2d";
  
  // ─────────────────────────────────────────────────────────────────────────
  // 表情控制器
  // ─────────────────────────────────────────────────────────────────────────
  readonly expression: ExpressionController = {
    setExpression: (name: string) => {
      if (this.model) {
        // pixi-live2d-display 的表情 API
        this.model.expression(name);
        this.emit("expressionChanged", { expression: name });
      }
    },
    
    getAvailableExpressions: (): string[] => {
      if (!this.model) return [];
      const internalModel = this.model.internalModel as Live2DInternalModel;
      const expressionManager = internalModel.motionManager.expressionManager;
      return expressionManager?.definitions?.map(d => d.name || "unknown") ?? [];
    },
    
    resetExpression: () => {
      if (this.model) {
        this.model.expression(); // 无参数重置
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
  // 动作控制器
  // ─────────────────────────────────────────────────────────────────────────
  readonly motion: MotionController = {
    playMotion: async (group: string, index?: number, priority = MotionPriority.Normal) => {
      if (this.model) {
        this.emit("motionStarted", { group, index });
        // pixi-live2d-display 的 motion 方法接受 undefined 作为 priority
        await this.model.motion(group, index, priority as number | undefined);
        this.emit("motionEnded", { group, index });
      }
    },
    
    stopMotion: () => {
      if (this.model) {
        const internalModel = this.model.internalModel as Live2DInternalModel;
        internalModel.motionManager.stopAllMotions();
        this.emit("motionEnded", {});
      }
    },
    
    getAvailableMotions: (): MotionGroup[] => {
      if (!this.model) return [];
      const internalModel = this.model.internalModel as Live2DInternalModel;
      const definitions = internalModel.motionManager.definitions;
      
      return Object.entries(definitions || {}).map(([name, defs]) => ({
        name,
        motions: (defs as unknown[]).map((_, i) => ({ index: i })),
      }));
    },
    
    playIdleMotion: () => {
      const motions = this.motion.getAvailableMotions();
      const idleGroup = motions.find(g => 
        g.name.toLowerCase().includes("idle") || 
        g.name.toLowerCase().includes("stand")
      );
      if (idleGroup && idleGroup.motions.length > 0) {
        const randomIndex = Math.floor(Math.random() * idleGroup.motions.length);
        this.motion.playMotion(idleGroup.name, randomIndex, MotionPriority.Idle);
      }
    },
    
    // Live2D 动作是内置的，不支持外部加载
    loadAnimation: undefined,
    unloadAnimation: undefined,
    unloadAllAnimations: undefined,
  };
  
  // ─────────────────────────────────────────────────────────────────────────
  // 视线控制器
  // ─────────────────────────────────────────────────────────────────────────
  readonly lookAt: LookAtController = {
    lookAt: (x: number, y: number) => {
      if (this.model && this.autoInteractEnabled) {
        this.model.focus(x, y);
      }
    },
    
    setAutoLookAt: (enabled: boolean) => {
      // 存储状态，用于鼠标跟踪等功能
      this.autoInteractEnabled = enabled;
    },
    
    resetLookAt: () => {
      if (this.model) {
        this.model.focus(0.5, 0.5);
      }
    },
  };
  
  /** 获取当前缩放比例 */
  getScale(): number {
    return this.currentScale;
  }
  
  /** 是否启用自动视线跟踪 */
  isAutoLookAtEnabled(): boolean {
    return this.autoInteractEnabled;
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
      
      // 动态导入 pixi-live2d-display
      const { Live2DModel } = await import("pixi-live2d-display");
      
      // 初始化 PixiJS（如果需要）
      if (!this.app) {
        this.app = new Application({
          backgroundAlpha: 0,
          resizeTo: this.container,
          resolution: window.devicePixelRatio,
          autoDensity: true,
        });
        
        this.container.appendChild(this.app.view as HTMLCanvasElement);
      }
      
      // 加载模型
      this.model = await Live2DModel.from(modelPath, {
        autoInteract: true,
        autoUpdate: true,
      });
      
      // 缩放和定位
      this.resetView();
      
      // 添加到舞台
      this.app.stage.addChild(this.model as unknown as Container);
      
      // 设置元数据
      this.setMetadata({
        type: "live2d",
        path: modelPath,
        name: modelPath.split("/").pop() || "Live2D Model",
        expressions: this.expression.getAvailableExpressions(),
        motions: this.motion.getAvailableMotions(),
      });
      
      this.setLoaded(true);
    } catch (error) {
      this.emit("error", error);
      throw error;
    }
  }
  
  /** 卸载模型但保留渲染器 */
  unloadModel(): void {
    // 停止所有动作
    this.motion.stopMotion();
    
    // 销毁模型
    if (this.model) {
      if (this.app) {
        this.app.stage.removeChild(this.model as unknown as Container);
      }
      this.model.destroy();
      this.model = null;
    }
    
    this._isLoaded = false;
    this._metadata = null;
    
    console.log("[Live2DRenderer] Model unloaded");
  }
  
  dispose(): void {
    // 先卸载模型
    this.unloadModel();
    
    // 销毁 PixiJS 应用
    if (this.app) {
      this.app.destroy(true);
      this.app = null;
    }
    
    this.emit("disposed");
  }
  
  // ─────────────────────────────────────────────────────────────────────────
  // 渲染控制
  // ─────────────────────────────────────────────────────────────────────────
  resize(width: number, height: number): void {
    if (this.app?.renderer) {
      this.app.renderer.resize(width, height);
      this.resetView();
    }
  }
  
  setScale(scale: number): void {
    if (this.model) {
      this.currentScale = scale;
      (this.model as any).scale.set(scale);
    }
  }
  
  setPosition(x: number, y: number): void {
    if (this.model) {
      // Cast to any to access PixiJS display object properties
      const modelAny = this.model as any;
      modelAny.x = x;
      modelAny.y = y;
    }
  }
  
  resetView(): void {
    if (!this.model || !this.container) return;
    
    // Cast to any to access PixiJS display object properties
    const modelAny = this.model as any;
    const scale = Math.min(
      this.container.clientWidth / modelAny.width,
      this.container.clientHeight / modelAny.height,
    ) * 0.8;
    
    this.currentScale = scale;
    modelAny.scale.set(scale);
    modelAny.x = this.container.clientWidth / 2;
    modelAny.y = this.container.clientHeight / 2;
    modelAny.anchor.set(0.5, 0.5);
  }
}

/** Live2D 渲染器工厂 */
export class Live2DRendererFactory {
  canHandle(modelPath: string): boolean {
    return modelPath.endsWith(".model.json") || 
           modelPath.endsWith(".model3.json");
  }
  
  create(container: HTMLElement): Live2DRendererAdapter {
    return new Live2DRendererAdapter(container);
  }
}
