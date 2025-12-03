import React from "react";

interface AvatarProps {
  role: "user" | "assistant" | "system";
  src?: string;
  alt?: string;
  className?: string;
}

export const Avatar: React.FC<AvatarProps> = ({ role, src, alt, className = "" }) => {
  const isUser = role === "user";
  const isSystem = role === "system";

  const getInitials = () => {
    if (role === "user") return "ME";
    if (role === "assistant") return "AI";
    return "SYS";
  };

  const getBgColor = () => {
    if (isUser) return "bg-primary-100 text-primary-600 dark:bg-primary-900 dark:text-primary-300";
    if (isSystem) return "bg-gray-100 text-gray-600 dark:bg-gray-800 dark:text-gray-400";
    return "bg-purple-100 text-purple-600 dark:bg-purple-900 dark:text-purple-300";
  };

  return (
    <div
      className={`
        w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold select-none
        ${getBgColor()}
        ${className}
      `}
    >
      {src ? (
        <img src={src} alt={alt || role} className="w-full h-full rounded-full object-cover" />
      ) : (
        <span>{getInitials()}</span>
      )}
    </div>
  );
};
