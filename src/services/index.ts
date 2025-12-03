export { chatService, type IChatService } from "./ChatService";
export { sessionService, type ISessionService } from "./SessionService";
export { windowService, type IWindowService } from "./WindowService";
export { configService, type IConfigService } from "./ConfigService";
export * from "./ipc";
export { 
  lipSyncController, 
  type Phoneme, 
  type LipSyncTarget, 
  type LipSyncConfig,
  createVRMLipSyncAdapter,
  createMMDLipSyncAdapter,
} from "./LipSyncService";
