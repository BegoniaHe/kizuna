import React from "react";
import { ModelViewer } from "@/components/model/ModelViewer";
import { windowService } from "@/services";

export const PetModeLayout: React.FC = () => {
  const handleMouseDown = async () => {
    try {
      await windowService.startDragging();
    } catch (error) {
      console.error("Failed to start dragging:", error);
    }
  };

  return (
    <div
      className="h-screen w-screen bg-transparent cursor-move"
      onMouseDown={handleMouseDown}
    >
      <ModelViewer compact />
    </div>
  );
};
