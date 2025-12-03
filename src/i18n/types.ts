export interface Translation {
  common: {
    confirm: string;
    cancel: string;
    close: string;
    save: string;
    delete: string;
    edit: string;
    copy: string;
    retry: string;
    loading: string;
  };
  sidebar: {
    newChat: string;
    recent: string;
    noConversations: string;
    rename: string;
    deleteSession: string;
  };
  chat: {
    startChat: string;
    startChatDesc: string;
    inputPlaceholder: string;
    inputHint: string;
    you: string;
    aiAssistant: string;
    newChatTitle: string;
    // Context menu
    editMessage: string;
    copyMessage: string;
    deleteMessage: string;
    regenerate: string;
    // Token info
    tokens: string;
    inputTokens: string;
    outputTokens: string;
  };
  model: {
    preview: string;
    model: string;
    loading: string;
    dropHint: string;
    supportedFormats: string;
    resetView: string;
    controls: {
      zoom: string;
      rotate: string;
      pan: string;
    };
  };
  settings: {
    title: string;
    tabs: {
      general: string;
      llm: string;
      model: string;
      shortcuts: string;
    };
    general: {
      theme: string;
      themeLight: string;
      themeDark: string;
      themeSystem: string;
      language: string;
      autoStart: string;
    };
    llm: {
      title: string;
      addProvider: string;
      configuredProviders: string;
      default: string;
      setDefault: string;
      delete: string;
      edit: string;
      providerName: string;
      providerType: string;
      apiBaseUrl: string;
      apiKey: string;
      model: string;
      models: string;
      presetModels: string;
      fetchModels: string;
      fetchingModels: string;
      selectModel: string;
      customModel: string;
      noModels: string;
      streamResponse: string;
      ollamaHelp: string;
      claudeHelp: string;
      openaiHelp: string;
      customHelp: string;
    };
    model: {
      title: string;
      defaultType: string;
      physics: string;
    };
    shortcuts: {
      title: string;
      toggleWindow: string;
      togglePetMode: string;
      newChat: string;
      comingSoon: string;
    };
  };
}
