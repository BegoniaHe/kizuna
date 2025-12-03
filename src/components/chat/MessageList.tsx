import React, { useRef, useEffect, useState, useCallback, useMemo } from "react";
import { useChatStore } from "@/stores";
import { MessageItem } from "./MessageItem";
import { MarkdownRenderer } from "./MarkdownRenderer";
import { Avatar } from "@/components/common";
import { useI18n } from "@/i18n";

export const MessageList: React.FC = () => {
  const { messages, streamContent, isGenerating, lastTokenUsage } = useChatStore();
  const { t } = useI18n();
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const messagesEndRef = useRef<HTMLDivElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const [newMessageId, setNewMessageId] = useState<string | null>(null);
  const prevMessagesLengthRef = useRef(messages.length);
  const shouldAutoScrollRef = useRef(true);

  // 找到最后一条 AI 消息的 ID
  const lastAssistantMessageId = useMemo(() => {
    for (let i = messages.length - 1; i >= 0; i--) {
      if (messages[i].role === "assistant") {
        return messages[i].id;
      }
    }
    return null;
  }, [messages]);

  // Track new messages
  useEffect(() => {
    if (messages.length > prevMessagesLengthRef.current) {
      const lastMessage = messages[messages.length - 1];
      if (lastMessage.role === "assistant") {
        setNewMessageId(lastMessage.id);
        // Clear after animation completes
        const timer = setTimeout(() => setNewMessageId(null), 5000);
        return () => clearTimeout(timer);
      }
    }
    prevMessagesLengthRef.current = messages.length;
  }, [messages]);

  // 处理滚动事件，判断用户是否手动向上滚动
  const handleScroll = useCallback(() => {
    if (scrollContainerRef.current) {
      const { scrollTop, scrollHeight, clientHeight } = scrollContainerRef.current;
      // 如果距离底部小于 50px，认为是在底部，允许自动滚动
      const isAtBottom = scrollHeight - scrollTop - clientHeight < 50;
      shouldAutoScrollRef.current = isAtBottom;
    }
  }, []);

  // 滚动到底部
  const scrollToBottom = useCallback((force = false) => {
    if (scrollContainerRef.current) {
      if (force || shouldAutoScrollRef.current) {
        const container = scrollContainerRef.current;
        // 对于流式输出，直接设置 scrollTop 以避免抖动
        container.scrollTop = container.scrollHeight;
        
        // 如果是强制滚动，重置自动滚动状态
        if (force) {
          shouldAutoScrollRef.current = true;
        }
      }
    }
  }, []);

  // 监听内容高度变化，处理图片加载等导致的布局变化
  useEffect(() => {
    const contentElement = contentRef.current;
    if (!contentElement) return;

    const observer = new ResizeObserver(() => {
      if (shouldAutoScrollRef.current) {
        scrollToBottom();
      }
    });

    observer.observe(contentElement);

    return () => observer.disconnect();
  }, [scrollToBottom, messages.length]); // 添加 messages.length 依赖以确保在切换会话时重新绑定

  // 监听消息列表变化（新消息添加时强制滚动）
  useEffect(() => {
    if (messages.length > 0) {
      // 使用 setTimeout 确保 DOM 已经完全更新
      setTimeout(() => scrollToBottom(true), 0);
    }
  }, [messages.length, scrollToBottom]);

  // 监听流式内容变化（仅在允许时滚动）
  useEffect(() => {
    if (streamContent) {
      scrollToBottom(false);
    }
  }, [streamContent, scrollToBottom]);

  // 监听生成状态变化（开始生成时也需要滚动，因为会出现 loading 状态）
  useEffect(() => {
    if (isGenerating) {
      setTimeout(() => scrollToBottom(true), 0);
    }
  }, [isGenerating, scrollToBottom]);

  return (
    <div 
      ref={scrollContainerRef} 
      className="flex-1 overflow-y-auto p-4"
      onScroll={handleScroll}
    >
      {messages.length === 0 && !streamContent ? (
        <div className="h-full flex items-center justify-center">
          <div className="text-center text-zinc-500 dark:text-zinc-400">
            <svg
              className="w-16 h-16 mx-auto mb-4 opacity-50"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={1.5}
                d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z"
              />
            </svg>
            <p className="text-lg font-medium">{t.chat.startChat}</p>
            <p className="text-sm mt-1">{t.chat.startChatDesc}</p>
          </div>
        </div>
      ) : (
        <div ref={contentRef} className="flex flex-col">
          {messages.map((message) => (
            <MessageItem 
              key={message.id} 
              message={message} 
              isNewMessage={message.id === newMessageId}
              isLastAssistantMessage={message.id === lastAssistantMessageId}
              tokenUsage={message.id === lastAssistantMessageId ? lastTokenUsage : null}
            />
          ))}

          {isGenerating && streamContent && (
            <div className="flex w-full mb-6 justify-start">
              <div className="flex max-w-3xl w-full gap-4 flex-row">
                <Avatar role="assistant" className="flex-shrink-0 w-8 h-8 mt-1" />
                <div className="flex flex-col flex-1 min-w-0 items-start">
                  <div className="flex items-center gap-2 mb-1 px-1">
                    <span className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                      {t.chat.aiAssistant}
                    </span>
                    <span className="text-xs text-zinc-400 dark:text-zinc-500">
                      Thinking...
                    </span>
                  </div>
                  <div className="text-sm leading-relaxed max-w-full w-fit shadow-sm bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 px-4 py-3 rounded-2xl rounded-tl-sm border border-zinc-100 dark:border-zinc-700">
                    <MarkdownRenderer content={streamContent} isStreaming={true} />
                  </div>
                </div>
              </div>
            </div>
          )}

          {isGenerating && !streamContent && (
            <div className="flex w-full mb-6 justify-start">
              <div className="flex max-w-3xl w-full gap-4 flex-row">
                <Avatar role="assistant" className="flex-shrink-0 w-8 h-8 mt-1" />
                <div className="flex flex-col flex-1 min-w-0 items-start">
                  <div className="flex items-center gap-2 mb-1 px-1">
                    <span className="text-sm font-medium text-zinc-900 dark:text-zinc-100">
                      {t.chat.aiAssistant}
                    </span>
                  </div>
                  <div className="px-1">
                    <div className="flex space-x-1">
                      <div className="w-2 h-2 bg-zinc-400 rounded-full animate-bounce" style={{ animationDelay: "0ms" }} />
                      <div className="w-2 h-2 bg-zinc-400 rounded-full animate-bounce" style={{ animationDelay: "150ms" }} />
                      <div className="w-2 h-2 bg-zinc-400 rounded-full animate-bounce" style={{ animationDelay: "300ms" }} />
                    </div>
                  </div>
                </div>
              </div>
            </div>
          )}
        </div>
      )}
      <div ref={messagesEndRef} />
    </div>
  );
};
