import { logger } from "@/utils/logger";
import { useEffect } from "react";
import { useChatStore, useModelStore } from "@/stores";
import { chatService } from "@/services";
import { lipSyncController } from "@/services/LipSyncService";

export function useChat() {
  const {
    currentSession,
    messages,
    isGenerating,
    streamContent,
    error,
    sendMessage,
    stopGeneration,
    appendStreamContent,
    finalizeMessage,
    setError,
  } = useChatStore();

  const { setExpressionFromEmotion } = useModelStore();

  useEffect(() => {
    logger.debug(`[useChat] Setting up event listeners, currentSession:`, currentSession?.id);
    
    const unsubChunk = chatService.onMessageChunk((chunk) => {
      logger.debug(`[useChat] Received chunk:`, chunk, `currentSession:`, currentSession?.id);
      logger.debug(`[useChat] sessionId match:`, chunk.sessionId === currentSession?.id);
      if (chunk.sessionId === currentSession?.id) {
        appendStreamContent(chunk.content);
        
        // 驱动口型同步 - 优先使用后端提供的精确音素
        lipSyncController.processChunkWithPhonemes(chunk.content, chunk.phonemes);
      }
    });

    const unsubComplete = chatService.onMessageComplete((data) => {
      logger.debug(`[useChat] Received complete:`, data, `currentSession:`, currentSession?.id);
      if (data.sessionId === currentSession?.id) {
        finalizeMessage(data.messageId, data.emotion);
        if (data.emotion) {
          setExpressionFromEmotion(data.emotion);
        }
        
        // 通知口型同步完成
        lipSyncController.onComplete();
      }
    });

    const unsubError = chatService.onMessageError((data) => {
      logger.debug(`[useChat] Received error:`, data);
      if (data.sessionId === currentSession?.id) {
        setError(data.error);
        
        // 出错时停止口型
        lipSyncController.stop();
      }
    });

    return () => {
      logger.debug(`[useChat] Cleaning up event listeners`);
      unsubChunk();
      unsubComplete();
      unsubError();
    };
  }, [
    currentSession?.id,
    appendStreamContent,
    finalizeMessage,
    setError,
    setExpressionFromEmotion,
  ]);

  return {
    currentSession,
    messages,
    isGenerating,
    streamContent,
    error,
    sendMessage,
    stopGeneration,
  };
}
