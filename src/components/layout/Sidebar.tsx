import React, { useState, useRef, useEffect } from "react";
import { useSessionStore, useChatStore, useUIStore } from "@/stores";
import { Button, ContextMenu, useContextMenu } from "@/components/common";
import type { ContextMenuItem } from "@/components/common";
import { useI18n } from "@/i18n";
import type { Session } from "@/types";

export const Sidebar: React.FC = () => {
  const { sessions, createSession, deleteSession, renameSession } = useSessionStore();
  const { currentSession, setCurrentSession } = useChatStore();
  const { sidebarOpen, toggleSettings } = useUIStore();
  const { t } = useI18n();
  
  // 重命名状态
  const [editingSessionId, setEditingSessionId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState("");
  const inputRef = useRef<HTMLInputElement>(null);
  
  // 右键菜单状态
  const { position: contextMenuPos, show: showContextMenu, hide: hideContextMenu } = useContextMenu();
  const [contextMenuSession, setContextMenuSession] = useState<Session | null>(null);

  const handleNewChat = async () => {
    try {
      const session = await createSession();
      setCurrentSession(session);
    } catch (error) {
      console.error("Failed to create session:", error);
    }
  };

  const handleSelectSession = (session: Session) => {
    setCurrentSession(session);
  };

  const handleDeleteSession = async (e: React.MouseEvent, id: string) => {
    e.stopPropagation();
    await deleteSession(id);
    if (currentSession?.id === id) {
      setCurrentSession(null);
    }
  };

  // 双击开始编辑会话名称
  const handleDoubleClick = (session: Session) => {
    setEditingSessionId(session.id);
    setEditingTitle(session.title || "");
  };

  // 确认重命名
  const handleRenameConfirm = async () => {
    if (editingSessionId && editingTitle.trim()) {
      await renameSession(editingSessionId, editingTitle.trim());
      // 如果当前会话被重命名，更新 currentSession
      if (currentSession?.id === editingSessionId) {
        setCurrentSession({ ...currentSession, title: editingTitle.trim() });
      }
    }
    setEditingSessionId(null);
    setEditingTitle("");
  };

  // 取消重命名
  const handleRenameCancel = () => {
    setEditingSessionId(null);
    setEditingTitle("");
  };

  // 键盘事件处理
  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter") {
      handleRenameConfirm();
    } else if (e.key === "Escape") {
      handleRenameCancel();
    }
  };

  // 聚焦输入框
  useEffect(() => {
    if (editingSessionId && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [editingSessionId]);

  // 右键菜单处理
  const handleContextMenu = (e: React.MouseEvent, session: Session) => {
    setContextMenuSession(session);
    showContextMenu(e);
  };

  // 右键菜单项
  const getContextMenuItems = (): ContextMenuItem[] => {
    if (!contextMenuSession) return [];
    return [
      {
        label: t.sidebar.rename,
        icon: (
          <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
          </svg>
        ),
        onClick: () => {
          setEditingSessionId(contextMenuSession.id);
          setEditingTitle(contextMenuSession.title || "");
        },
      },
      { label: "", divider: true, onClick: () => {} },
      {
        label: t.sidebar.deleteSession,
        icon: (
          <svg fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
          </svg>
        ),
        onClick: async () => {
          await deleteSession(contextMenuSession.id);
          if (currentSession?.id === contextMenuSession.id) {
            setCurrentSession(null);
          }
        },
        danger: true,
      },
    ];
  };

  if (!sidebarOpen) return null;

  return (
    <aside className="w-full h-full flex flex-col border-r border-zinc-200 dark:border-zinc-800/50 bg-zinc-50/50 dark:bg-zinc-900/50 backdrop-blur-xl">
      <div className="h-16 flex items-center px-4 border-b border-zinc-200/50 dark:border-zinc-800/50">
        <div className="flex items-center gap-3 text-zinc-900 dark:text-zinc-100 font-semibold text-lg">
          <div className="w-8 h-8 rounded-xl bg-gradient-to-br from-primary-400 to-primary-600 flex items-center justify-center text-white shadow-lg shadow-primary-500/20">
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
            </svg>
          </div>
          <span className="tracking-tight">Kizuna</span>
        </div>
      </div>

      <div className="px-3 py-4">
        <Button 
          onClick={handleNewChat} 
          className="
            w-full justify-start gap-3 px-4 py-3
            bg-white dark:bg-zinc-800 
            border border-zinc-200 dark:border-zinc-700 
            hover:border-primary-500 dark:hover:border-primary-500
            hover:shadow-md hover:shadow-primary-500/10
            text-zinc-700 dark:text-zinc-200 
            shadow-sm transition-all duration-200 group
            rounded-xl
          "
        >
          <div className="w-6 h-6 rounded-lg bg-zinc-100 dark:bg-zinc-700 group-hover:bg-primary-50 dark:group-hover:bg-primary-900/30 flex items-center justify-center transition-colors">
            <svg className="w-4 h-4 text-zinc-500 dark:text-zinc-400 group-hover:text-primary-600 dark:group-hover:text-primary-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
            </svg>
          </div>
          <span className="font-medium">{t.sidebar.newChat}</span>
        </Button>
      </div>

      <div className="flex-1 overflow-y-auto px-3 py-2 space-y-1">
        <div className="text-xs font-semibold text-zinc-400 dark:text-zinc-500 px-3 py-2 uppercase tracking-wider mb-1">
          {t.sidebar.recent}
        </div>
        {sessions.length === 0 ? (
          <div className="px-3 py-8 text-center">
            <div className="w-12 h-12 mx-auto mb-3 rounded-full bg-zinc-100 dark:bg-zinc-800 flex items-center justify-center">
              <svg className="w-6 h-6 text-zinc-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
              </svg>
            </div>
            <p className="text-sm text-zinc-500 dark:text-zinc-400">{t.sidebar.noConversations}</p>
          </div>
        ) : (
          <ul className="space-y-1">
            {sessions.map((session) => (
              <li key={session.id}>
                <button
                  onClick={() => handleSelectSession(session)}
                  onDoubleClick={() => handleDoubleClick(session)}
                  onContextMenu={(e) => handleContextMenu(e, session)}
                  className={`
                    w-full text-left px-3 py-3 rounded-xl text-sm transition-all duration-200
                    flex items-center gap-3 group relative overflow-hidden
                    ${
                      currentSession?.id === session.id
                        ? "bg-white dark:bg-zinc-800 text-zinc-900 dark:text-zinc-100 shadow-sm ring-1 ring-zinc-200 dark:ring-zinc-700"
                        : "hover:bg-zinc-100 dark:hover:bg-zinc-800/50 text-zinc-600 dark:text-zinc-400"
                    }
                  `}
                >
                  {currentSession?.id === session.id && (
                    <div className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-8 bg-primary-500 rounded-r-full" />
                  )}
                  
                  <div className={`
                    flex-shrink-0 transition-colors
                    ${currentSession?.id === session.id ? "text-primary-500" : "text-zinc-400 group-hover:text-zinc-500 dark:group-hover:text-zinc-300"}
                  `}>
                    <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M8 10h.01M12 10h.01M16 10h.01M9 16H5a2 2 0 01-2-2V6a2 2 0 012-2h14a2 2 0 012 2v8a2 2 0 01-2 2h-5l-5 5v-5z" />
                    </svg>
                  </div>

                  {editingSessionId === session.id ? (
                    <input
                      ref={inputRef}
                      type="text"
                      value={editingTitle}
                      onChange={(e) => setEditingTitle(e.target.value)}
                      onBlur={handleRenameConfirm}
                      onKeyDown={handleKeyDown}
                      onClick={(e) => e.stopPropagation()}
                      className="flex-1 bg-transparent border-b border-primary-500 px-0 py-0.5 text-sm outline-none min-w-0"
                    />
                  ) : (
                    <span className="truncate flex-1 font-medium">{session.title || t.chat.newChatTitle}</span>
                  )}
                  
                  <div 
                    className={`
                      opacity-0 group-hover:opacity-100 transition-opacity flex items-center
                      ${currentSession?.id === session.id ? "opacity-100" : ""}
                    `}
                  >
                    <button
                      onClick={(e) => handleDeleteSession(e, session.id)}
                      className="p-1.5 rounded-md hover:bg-red-50 dark:hover:bg-red-900/30 text-zinc-400 hover:text-red-500 transition-colors"
                    >
                      <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                      </svg>
                    </button>
                  </div>
                </button>
              </li>
            ))}
          </ul>
        )}
      </div>

      <div className="p-4 border-t border-zinc-200 dark:border-zinc-800/50 bg-zinc-50/50 dark:bg-zinc-900/50">
        <Button 
          variant="ghost" 
          onClick={toggleSettings} 
          className="w-full justify-start gap-3 px-3 py-2.5 text-zinc-600 dark:text-zinc-400 hover:bg-white dark:hover:bg-zinc-800 hover:shadow-sm hover:text-zinc-900 dark:hover:text-zinc-100 transition-all rounded-xl"
        >
          <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
          </svg>
          <span className="font-medium">{t.settings.title}</span>
        </Button>
      </div>

      <ContextMenu 
        items={getContextMenuItems()} 
        position={contextMenuPos} 
        onClose={hideContextMenu} 
      />
    </aside>
  );
};
