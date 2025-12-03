import React, { useEffect } from "react";
import { MainLayout } from "@/components/layout";
import { useSessionStore, useConfigStore, useUIStore } from "@/stores";
import { windowService } from "@/services";

const App: React.FC = () => {
  const { loadSessions } = useSessionStore();
  const { isLoaded: configLoaded } = useConfigStore();
  const { setWindowMode } = useUIStore();

  useEffect(() => {
    loadSessions();
  }, [loadSessions]);

  useEffect(() => {
    const unsubscribe = windowService.onModeChanged(({ mode }) => {
      setWindowMode(mode);
    });

    return unsubscribe;
  }, [setWindowMode]);

  if (!configLoaded) {
    return (
      <div className="h-screen flex items-center justify-center bg-gray-100 dark:bg-gray-900">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary-600" />
      </div>
    );
  }

  return (
    <>
      <MainLayout />
    </>
  );
};

export default App;
