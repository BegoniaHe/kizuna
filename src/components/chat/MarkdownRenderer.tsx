import React, { useEffect, useRef, useState, useCallback, memo } from "react";
import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";
import rehypeHighlight from "rehype-highlight";
import mermaid from "mermaid";
import "highlight.js/styles/atom-one-dark.css";
import { useI18n } from "@/i18n";

// 初始化 Mermaid
mermaid.initialize({
  startOnLoad: false,
  theme: "dark",
  securityLevel: "loose",
  fontFamily: "inherit",
});

// 内联右键菜单组件
const InlineContextMenu: React.FC<{
  items: Array<{ icon: React.ReactNode; label: string; onClick: () => void; divider?: boolean }>;
  position: { x: number; y: number };
  onClose: () => void;
}> = ({ items, position, onClose }) => {
  const menuRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handleClickOutside = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose();
      }
    };
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, [onClose]);

  return (
    <div
      ref={menuRef}
      className="absolute z-50 bg-white dark:bg-zinc-800 rounded-lg shadow-lg border border-zinc-200 dark:border-zinc-700 py-1 min-w-[140px]"
      style={{ left: position.x, top: position.y }}
    >
      {items.map((item, index) => {
        if (item.divider) {
          return <div key={index} className="my-1 border-t border-zinc-200 dark:border-zinc-700" />;
        }
        return (
          <button
            key={index}
            className="w-full px-3 py-2 text-left text-sm text-zinc-700 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-700 flex items-center gap-2"
            onClick={() => {
              item.onClick();
              onClose();
            }}
          >
            <span className="w-4 h-4">{item.icon}</span>
            {item.label}
          </button>
        );
      })}
    </div>
  );
};

interface MarkdownRendererProps {
  content: string;
  isStreaming?: boolean;
  enableTypewriter?: boolean;
  typingSpeed?: number;
}

const Cursor = () => (
  <span className="inline-block w-2 h-4 ml-0.5 bg-indigo-500 animate-pulse align-middle" />
);

// Mermaid 图表渲染组件
const MermaidDiagram: React.FC<{ code: string }> = ({ code }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const [svg, setSvg] = useState<string>("");
  const [error, setError] = useState<string>("");
  const [contextMenuPos, setContextMenuPos] = useState<{ x: number; y: number } | null>(null);
  
  // 使用 code 的 hash 作为稳定 ID
  const idRef = useRef(`mermaid-${Math.random().toString(36).substr(2, 9)}-${Date.now()}`);

  useEffect(() => {
    const timer = setTimeout(() => {
      const renderDiagram = async () => {
        if (!code.trim()) return;
        
        try {
          // 每次渲染使用新 ID 避免缓存问题
          const uniqueId = `mermaid-${Math.random().toString(36).substr(2, 9)}-${Date.now()}`;
          idRef.current = uniqueId;
          
          // 验证语法
          await mermaid.parse(code);
          // 渲染图表
          const { svg: renderedSvg } = await mermaid.render(uniqueId, code);
          setSvg(renderedSvg);
          setError("");
        } catch (err) {
          console.error("Mermaid render error:", err);
          setError(err instanceof Error ? err.message : "Failed to render diagram");
          setSvg("");
        }
      };

      renderDiagram();
    }, 500); // Debounce 500ms to prevent freezing during streaming

    return () => clearTimeout(timer);
  }, [code]);

  if (error) {
    return (
      <div className="my-4 p-4 bg-red-50 dark:bg-red-900/20 border border-red-200 dark:border-red-800 rounded-lg">
        <div className="text-sm text-red-600 dark:text-red-400 mb-2">Mermaid 渲染错误:</div>
        <pre className="text-xs text-red-500 dark:text-red-300 overflow-x-auto">{error}</pre>
        <details className="mt-2">
          <summary className="text-xs text-red-400 cursor-pointer">查看源码</summary>
          <pre className="mt-2 text-xs bg-gray-100 dark:bg-gray-900 p-2 rounded overflow-x-auto">{code}</pre>
        </details>
      </div>
    );
  }

  if (!svg) {
    return (
      <div className="my-4 p-4 bg-gray-100 dark:bg-gray-800 rounded-lg animate-pulse">
        <div className="h-32 flex items-center justify-center text-gray-400">
          渲染中...
        </div>
      </div>
    );
  }

  // 保存 SVG 为图片
  const handleSaveSvg = async () => {
    try {
      const svgElement = containerRef.current?.querySelector('svg');
      if (!svgElement) return;

      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      const bbox = svgElement.getBoundingClientRect();
      const scale = 2;
      canvas.width = bbox.width * scale;
      canvas.height = bbox.height * scale;
      ctx.scale(scale, scale);

      const svgData = new XMLSerializer().serializeToString(svgElement);
      const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' });
      const url = URL.createObjectURL(svgBlob);

      const img = new Image();
      img.onload = () => {
        ctx.fillStyle = '#1a1a2e';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.drawImage(img, 0, 0, bbox.width, bbox.height);
        URL.revokeObjectURL(url);

        const link = document.createElement('a');
        link.download = `mermaid-diagram-${Date.now()}.png`;
        link.href = canvas.toDataURL('image/png');
        link.click();
      };
      img.src = url;
    } catch (err) {
      console.error('Failed to save diagram:', err);
    }
  };

  // 复制图片
  const handleCopyImage = async () => {
    try {
      const svgElement = containerRef.current?.querySelector('svg');
      if (!svgElement) return;

      const canvas = document.createElement('canvas');
      const ctx = canvas.getContext('2d');
      if (!ctx) return;

      const bbox = svgElement.getBoundingClientRect();
      const scale = 2;
      canvas.width = bbox.width * scale;
      canvas.height = bbox.height * scale;
      ctx.scale(scale, scale);

      const svgData = new XMLSerializer().serializeToString(svgElement);
      const svgBlob = new Blob([svgData], { type: 'image/svg+xml;charset=utf-8' });
      const url = URL.createObjectURL(svgBlob);

      const img = new Image();
      img.onload = async () => {
        ctx.drawImage(img, 0, 0, bbox.width, bbox.height);
        URL.revokeObjectURL(url);

        // 使用 Tauri 剪贴板 API
        const dataUrl = canvas.toDataURL('image/png');
        const base64Data = dataUrl.replace(/^data:image\/png;base64,/, '');
        
        try {
          const { writeImage } = await import('@tauri-apps/plugin-clipboard-manager');
          // 将 base64 转换为 Uint8Array
          const binaryString = atob(base64Data);
          const bytes = new Uint8Array(binaryString.length);
          for (let i = 0; i < binaryString.length; i++) {
            bytes[i] = binaryString.charCodeAt(i);
          }
          await writeImage(bytes);
          console.log('Image copied to clipboard successfully');
        } catch (tauriErr) {
          console.error('Tauri clipboard failed, trying web API:', tauriErr);
          // 回退到 Web API
          canvas.toBlob((blob) => {
            if (blob) {
              navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
            }
          });
        }
      };
      img.src = url;
    } catch (err) {
      console.error('Failed to copy diagram:', err);
    }
  };


  // 复制源码
  const handleCopySource = () => {
    navigator.clipboard.writeText(code);
  };

  // 右键菜单
  const handleContextMenu = (e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    const rect = containerRef.current?.getBoundingClientRect();
    if (rect) {
      setContextMenuPos({
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      });
    }
  };

  return (
    <div className="relative group">
      <div 
        ref={containerRef}
        className="my-4 p-4 bg-gray-50 dark:bg-gray-900 rounded-lg overflow-x-auto"
        onContextMenu={handleContextMenu}
      >
        <div dangerouslySetInnerHTML={{ __html: svg }} />
      </div>
      
      {contextMenuPos && (
        <InlineContextMenu
          position={contextMenuPos}
          onClose={() => setContextMenuPos(null)}
          items={[
            {
              icon: <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>,
              label: '复制图片',
              onClick: handleCopyImage,
            },
            {
              icon: <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>,
              label: '复制源码',
              onClick: handleCopySource,
            },
            {
              icon: <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" /></svg>,
              label: '保存图片',
              onClick: handleSaveSvg,
            },
          ]}
        />
      )}
    </div>
  );
};

// 图片预览组件
const ImagePreview: React.FC<{ src: string; alt?: string }> = ({ src, alt }) => {
  const [isLoading, setIsLoading] = useState(true);
  const [hasError, setHasError] = useState(false);
  const [contextMenuPos, setContextMenuPos] = useState<{ x: number; y: number } | null>(null);
  const containerRef = useRef<HTMLSpanElement>(null);

  const handleLoad = useCallback(() => {
    setIsLoading(false);
  }, []);

  const handleError = useCallback(() => {
    setIsLoading(false);
    setHasError(true);
  }, []);

  // 保存图片
  const handleSaveImage = useCallback(async () => {
    try {
      const urlParts = src.split('/');
      let filename = urlParts[urlParts.length - 1] || 'image';
      filename = filename.split('?')[0];
      if (!filename.includes('.')) {
        filename += '.png';
      }

      const response = await fetch(src);
      const blob = await response.blob();
      
      const url = URL.createObjectURL(blob);
      const link = document.createElement('a');
      link.href = url;
      link.download = filename;
      document.body.appendChild(link);
      link.click();
      document.body.removeChild(link);
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error('Failed to save image:', err);
      window.open(src, '_blank');
    }
  }, [src]);

  // 复制图片
  const handleCopyImage = useCallback(async () => {
    try {
      const response = await fetch(src);
      const blob = await response.blob();
      await navigator.clipboard.write([
        new ClipboardItem({
          [blob.type]: blob,
        }),
      ]);
    } catch (err) {
      console.error('Failed to copy image:', err);
    }
  }, [src]);

  // 复制图片链接
  const handleCopyLink = useCallback(() => {
    navigator.clipboard.writeText(src);
  }, [src]);

  // 在新标签页打开
  const handleOpenInNewTab = useCallback(() => {
    window.open(src, '_blank');
  }, [src]);

  // 右键菜单
  const handleContextMenu = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    const rect = containerRef.current?.getBoundingClientRect();
    if (rect) {
      setContextMenuPos({
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      });
    }
  }, []);

  if (hasError) {
    return (
      <div className="my-3 p-4 bg-gray-100 dark:bg-gray-800 rounded-lg text-center text-gray-500">
        <svg className="w-8 h-8 mx-auto mb-2" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
        </svg>
        <span className="text-sm">图片加载失败</span>
        {alt && <div className="text-xs mt-1 text-gray-400">{alt}</div>}
      </div>
    );
  }

  return (
    <>
      <span 
        ref={containerRef}
        className="inline-block my-3 relative group" 
        onContextMenu={handleContextMenu}
      >
        {isLoading && (
          <div className="absolute inset-0 bg-gray-100 dark:bg-gray-800 rounded-lg animate-pulse flex items-center justify-center">
            <svg className="w-6 h-6 text-gray-400 animate-spin" fill="none" viewBox="0 0 24 24">
              <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
              <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
            </svg>
          </div>
        )}
        <img
          src={src}
          alt={alt || ""}
          onLoad={handleLoad}
          onError={handleError}
          className={`max-w-full h-auto rounded-lg transition-opacity ${isLoading ? 'opacity-0' : 'opacity-100'}`}
          style={{ maxHeight: '400px' }}
        />

        {contextMenuPos && (
          <InlineContextMenu
            position={contextMenuPos}
            onClose={() => setContextMenuPos(null)}
            items={[
              {
                icon: <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10 6H6a2 2 0 00-2 2v10a2 2 0 002 2h10a2 2 0 002-2v-4M14 4h6m0 0v6m0-6L10 14" /></svg>,
                label: '新标签页打开',
                onClick: handleOpenInNewTab,
              },
              { icon: null, label: '', onClick: () => {}, divider: true },
              {
                icon: <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>,
                label: '复制图片',
                onClick: handleCopyImage,
              },
              {
                icon: <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 16H6a2 2 0 01-2-2V6a2 2 0 012-2h8a2 2 0 012 2v2m-6 12h8a2 2 0 002-2v-8a2 2 0 00-2-2h-8a2 2 0 00-2 2v8a2 2 0 002 2z" /></svg>,
                label: '复制链接',
                onClick: handleCopyLink,
              },
              {
                icon: <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" /></svg>,
                label: '保存图片',
                onClick: handleSaveImage,
              },
            ]}
          />
        )}
      </span>
    </>
  );
};

// Helper to recursively find and remove cursor marker from React children
const processChildrenForCursor = (children: React.ReactNode): { nodes: React.ReactNode, hasCursor: boolean } => {
  let hasCursor = false;

  const traverse = (node: React.ReactNode): React.ReactNode => {
    if (typeof node === 'string') {
      if (node.includes('![cursor](cursor-marker)')) {
        hasCursor = true;
        return node.replace('![cursor](cursor-marker)', '');
      }
      return node;
    }

    if (Array.isArray(node)) {
      return React.Children.map(node, traverse);
    }

    if (React.isValidElement(node)) {
      const elementProps = node.props as { children?: React.ReactNode };
      return React.cloneElement(node, {
        children: traverse(elementProps.children) as React.ReactNode
      } as any);
    }

    return node;
  };

  const nodes = traverse(children);
  return { nodes, hasCursor };
};

export const MarkdownRenderer: React.FC<MarkdownRendererProps> = memo(({
  content,
  isStreaming = false,
}) => {
  const { t } = useI18n();
  // Inject a special marker for the cursor when streaming
  const displayContent = isStreaming ? `${content}![cursor](cursor-marker)` : content;

  return (
    <div className="markdown-body prose prose-sm dark:prose-invert max-w-none">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeHighlight]}
        components={{
          // Handle the cursor image marker and regular images
          img({ src, alt }) {
            if (src === "cursor-marker" && alt === "cursor") {
              return <Cursor />;
            }
            return <ImagePreview src={src || ""} alt={alt} />;
          },
          // Custom code block styling
          code({ node, className, children, ...props }) {
            const match = /language-(\w+)/.exec(className || "");
            const isInline = !match && !className;
            const language = match?.[1]?.toLowerCase();
            
            // Cast children to React.ReactNode for type compatibility
            const childrenNode = children as React.ReactNode;
            
            // Handle cursor in code blocks
            const { nodes: processedChildren, hasCursor } = processChildrenForCursor(childrenNode);

            // 提取代码文本
            const getCodeText = (node: React.ReactNode): string => {
              if (typeof node === 'string') return node;
              if (Array.isArray(node)) return node.map(getCodeText).join('');
              if (React.isValidElement(node)) {
                const elementProps = (node as React.ReactElement).props as { children?: React.ReactNode };
                return getCodeText(elementProps.children);
              }
              return '';
            };
            const codeText = getCodeText(childrenNode).replace('![cursor](cursor-marker)', '').trim();

            // Mermaid 图表渲染
            if (language === 'mermaid') {
              return <MermaidDiagram code={codeText} />;
            }

            if (isInline) {
              return (
                <code
                  className="px-1.5 py-0.5 rounded bg-zinc-200 dark:bg-zinc-700 text-pink-600 dark:text-pink-400 text-sm font-mono"
                  {...props}
                >
                  {processedChildren}
                  {hasCursor && <Cursor />}
                </code>
              );
            }
            
            return (
              <div className="relative group my-4">
                {match && (
                  <div className="absolute top-0 right-0 px-2 py-1 text-xs text-gray-400 dark:text-gray-500 bg-gray-100 dark:bg-gray-900 rounded-bl rounded-tr-lg">
                    {match[1]}
                  </div>
                )}
                <pre className="!bg-gray-100 dark:!bg-gray-900 rounded-lg p-4 overflow-x-auto">
                  <code className={`${className} text-sm`} {...props}>
                    {processedChildren}
                    {hasCursor && <Cursor />}
                  </code>
                </pre>
                <button
                  className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity px-2 py-1 text-xs bg-gray-200 dark:bg-gray-700 hover:bg-gray-300 dark:hover:bg-gray-600 rounded"
                  onClick={() => {
                    // We need to get the raw text for copying, stripping the cursor marker
                    // Since children might be a tree of nodes now, getting raw text is harder.
                    // But we can just use the node's text content if available, or traverse again.
                    // A simpler way is to use the `node` prop which contains the raw value?
                    // ReactMarkdown passes `node` which is the AST node.
                    // node.children[0].value might have the text if it's a simple code block.
                    // But with highlighting, it's complex.
                    // Let's try to extract text from children recursively.
                    
                    const getText = (node: React.ReactNode): string => {
                        if (typeof node === 'string') return node;
                        if (Array.isArray(node)) return node.map(getText).join('');
                        if (React.isValidElement(node)) {
                          const elementProps = (node as React.ReactElement).props as { children?: React.ReactNode };
                          return getText(elementProps.children);
                        }
                        return '';
                    };
                    const text = getText(childrenNode).replace('![cursor](cursor-marker)', '');
                    navigator.clipboard.writeText(text.replace(/\n$/, ""));
                  }}
                >
                  {t.common.copy}
                </button>
              </div>
            );
          },
          // Custom link styling
          a({ node, children, href, ...props }) {
            return (
              <a
                href={href}
                target="_blank"
                rel="noopener noreferrer"
                className="text-blue-600 dark:text-blue-400 hover:underline"
                {...props}
              >
                {children}
              </a>
            );
          },
          // Custom paragraph
          p({ node, children, ...props }) {
            return (
              <p className="mb-3 last:mb-0 leading-relaxed" {...props}>
                {children}
              </p>
            );
          },
          // Custom list styling
          ul({ node, children, ...props }) {
            return (
              <ul className="list-disc list-inside mb-3 space-y-1" {...props}>
                {children}
              </ul>
            );
          },
          ol({ node, children, ...props }) {
            return (
              <ol className="list-decimal list-inside mb-3 space-y-1" {...props}>
                {children}
              </ol>
            );
          },
          // Custom heading styling
          h1({ node, children, ...props }) {
            return (
              <h1 className="text-xl font-bold mb-3 mt-4 first:mt-0" {...props}>
                {children}
              </h1>
            );
          },
          h2({ node, children, ...props }) {
            return (
              <h2 className="text-lg font-bold mb-2 mt-3 first:mt-0" {...props}>
                {children}
              </h2>
            );
          },
          h3({ node, children, ...props }) {
            return (
              <h3 className="text-base font-bold mb-2 mt-3 first:mt-0" {...props}>
                {children}
              </h3>
            );
          },
          // Custom blockquote
          blockquote({ node, children, ...props }) {
            return (
              <blockquote
                className="border-l-4 border-gray-300 dark:border-gray-600 pl-4 my-3 italic text-gray-600 dark:text-gray-400"
                {...props}
              >
                {children}
              </blockquote>
            );
          },
          // Custom table
          table({ node, children, ...props }) {
            return (
              <div className="overflow-x-auto my-3">
                <table className="min-w-full border-collapse border border-gray-300 dark:border-gray-600" {...props}>
                  {children}
                </table>
              </div>
            );
          },
          th({ node, children, ...props }) {
            return (
              <th className="border border-gray-300 dark:border-gray-600 px-3 py-2 bg-gray-100 dark:bg-gray-800 font-semibold text-left" {...props}>
                {children}
              </th>
            );
          },
          td({ node, children, ...props }) {
            return (
              <td className="border border-gray-300 dark:border-gray-600 px-3 py-2" {...props}>
                {children}
              </td>
            );
          },
          // Horizontal rule
          hr({ node, ...props }) {
            return <hr className="my-4 border-gray-300 dark:border-gray-600" {...props} />;
          },
          // Math block ($$...$$) with context menu
          div({ className, children, ...props }) {
            // KaTeX 渲染的数学公式块会有 math-display 类
            if (className?.includes('math-display')) {
              const handleContextMenu = (e: React.MouseEvent) => {
                e.preventDefault();
              };
              
              return (
                <div 
                  className={className} 
                  onContextMenu={handleContextMenu}
                  {...props}
                >
                  {children}
                </div>
              );
            }
            return <div className={className} {...props}>{children}</div>;
          },
          // Inline math ($...$) with context menu
          span({ className, children, ...props }) {
            if (className?.includes('math-inline')) {
              const handleContextMenu = (e: React.MouseEvent) => {
                 e.preventDefault();
              };

              return (
                <span 
                  className={className}
                  onContextMenu={handleContextMenu}
                  {...props}
                >
                  {children}
                </span>
              );
            }
            return <span className={className} {...props}>{children}</span>;
          },
        }}
      >
        {displayContent}
      </ReactMarkdown>
    </div>
  );
});
