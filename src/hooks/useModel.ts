import { useModelStore } from "@/stores";

export function useModel() {
  const {
    modelType,
    modelPath,
    isLoading,
    isLoaded,
    currentExpression,
    currentMotion,
    modelInfo,
    error,
    loadModel,
    unloadModel,
    setExpression,
    playMotion,
  } = useModelStore();

  return {
    modelType,
    modelPath,
    isLoading,
    isLoaded,
    currentExpression,
    currentMotion,
    modelInfo,
    error,
    loadModel,
    unloadModel,
    setExpression,
    playMotion,
  };
}
