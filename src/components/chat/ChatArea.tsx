import React from "react";
import { MessageList } from "./MessageList";
import { InputArea } from "./InputArea";
import { useChatStore, useUIStore } from "@/stores";
import { useChat } from "@/hooks";
import { Button } from "@/components/common";

export const ChatArea: React.FC = () => {
  // useChat hook 设置事件监听器
  useChat();
  
  const { currentSession, error } = useChatStore();
  const { toggleSidebar, sidebarOpen } = useUIStore();

  return (
    <div className="flex-1 flex flex-col h-full bg-white dark:bg-zinc-900 relative z-0">
      <header className="h-16 flex items-center justify-between px-4 border-b border-zinc-100 dark:border-zinc-800 bg-white/80 dark:bg-zinc-900/80 backdrop-blur-md sticky top-0 z-10">
        <div className="flex items-center gap-3">
          <Button variant="ghost" size="sm" onClick={toggleSidebar} className="text-zinc-500 hover:text-zinc-700 dark:text-zinc-400 dark:hover:text-zinc-200">
            <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              {sidebarOpen ? (
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 19l-7-7 7-7m8 14l-7-7 7-7" />
              ) : (
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
              )}
            </svg>
          </Button>
          <div className="flex flex-col">
            <div className="flex items-center gap-2">
              <h1 className="text-base font-bold text-zinc-800 dark:text-zinc-100 leading-tight">
                {currentSession?.title || "Kizuna"}
              </h1>
              {currentSession?.modelConfig?.model && (
                <span className="px-1.5 py-0.5 rounded text-[10px] font-medium bg-primary-50 text-primary-600 dark:bg-primary-900/20 dark:text-primary-400 border border-primary-100 dark:border-primary-800/30">
                  {currentSession.modelConfig.model}
                </span>
              )}
            </div>
          </div>
        </div>
      </header>

      {error && (
        <div className="mx-4 mt-4 p-3 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
          <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
        </div>
      )}

      <MessageList />
      <InputArea />
    </div>
  );
};
