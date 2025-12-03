import React, { useState, useCallback, useEffect, useRef } from "react";
import { Sidebar } from "./Sidebar";
import { ChatArea } from "@/components/chat/ChatArea";
import { ModelViewer } from "@/components/model/ModelViewer";
import { SettingsModal } from "@/components/settings/SettingsModal";
import { useUIStore } from "@/stores";

export const MainLayout: React.FC = () => {
  const { 
    sidebarOpen, 
    sidebarWidth, 
    setSidebarWidth, 
    modelPanelWidth, 
    setModelPanelWidth, 
    settingsOpen, 
    setSettingsOpen 
  } = useUIStore();
  const [isResizingSidebar, setIsResizingSidebar] = useState(false);
  const [isResizingModel, setIsResizingModel] = useState(false);
  const containerRef = useRef<HTMLDivElement>(null);

  const startResizingSidebar = useCallback(() => {
    setIsResizingSidebar(true);
  }, []);

  const startResizingModel = useCallback(() => {
    setIsResizingModel(true);
  }, []);

  const stopResizing = useCallback(() => {
    setIsResizingSidebar(false);
    setIsResizingModel(false);
  }, []);

  const resize = useCallback(
    (mouseMoveEvent: MouseEvent) => {
      if (isResizingSidebar) {
        const newWidth = mouseMoveEvent.clientX;
        if (newWidth >= 200 && newWidth <= 480) {
          setSidebarWidth(newWidth);
        }
      }
      if (isResizingModel && containerRef.current) {
        const containerRect = containerRef.current.getBoundingClientRect();
        const newWidth = containerRect.right - mouseMoveEvent.clientX;
        if (newWidth >= 280 && newWidth <= 600) {
          setModelPanelWidth(newWidth);
        }
      }
    },
    [isResizingSidebar, isResizingModel, setSidebarWidth, setModelPanelWidth]
  );

  useEffect(() => {
    window.addEventListener("mousemove", resize);
    window.addEventListener("mouseup", stopResizing);
    return () => {
      window.removeEventListener("mousemove", resize);
      window.removeEventListener("mouseup", stopResizing);
    };
  }, [resize, stopResizing]);

  return (
    <div ref={containerRef} className="h-screen flex overflow-hidden bg-zinc-50 dark:bg-zinc-950 text-zinc-900 dark:text-zinc-100">
      <div
        className="relative flex-shrink-0 transition-all duration-300 ease-in-out overflow-hidden shadow-xl z-20"
        style={{ width: sidebarOpen ? sidebarWidth : 0 }}
      >
        <Sidebar />
        {/* Sidebar Resize Handle */}
        <div
          className="absolute top-0 right-0 w-1 h-full cursor-col-resize hover:bg-primary-500/50 active:bg-primary-500 transition-colors z-50 group"
          onMouseDown={startResizingSidebar}
        >
           <div className="absolute right-0 top-0 w-4 h-full -mr-2 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
             <div className="w-1 h-8 bg-zinc-300 dark:bg-zinc-600 rounded-full" />
           </div>
        </div>
      </div>

      <main className="flex-1 flex overflow-hidden min-w-0 relative z-10">
        <div className="flex-1 flex flex-col min-w-0 bg-white dark:bg-zinc-900">
          <ChatArea />
        </div>

        <div 
          className="relative flex-shrink-0 border-l border-zinc-200 dark:border-zinc-800/50 bg-zinc-50/50 dark:bg-zinc-900/50 backdrop-blur-sm"
          style={{ width: modelPanelWidth }}
        >
          {/* Model Panel Resize Handle */}
          <div
            className="absolute top-0 left-0 w-1 h-full cursor-col-resize hover:bg-primary-500/50 active:bg-primary-500 transition-colors z-50 group"
            onMouseDown={startResizingModel}
          >
            <div className="absolute left-0 top-0 w-4 h-full -ml-2 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
             <div className="w-1 h-8 bg-zinc-300 dark:bg-zinc-600 rounded-full" />
           </div>
          </div>
          <ModelViewer />
        </div>
      </main>

      <SettingsModal isOpen={settingsOpen} onClose={() => setSettingsOpen(false)} />
    </div>
  );
};