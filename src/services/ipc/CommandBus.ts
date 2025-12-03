import { invoke } from "@tauri-apps/api/core";
import { logger } from "@/utils/logger";

export interface CommandOptions {
  timeout?: number;
  retries?: number;
}

/**
 * 将命令名称从前端格式转换为 Tauri 命令格式
 * 例如: "chat:send_message" -> "chat_send_message"
 */
function normalizeCommandName(command: string): string {
  return command.replace(/:/g, "_");
}

class CommandBus {
  private defaultTimeout = 30000;
  private defaultRetries = 0;

  async dispatch<T, R>(
    command: string,
    payload?: T,
    options?: CommandOptions,
  ): Promise<R> {
    const { timeout = this.defaultTimeout, retries = this.defaultRetries } = options ?? {};
    const normalizedCommand = normalizeCommandName(command);

    logger.debug(`[CommandBus] Dispatching: ${command} -> ${normalizedCommand}`, payload);

    let lastError: Error | null = null;
    for (let attempt = 0; attempt <= retries; attempt++) {
      try {
        const result = await Promise.race([
          invoke<R>(normalizedCommand, payload as Record<string, unknown>),
          this.createTimeout(timeout),
        ]);
        logger.debug(`[CommandBus] Success: ${normalizedCommand}`, result);
        return result as R;
      } catch (error) {
        lastError = error instanceof Error ? error : new Error(String(error));
        console.error(`[CommandBus] Error: ${normalizedCommand}`, lastError);
        if (attempt < retries) {
          await this.delay(Math.pow(2, attempt) * 100);
        }
      }
    }

    throw lastError ?? new Error("Command failed");
  }

  private createTimeout(ms: number): Promise<never> {
    return new Promise((_, reject) => {
      setTimeout(() => reject(new Error("Command timeout")), ms);
    });
  }

  private delay(ms: number): Promise<void> {
    return new Promise((resolve) => setTimeout(resolve, ms));
  }
}

export const commandBus = new CommandBus();
