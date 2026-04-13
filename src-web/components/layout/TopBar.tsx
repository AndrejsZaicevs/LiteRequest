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
    <div
      className="flex items-center h-14 px-4 gap-2 border-b flex-shrink-0 overflow-x-auto"
      style={{ background: "var(--surface-1)", borderColor: "var(--border)" }}
    >
      {/* Environment chips — one per env, click to activate/deactivate */}
      {environments.length > 0 && (
        <div className="flex items-center gap-2 flex-shrink-0">
          <span className="text-xs font-medium mr-1" style={{ color: "var(--text-muted)" }}>Env</span>
          {environments.map(env => {
            const isActive = env.is_active;
            return (
              <button
                key={env.id}
                onClick={() => onEnvChange(isActive ? null : env.id)}
                className="px-3 py-1.5 rounded-full text-sm font-medium transition-all flex-shrink-0"
                style={{
                  background: isActive ? "var(--accent)" : "var(--surface-2)",
                  color: isActive ? "#fff" : "var(--text-secondary)",
                  border: isActive ? "1px solid var(--accent)" : "1px solid var(--border)",
                  opacity: isActive ? 1 : 0.85,
                }}
              >
                {env.name}
              </button>
            );
          })}
        </div>
      )}

      <div className="flex-1" />

      {/* Status messages */}
      {isLoading && (
        <div className="text-xs animate-pulse flex-shrink-0" style={{ color: "var(--accent)" }}>
          Sending…
        </div>
      )}
      {errorMessage && (
        <div className="text-xs truncate max-w-xs flex-shrink-0" style={{ color: "var(--danger)" }} title={errorMessage}>
          {errorMessage}
        </div>
      )}
      {statusMessage && (
        <div className="text-xs flex-shrink-0" style={{ color: "var(--text-muted)" }}>
          {statusMessage}
        </div>
      )}

      {/* Search button */}
      <button
        onClick={onSearch}
        className="px-4 py-2 rounded-md text-sm hover:opacity-80 transition-opacity flex items-center gap-2 flex-shrink-0"
        style={{ background: "var(--surface-2)", border: "1px solid var(--border)" }}
      >
        <Search size={16} style={{ color: "var(--text-secondary)" }} />
        <span style={{ color: "var(--text-muted)" }}>Ctrl+K</span>
      </button>

      {/* Settings */}
      <button
        onClick={onSettings}
        className="px-4 py-2 rounded-md text-sm hover:opacity-80 transition-opacity flex-shrink-0"
        style={{ background: "var(--surface-2)", border: "1px solid var(--border)" }}
      >
        <Settings size={16} style={{ color: "var(--text-secondary)" }} />
      </button>
    </div>
  );
}
