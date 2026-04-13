import { Search, Settings } from "lucide-react";
import type { Environment } from "../../lib/types";

interface TopBarProps {
  environments: Environment[];
  onEnvChange: (id: string | null) => void;
  onSearch: () => void;
  onSettings: () => void;
  errorMessage: string | null;
  statusMessage: string | null;
  isLoading: boolean;
}

export function TopBar({
  environments, onEnvChange, onSearch, onSettings,
  errorMessage, statusMessage, isLoading,
}: TopBarProps) {
  const activeEnv = environments.find(e => e.is_active);

  return (
    <div className="flex items-center h-12 px-4 gap-3 border-b border-[var(--border)] flex-shrink-0 bg-[#161616]">
      {/* Environment chips */}
      {environments.length > 0 && (
        <div className="flex items-center gap-1.5 flex-shrink-0">
          {environments.map(env => {
            const isActive = env.is_active;
            return (
              <button
                key={env.id}
                onClick={() => onEnvChange(isActive ? null : env.id)}
                className={`px-2.5 py-1 rounded-full text-xs font-medium transition-all flex-shrink-0 border ${
                  isActive
                    ? "bg-blue-500 text-white border-blue-500"
                    : "bg-transparent text-gray-500 border-gray-700 hover:border-gray-500 hover:text-gray-300"
                }`}
              >
                {env.name}
              </button>
            );
          })}
        </div>
      )}

      {/* Search trigger — faux input */}
      <button
        onClick={onSearch}
        className="flex-1 max-w-md mx-2 flex items-center justify-between bg-[#0d0d0d] border border-[var(--border)] hover:border-gray-600 rounded-md px-3 py-1.5 text-sm text-gray-500 transition-colors group"
      >
        <div className="flex items-center gap-2 overflow-hidden">
          <Search size={14} className="text-gray-600 group-hover:text-gray-400 shrink-0" />
          <span className="truncate text-xs">Search requests…</span>
        </div>
        <div className="flex items-center gap-1 shrink-0 ml-2">
          <span className="text-[10px] font-mono bg-[#1a1a1a] text-gray-600 px-1.5 py-0.5 rounded border border-gray-700">⌘</span>
          <span className="text-[10px] font-mono bg-[#1a1a1a] text-gray-600 px-1.5 py-0.5 rounded border border-gray-700">K</span>
        </div>
      </button>

      <div className="flex-1" />

      {/* Status messages */}
      {isLoading && (
        <div className="text-xs animate-pulse flex-shrink-0 text-blue-400">
          Sending…
        </div>
      )}
      {errorMessage && (
        <div className="text-xs truncate max-w-xs flex-shrink-0 text-red-400" title={errorMessage}>
          {errorMessage}
        </div>
      )}
      {statusMessage && (
        <div className="text-xs flex-shrink-0 text-gray-500">
          {statusMessage}
        </div>
      )}

      {/* Settings */}
      <button
        onClick={onSettings}
        className="p-2 rounded-md text-gray-500 hover:text-gray-300 hover:bg-[#1a1a1a] transition-colors flex-shrink-0"
      >
        <Settings size={16} />
      </button>
    </div>
  );
}
