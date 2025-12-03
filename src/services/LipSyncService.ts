// Lip Sync Service - 口型同步服务
// 将文字流转换为口型动画

import { logger } from "@/utils/logger";

// ═══════════════════════════════════════════════════════════════════════════
// 口型音素类型定义
// ═══════════════════════════════════════════════════════════════════════════

/** 口型音素 (基于 AEIOU 五元音系统) */
export type Phoneme = "A" | "E" | "I" | "O" | "U" | "N" | "closed";

/** 口型帧 */
export interface LipFrame {
  phoneme: Phoneme;
  weight: number;
  duration: number; // 毫秒
}

/** 口型控制器接口 - 渲染器需要实现这个 */
export interface LipSyncTarget {
  setMouthShape(phoneme: Phoneme, weight: number): void;
  resetMouth(): void;
}

// ═══════════════════════════════════════════════════════════════════════════
// 前端回退用的简单音素映射 (当后端未提供音素时使用)
// ═══════════════════════════════════════════════════════════════════════════

/**
 * 简单的字符到口型映射 (不需要完整拼音库)
 * 基于汉字的读音特点进行概率性映射
 */
function getPhonemeForChar(char: string): Phoneme {
  const code = char.charCodeAt(0);
  
  // 非汉字字符
  if (code < 0x4e00 || code > 0x9fff) {
    // 标点符号 - 闭嘴
    if (/[，。！？、；：""''（）\s,.!?;:()"\s]/.test(char)) {
      return "closed";
    }
    // 英文字母 - 简单映射
    const lower = char.toLowerCase();
    if ("aeiou".includes(lower)) {
      return lower.toUpperCase() as Phoneme;
    }
    // 其他辅音 - 随机元音
    return ["A", "E", "I", "O", "U"][Math.floor(Math.random() * 5)] as Phoneme;
  }
  
  // 汉字 - 基于 Unicode 编码伪随机分配
  // 这不是真正的拼音转换，但足够产生自然的口型变化
  const hash = (code * 7 + code >> 3) % 100;
  
  if (hash < 25) return "A";      // 25%
  if (hash < 45) return "I";      // 20%
  if (hash < 60) return "E";      // 15%
  if (hash < 75) return "O";      // 15%
  if (hash < 90) return "U";      // 15%
  return "N";                     // 10% - 鼻音
}

// ═══════════════════════════════════════════════════════════════════════════
// 口型同步控制器
// ═══════════════════════════════════════════════════════════════════════════

export interface LipSyncConfig {
  /** 每个字符的持续时间 (ms) */
  charDuration: number;
  /** 口型过渡时间 (ms) */
  transitionDuration: number;
  /** 说话时的口型权重 (0-1) */
  speakingWeight: number;
  /** 是否启用 */
  enabled: boolean;
}

const DEFAULT_CONFIG: LipSyncConfig = {
  charDuration: 80,         // 每个字 80ms
  transitionDuration: 50,   // 过渡 50ms
  speakingWeight: 0.7,      // 口型权重 70%
  enabled: true,
};

class LipSyncController {
  private target: LipSyncTarget | null = null;
  private config: LipSyncConfig = { ...DEFAULT_CONFIG };
  private queue: LipFrame[] = [];
  private isPlaying = false;
  private animationTimer: number | null = null;
  private lastChunkTime = 0;
  
  /** 设置口型控制目标 (渲染器) */
  setTarget(target: LipSyncTarget | null): void {
    console.log("[LipSync] setTarget called:", target ? "adapter registered" : "null (clearing)");
    this.target = target;
    if (!target) {
      this.stop();
    }
  }
  
  /** 更新配置 */
  setConfig(config: Partial<LipSyncConfig>): void {
    this.config = { ...this.config, ...config };
  }
  
  /** 获取配置 */
  getConfig(): LipSyncConfig {
    return { ...this.config };
  }
  
  /** 处理文字流块 (使用前端本地音素转换) */
  processChunk(content: string): void {
    if (!this.config.enabled || !this.target) return;
    
    this.lastChunkTime = Date.now();
    
    // 将文字转换为口型帧
    const frames = this.textToFrames(content);
    this.queue.push(...frames);
    
    // 开始播放
    if (!this.isPlaying) {
      this.startPlayback();
    }
  }
  
  /** 处理带音素的文字流块 (使用后端 rust-pinyin 生成的精确音素) */
  processChunkWithPhonemes(content: string, phonemes?: string[]): void {
    if (!this.config.enabled) {
      console.log("[LipSync] Lip sync is disabled");
      return;
    }
    if (!this.target) {
      console.log("[LipSync] No target registered, skipping");
      return;
    }
    
    console.log(`[LipSync] Processing chunk: "${content}", phonemes:`, phonemes);
    
    this.lastChunkTime = Date.now();
    
    let frames: LipFrame[];
    
    if (phonemes && phonemes.length > 0) {
      // 使用后端提供的精确音素
      frames = this.phonemesToFrames(phonemes, content.length);
      logger.debug(`[LipSync] Using backend phonemes: ${phonemes.join(',')}`);
    } else {
      // 回退到前端本地转换
      frames = this.textToFrames(content);
    }
    
    this.queue.push(...frames);
    
    // 开始播放
    if (!this.isPlaying) {
      this.startPlayback();
    }
  }
  
  /** 将音素数组转换为口型帧序列 */
  private phonemesToFrames(phonemes: string[], textLength: number): LipFrame[] {
    const frames: LipFrame[] = [];
    
    // 计算每个音素的平均持续时间
    const avgDuration = Math.max(
      this.config.charDuration,
      (textLength * this.config.charDuration) / phonemes.length
    );
    
    for (const phonemeStr of phonemes) {
      const upperPhoneme = phonemeStr.toUpperCase();
      
      // 判断音素类型
      let phoneme: Phoneme;
      let isSilent = false;
      
      if (upperPhoneme === "CLOSED" || phonemeStr === "closed") {
        phoneme = "closed";
        isSilent = true;
      } else if (upperPhoneme === "N") {
        phoneme = "N";
        isSilent = true;
      } else if (["A", "E", "I", "O", "U"].includes(upperPhoneme)) {
        phoneme = upperPhoneme as Phoneme;
      } else {
        phoneme = "A"; // 默认
      }
      
      frames.push({
        phoneme,
        weight: isSilent ? 0.1 : this.config.speakingWeight,
        duration: avgDuration,
      });
    }
    
    return frames;
  }
  
  /** 文字流完成 */
  onComplete(): void {
    // 添加闭嘴帧
    this.queue.push({
      phoneme: "closed",
      weight: 0,
      duration: this.config.transitionDuration * 2,
    });
  }
  
  /** 停止口型 */
  stop(): void {
    this.isPlaying = false;
    if (this.animationTimer) {
      cancelAnimationFrame(this.animationTimer);
      this.animationTimer = null;
    }
    this.queue = [];
    this.target?.resetMouth();
  }
  
  /** 将文字转换为口型帧序列 */
  private textToFrames(text: string): LipFrame[] {
    const frames: LipFrame[] = [];
    
    for (const char of text) {
      const phoneme = getPhonemeForChar(char);
      
      // 如果和上一个音素相同，合并
      const lastFrame = frames[frames.length - 1];
      if (lastFrame && lastFrame.phoneme === phoneme) {
        lastFrame.duration += this.config.charDuration;
      } else {
        frames.push({
          phoneme,
          weight: phoneme === "closed" || phoneme === "N" 
            ? 0.1 
            : this.config.speakingWeight,
          duration: this.config.charDuration,
        });
      }
    }
    
    return frames;
  }
  
  /** 开始播放口型动画 */
  private startPlayback(): void {
    if (this.isPlaying) return;
    this.isPlaying = true;
    
    let lastTime = performance.now();
    let frameTimeRemaining = 0;
    let currentFrame: LipFrame | null = null;
    
    const animate = (time: number) => {
      if (!this.isPlaying) return;
      
      const deltaTime = time - lastTime;
      lastTime = time;
      
      // 消费当前帧时间
      if (currentFrame) {
        frameTimeRemaining -= deltaTime;
        
        // 计算当前权重 (带过渡效果)
        const progress = 1 - (frameTimeRemaining / currentFrame.duration);
        const easedWeight = this.easeInOut(progress) * currentFrame.weight;
        
        this.applyMouthShape(currentFrame.phoneme, easedWeight);
      }
      
      // 切换到下一帧
      if (frameTimeRemaining <= 0) {
        currentFrame = this.queue.shift() || null;
        
        if (currentFrame) {
          frameTimeRemaining = currentFrame.duration;
        } else {
          // 队列空了，检查是否需要继续等待
          const timeSinceLastChunk = Date.now() - this.lastChunkTime;
          
          if (timeSinceLastChunk > 500) {
            // 超过 500ms 没有新内容，闭嘴
            this.target?.resetMouth();
            this.isPlaying = false;
            return;
          }
          // 继续等待新内容
        }
      }
      
      this.animationTimer = requestAnimationFrame(animate);
    };
    
    this.animationTimer = requestAnimationFrame(animate);
  }
  
  /** 应用口型到目标 */
  private applyMouthShape(phoneme: Phoneme, weight: number): void {
    if (!this.target) return;
    
    // 平滑过渡
    this.target.setMouthShape(phoneme, Math.max(0, Math.min(1, weight)));
  }
  
  /** 缓动函数 */
  private easeInOut(t: number): number {
    return t < 0.5 
      ? 2 * t * t 
      : 1 - Math.pow(-2 * t + 2, 2) / 2;
  }
}

// 单例导出
export const lipSyncController = new LipSyncController();

// ═══════════════════════════════════════════════════════════════════════════
// 辅助：VRM/MMD 口型适配器
// ═══════════════════════════════════════════════════════════════════════════

/** VRM 口型 BlendShape 名称 - 支持多种命名变体 */
export const VRM_MOUTH_SHAPES: Record<Phoneme, string[]> = {
  "A": ["aa", "a", "mouth_a", "vrc.v_aa", "fcl_mth_a"],
  "E": ["ee", "e", "mouth_e", "vrc.v_ee", "fcl_mth_e"], 
  "I": ["ih", "i", "mouth_i", "vrc.v_ih", "fcl_mth_i"],
  "O": ["oh", "o", "mouth_o", "vrc.v_oh", "fcl_mth_o"],
  "U": ["ou", "u", "mouth_u", "vrc.v_ou", "fcl_mth_u"],
  "N": ["nn", "n"],
  "closed": [],
};

/** MMD 口型 Morph 名称 */
export const MMD_MOUTH_SHAPES: Record<Phoneme, string[]> = {
  "A": ["あ", "a"],
  "E": ["え", "e"],
  "I": ["い", "i"],
  "O": ["お", "o"],
  "U": ["う", "u"],
  "N": ["ん", "n"],
  "closed": [],
};

/**
 * 创建 VRM 口型控制适配器
 */
export function createVRMLipSyncAdapter(
  setExpression: (name: string, weight: number) => void,
  _resetExpression: () => void,
  availableExpressions: string[]
): LipSyncTarget {
  const lowerExpressions = availableExpressions.map(e => e.toLowerCase());
  
  // 查找可用的口型表情
  const findShape = (phoneme: Phoneme): string | null => {
    const candidates = VRM_MOUTH_SHAPES[phoneme];
    if (!candidates || candidates.length === 0) return null;
    
    // 尝试每个候选名称
    for (const candidate of candidates) {
      const index = lowerExpressions.findIndex(e => 
        e === candidate || e.includes(candidate)
      );
      if (index !== -1) {
        return availableExpressions[index];
      }
    }
    return null;
  };
  
  // 预先计算可用的口型映射
  const shapeMap: Partial<Record<Phoneme, string>> = {};
  const phonemes: Phoneme[] = ["A", "E", "I", "O", "U", "N"];
  for (const p of phonemes) {
    const shape = findShape(p);
    if (shape) {
      shapeMap[p] = shape;
    }
  }
  
  console.log("[LipSync] VRM shape mapping:", shapeMap);
  
  let lastPhoneme: Phoneme = "closed";
  
  return {
    setMouthShape(phoneme: Phoneme, weight: number): void {
      // 重置上一个口型
      if (lastPhoneme !== phoneme && lastPhoneme !== "closed") {
        const lastShape = shapeMap[lastPhoneme];
        if (lastShape) {
          setExpression(lastShape, 0);
        }
      }
      
      // 设置新口型
      const shape = shapeMap[phoneme];
      if (shape) {
        setExpression(shape, weight);
      }
      
      lastPhoneme = phoneme;
    },
    
    resetMouth(): void {
      if (lastPhoneme !== "closed") {
        const shape = shapeMap[lastPhoneme];
        if (shape) {
          setExpression(shape, 0);
        }
      }
      lastPhoneme = "closed";
    },
  };
}

/**
 * 创建 MMD 口型控制适配器
 */
export function createMMDLipSyncAdapter(
  mesh: { morphTargetDictionary?: Record<string, number>; morphTargetInfluences?: number[] }
): LipSyncTarget {
  const dict = mesh.morphTargetDictionary || {};
  const influences = mesh.morphTargetInfluences || [];
  
  // 查找可用的口型 morph
  const findMorphIndex = (phoneme: Phoneme): number => {
    const candidates = MMD_MOUTH_SHAPES[phoneme] || [];
    for (const name of candidates) {
      if (dict[name] !== undefined) {
        return dict[name];
      }
    }
    return -1;
  };
  
  let lastIndex = -1;
  
  return {
    setMouthShape(phoneme: Phoneme, weight: number): void {
      // 重置上一个
      if (lastIndex !== -1 && influences[lastIndex] !== undefined) {
        influences[lastIndex] = 0;
      }
      
      // 设置新的
      const index = findMorphIndex(phoneme);
      if (index !== -1 && influences[index] !== undefined) {
        influences[index] = weight;
        lastIndex = index;
      }
    },
    
    resetMouth(): void {
      if (lastIndex !== -1 && influences[lastIndex] !== undefined) {
        influences[lastIndex] = 0;
      }
      lastIndex = -1;
    },
  };
}

logger.debug("[LipSyncService] Initialized");
