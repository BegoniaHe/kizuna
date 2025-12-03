import React, { useState, useRef, useEffect, useCallback } from "react";
import type { Message } from "@/types";
import { Avatar, ContextMenu, useContextMenu } from "@/components/common";
import type { ContextMenuItem } from "@/components/common";
import { MarkdownRenderer } from "./MarkdownRenderer";
import { useI18n } from "@/i18n";
import { useChatStore, type TokenUsage } from "@/stores/chatStore";

interface MessageItemProps {
  message: Message;
  isStreaming?: boolean;
  isNewMessage?: boolean;
  isLastAssistantMessage?: boolean;
  tokenUsage?: TokenUsage | null;
}

export const MessageItem: React.FC<MessageItemProps> = ({
  message,
  isStreaming = false,
  isNewMessage = false,
  tokenUsage,
}) => {
  const isUser = message.role === "user";
  const { t } = useI18n();
  const { editMessage, deleteMessage, deleteMessagesFrom, regenerateFrom, currentSession } = useChatStore();
  const { position: contextMenuPos, show: showContextMenu, hide: hideContextMenu } = useContextMenu();

  // 获取显示名称
  const displayName = isUser 
    ? t.chat.you 
    : (currentSession?.modelConfig?.model || t.chat.aiAssistant);

  // 编辑状态
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState(message.content);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const contentRef = useRef<HTMLDivElement>(null);
  const [contentSize, setContentSize] = useState<{ width: number; height: number } | null>(null);

  // 双击开始编辑
  const handleDoubleClick = () => {
    // 记录当前内容尺寸
    if (contentRef.current) {
      setContentSize({
        width: contentRef.current.offsetWidth,
        height: contentRef.current.offsetHeight,
      });
    }
    setIsEditing(true);
    setEditContent(message.content);
  };

  // 确认编辑
  const handleEditConfirm = useCallback(async () => {
    if (editContent.trim() && editContent !== message.content) {
      if (isUser) {
        // 用户消息：编辑后删除下文并重新请求
        editMessage(message.id, editContent.trim());
        // 删除此消息之后的所有消息
        deleteMessagesFrom(message.id);
        // 重新发送（会自动使用编辑后的内容）
        await regenerateFrom(message.id);
      } else {
        // AI 消息：编辑后保存并删除下文
        editMessage(message.id, editContent.trim());
        // 找到下一条消息并删除从那里开始的所有消息
        const messages = useChatStore.getState().messages;
        const currentIndex = messages.findIndex((m) => m.id === message.id);
        if (currentIndex !== -1 && currentIndex < messages.length - 1) {
          deleteMessagesFrom(messages[currentIndex + 1].id);
        }
      }
    }
    setIsEditing(false);
  }, [editContent, message, isUser, editMessage, deleteMessagesFrom, regenerateFrom]);

  // 取消编辑
  const handleEditCancel = () => {
    setIsEditing(false);
    setEditContent(message.content);
  };

  // 键盘事件
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleEditConfirm();
    } else if (e.key === "Escape") {
      handleEditCancel();
    }
  };

  // 复制消息
  const handleCopy = () => {
    navigator.clipboard.writeText(message.content);
  };

  // 删除消息
  const handleDelete = () => {
    deleteMessage(message.id);
  };

  // 重新生成
  const handleRegenerate = () => {
    regenerateFrom(message.id);
  };

  // 自动调整 textarea 高度
  useEffect(() => {
    if (isEditing && textareaRef.current) {
      textareaRef.current.focus();
      textareaRef.current.select();
      // 自动调整高度
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${textareaRef.current.scrollHeight}px`;
    }
  }, [isEditing]);

  // 构建右键菜单项
  const contextMenuItems: ContextMenuItem[] = [
    {
      label: t.chat.editMessage,
      icon: (
        <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
        </svg>
      ),
      onClick: () => {
        setIsEditing(true);
        setEditContent(message.content);
      },
    },
    {
      label: t.chat.copyMessage,
      icon: (
        <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
        </svg>
      ),
      onClick: handleCopy,
    },
    {
      label: t.chat.regenerate,
      icon: (
        <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
        </svg>
      ),
      onClick: handleRegenerate,
    },
    { label: "", divider: true, onClick: () => {} },
    {
      label: t.chat.deleteMessage,
      icon: (
        <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
        </svg>
      ),
      onClick: handleDelete,
      danger: true,
    },
  ];

  return (
    <div 
      className={`
        group flex w-full mb-6 
        ${isUser ? "justify-end" : "justify-start"}
        ${isNewMessage ? "animate-fade-in" : ""}
      `}
      onContextMenu={(e) => {
        e.preventDefault();
        showContextMenu(e);
      }}
    >
      <div 
        className={`
          flex max-w-3xl w-full gap-4 
          ${isUser ? "flex-row-reverse" : "flex-row"}
        `}
      >
        <Avatar 
          role={message.role} 
          className={`
            flex-shrink-0 w-9 h-9 mt-1 shadow-sm rounded-xl
            ${isUser 
              ? "bg-primary-100 text-primary-600 dark:bg-primary-900/30 dark:text-primary-400" 
              : "bg-gradient-to-br from-indigo-500 to-purple-600 text-white border-0"
            }
          `} 
        />
        
        <div className={`flex flex-col flex-1 min-w-0 ${isUser ? "items-end" : "items-start"}`}>
          <div className={`flex items-center gap-2 mb-1 px-1 ${isUser ? "flex-row-reverse" : "flex-row"}`}>
            <span className="text-sm font-semibold text-zinc-800 dark:text-zinc-200">
              {displayName}
            </span>
            <span className="text-[10px] text-zinc-400 dark:text-zinc-500 font-medium">
              {(() => {
                try {
                  const date = new Date(message.createdAt);
                  if (isNaN(date.getTime())) return "";
                  return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
                } catch {
                  return "";
                }
              })()}
            </span>
          </div>

          <div 
            ref={contentRef}
            onDoubleClick={handleDoubleClick}
            className={`
              relative group/content rounded-2xl px-4 py-3 text-sm leading-relaxed shadow-sm
              ${isUser 
                ? "bg-primary-50 dark:bg-primary-900/20 text-zinc-800 dark:text-zinc-100 border border-primary-100 dark:border-primary-800/30 rounded-tr-sm" 
                : "bg-white dark:bg-zinc-800 text-zinc-800 dark:text-zinc-100 border border-zinc-100 dark:border-zinc-700/50 rounded-tl-sm"
              }
              ${isEditing ? "w-full ring-2 ring-primary-500 ring-offset-2 dark:ring-offset-zinc-900" : ""}
            `}
          >
            {isEditing ? (
              <div className="w-full">
                <textarea
                  ref={textareaRef}
                  value={editContent}
                  onChange={(e) => setEditContent(e.target.value)}
                  onKeyDown={handleKeyDown}
                  className="w-full bg-transparent border-none resize-none focus:ring-0 p-0 text-sm leading-relaxed"
                  style={{ 
                    height: contentSize ? `${contentSize.height}px` : 'auto',
                    minHeight: '60px'
                  }}
                  autoFocus
                />
                <div className="flex justify-end gap-2 mt-2">
                  <button 
                    onClick={handleEditCancel}
                    className="text-xs px-2 py-1 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-500"
                  >
                    Cancel
                  </button>
                  <button 
                    onClick={handleEditConfirm}
                    className="text-xs px-2 py-1 rounded bg-primary-600 text-white hover:bg-primary-700"
                  >
                    Save & Submit
                  </button>
                </div>
              </div>
            ) : (
              <MarkdownRenderer content={message.content} isStreaming={isStreaming} />
            )}
          </div>

          {/* Token Usage & Footer */}
          {!isEditing && (
            <div className={`flex items-center gap-2 mt-1 px-1 flex-row`}>
              {tokenUsage && (
                <span className="text-[10px] text-zinc-300 dark:text-zinc-600 select-none">
                  {tokenUsage.totalTokens} tokens
                </span>
              )}

              {/* Actions */}
              <div className={`
                flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity flex-row
              `}>
                <button 
                  onClick={handleCopy}
                  className="p-1 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 rounded transition-colors"
                  title={t.chat.copyMessage}
                >
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" />
                  </svg>
                </button>
                <button 
                  onClick={handleDoubleClick}
                  className="p-1 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 rounded transition-colors"
                  title={t.chat.editMessage}
                >
                  <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
                  </svg>
                </button>
                {!isUser && (
                  <button 
                    onClick={handleRegenerate}
                    className="p-1 text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 rounded transition-colors"
                    title={t.chat.regenerate}
                  >
                    <svg className="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                    </svg>
                  </button>
                )}
              </div>
            </div>
          )}
        </div>
      </div>

      <ContextMenu
        items={contextMenuItems}
        position={contextMenuPos}
        onClose={hideContextMenu}
      />
    </div>
  );
};
