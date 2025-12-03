import React, { useState, useRef, useEffect } from "react";
import { useChatStore } from "@/stores";
import { Button } from "@/components/common";
import { useI18n } from "@/i18n";

export const InputArea: React.FC = () => {
  const [input, setInput] = useState("");
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const { sendMessage, stopGeneration, isGenerating, currentSession } = useChatStore();
  const { t } = useI18n();

  useEffect(() => {
    if (textareaRef.current) {
      textareaRef.current.style.height = "auto";
      textareaRef.current.style.height = `${Math.min(textareaRef.current.scrollHeight, 200)}px`;
    }
  }, [input]);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!input.trim() || isGenerating || !currentSession) return;

    const message = input.trim();
    setInput("");
    await sendMessage(message);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit(e);
    }
  };

  return (
    <div className="p-4 bg-white dark:bg-zinc-900 relative z-10">
      <div className="max-w-3xl mx-auto w-full">
        <form
          onSubmit={handleSubmit}
          className="
            relative flex flex-col gap-2 p-3
            bg-white dark:bg-zinc-800
            border border-zinc-200 dark:border-zinc-700
            rounded-2xl shadow-xl shadow-zinc-200/50 dark:shadow-zinc-900/50
            focus-within:ring-2 focus-within:ring-primary-500/20 focus-within:border-primary-500
            transition-all duration-200 ease-in-out
          "
        >
          <div className="flex-1 min-w-0">
            <textarea
              ref={textareaRef}
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={handleKeyDown}
              placeholder={currentSession ? t.chat.inputPlaceholder : t.chat.startChat}
              disabled={!currentSession || isGenerating}
              rows={1}
              className="
                w-full px-2 py-2 max-h-[200px]
                bg-transparent border-none resize-none
                text-zinc-900 dark:text-zinc-100
                placeholder-zinc-400 dark:placeholder-zinc-500
                focus:ring-0 focus:outline-none
                disabled:opacity-50 disabled:cursor-not-allowed
                text-base leading-relaxed
              "
            />
          </div>

          <div className="flex items-center justify-between pt-2 border-t border-zinc-100 dark:border-zinc-700/50">
            <div className="flex items-center gap-2">
              {/* Tools Section */}
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="rounded-lg w-9 h-9 p-0 flex items-center justify-center text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors"
                disabled={!currentSession}
                title="Attach file"
              >
                 <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M15.172 7l-6.586 6.586a2 2 0 102.828 2.828l6.414-6.586a4 4 0 00-5.656-5.656l-6.415 6.585a6 6 0 108.486 8.486L20.5 13" />
                </svg>
              </Button>
              
              <Button
                type="button"
                variant="ghost"
                size="sm"
                className="rounded-lg w-9 h-9 p-0 flex items-center justify-center text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors"
                disabled={!currentSession}
                title="Web Search"
              >
                 <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9" />
                </svg>
              </Button>

               <Button
                type="button"
                variant="ghost"
                size="sm"
                className="rounded-lg w-9 h-9 p-0 flex items-center justify-center text-zinc-400 hover:text-zinc-600 dark:hover:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-700 transition-colors"
                disabled={!currentSession}
                title="Model Settings"
              >
                 <svg className="w-6 h-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M12 6V4m0 2a2 2 0 100 4m0-4a2 2 0 110 4m-6 8a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4m6 6v10m6-2a2 2 0 100-4m0 4a2 2 0 110-4m0 4v2m0-6V4" />
                </svg>
              </Button>
            </div>

            <div className="flex items-center gap-2">
              {isGenerating ? (
                <Button 
                  type="button" 
                  variant="danger" 
                  size="sm"
                  onClick={stopGeneration}
                  className="rounded-lg w-8 h-8 p-0 flex items-center justify-center shadow-sm"
                >
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </Button>
              ) : (
                <Button
                  type="submit"
                  variant="primary"
                  size="sm"
                  disabled={!input.trim() || !currentSession}
                  className={`
                    rounded-lg w-8 h-8 p-0 flex items-center justify-center shadow-sm transition-all duration-200
                    ${input.trim() ? 'bg-primary-600 hover:bg-primary-700 text-white' : 'bg-zinc-200 dark:bg-zinc-700 text-zinc-400 dark:text-zinc-500 cursor-not-allowed'}
                  `}
                >
                  <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 10l7-7m0 0l7 7m-7-7v18" />
                  </svg>
                </Button>
              )}
            </div>
          </div>
        </form>
        <div className="text-center mt-2">
           <span className="text-xs text-zinc-400 dark:text-zinc-500">
             {t.chat.inputHint}
           </span>
        </div>
      </div>
    </div>
  );
};
