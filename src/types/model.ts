export type ModelType = "live2d" | "vrm" | "gltf" | "fbx" | "mmd";

export interface ModelInfo {
  type: ModelType;
  path: string;
  name: string;
  expressions: string[];
  motions: MotionGroup[];
}

export interface MotionGroup {
  name: string;
  motions: MotionInfo[];
}

export interface MotionInfo {
  index: number;
  name?: string;
}

export enum MotionPriority {
  None = 0,
  Idle = 1,
  Normal = 2,
  Force = 3,
}

export interface ExpressionController {
  setExpression(name: string, weight?: number): void;
  getAvailableExpressions(): string[];
  resetExpression(): void;
}

export interface MotionController {
  playMotion(
    group: string,
    index?: number,
    priority?: MotionPriority,
  ): Promise<void>;
  stopMotion(): void;
  getAvailableMotions(): MotionGroup[];
}

export interface LookAtController {
  lookAt(x: number, y: number): void;
  setAutoLookAt(enabled: boolean): void;
}

export interface PhysicsController {
  setEnabled(enabled: boolean): void;
  update(deltaTime: number): void;
}

export type ModelEvent = "loaded" | "error" | "expressionChange" | "motionEnd";
export type EventHandler = (data?: unknown) => void;
export type Unsubscribe = () => void;

export interface ModelRendererPort {
  load(modelPath: string): Promise<void>;
  dispose(): void;
  resize(width: number, height: number): void;
  setScale(scale: number): void;
  setPosition(x: number, y: number): void;

  readonly expression?: ExpressionController;
  readonly motion?: MotionController;
  readonly lookAt?: LookAtController;
  readonly physics?: PhysicsController;

  on(event: ModelEvent, handler: EventHandler): Unsubscribe;

  readonly modelType: ModelType;
  readonly isLoaded: boolean;
  readonly modelInfo: ModelInfo | null;
}
