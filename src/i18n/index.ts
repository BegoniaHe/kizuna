import { useConfigStore } from "@/stores";
import { zhCN } from "./locales/zh-CN";
import { enUS } from "./locales/en-US";
import { jaJP } from "./locales/ja-JP";
import { Translation } from "./types";

const locales: Record<string, Translation> = {
  "zh-CN": zhCN,
  "en-US": enUS,
  "ja-JP": jaJP,
};

export const useI18n = () => {
  const { config } = useConfigStore();
  const language = config.general.language;
  
  const t = locales[language] || locales["en-US"];

  return { t, language };
};

export type { Translation };
