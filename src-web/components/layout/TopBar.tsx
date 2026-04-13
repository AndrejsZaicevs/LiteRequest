import { useState, useRef, useEffect } from "react";
import { Search, Settings, ChevronDown } from "lucide-react";
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
  const [envOpen, setEnvOpen] = useState(false);
  const envRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const handler = (e: MouseEvent) => {
      if (envRef.current && !envRef.current.contains(e.target as Node)) setEnvOpen(false);
    };
    document.addEventListener("mousedown", handler);
    return () => document.removeEventListener("mousedown", handler);
  }, []);

  return (
    <div
      className="flex items-center h-14 px-4 gap-3 border-b flex-shrink-0"
      style={{ background: "var(--surface-1)", borderColor: "var(--border)" }}
    >
      <div className="font-bold text-lg tracking-tight mr-4" style={{ color: "var(--accent)" }}>
        LiteRequest
      </div>

      {/* Environment dropdown */}
      {environments.length > 0 && (
        <div className="relative" ref={envRef}>
          <button
            onClick={() => setEnvOpen(!envOpen)}
            className="px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center gap-2"
            style={{
              background: "var(--surface-2)",
              color: "var(--text-secondary)",
              border: "1px solid var(--border)",
            }}
          >
            <span style={{ color: "var(--text-muted)" }}>Env:</span>
            <span style={{ color: activeEnv ? "var(--accent)" : "var(--text-muted)" }}>
              {activeEnv?.name ?? "No Environment"}
            </span>
            <ChevronDown size={14} style={{ color: "var(--text-muted)" }} />
          </button>
          {envOpen && (
            <div
              className="absolute top-full left-0 mt-1 rounded-md shadow-lg z-50 overflow-hidden"
              style={{ background: "var(--surface-2)", border: "1px solid var(--border)", minWidth: 180 }}
            >
              <button
                className="w-full text-left px-4 py-2.5 text-sm transition-colors hover:bg-[var(--surface-3)]"
                style={{ color: !activeEnv ? "var(--accent)" : "var(--text-secondary)" }}
                onClick={() => { onEnvChange(null); setEnvOpen(false); }}
              >
                No Environment
              </button>
              {environments.map(env => (
                <button
                  key={env.id}
                  className="w-full text-left px-4 py-2.5 text-sm transition-colors hover:bg-[var(--surface-3)]"
                  style={{ color: env.is_active ? "var(--accent)" : "var(--text-secondary)" }}
                  onClick={() => { onEnvChange(env.is_active ? null : env.id); setEnvOpen(false); }}
                >
                  {env.name}
                </button>
              ))}
            </div>
          )}
        </div>
      )}

      <div className="flex-1" />

      {/* Status messages */}
      {isLoading && (
        <div className="text-xs animate-pulse" style={{ color: "var(--accent)" }}>
          Sending…
        </div>
      )}
      {errorMessage && (
        <div className="text-xs truncate max-w-xs" style={{ color: "var(--danger)" }} title={errorMessage}>
          {errorMessage}
        </div>
      )}
      {statusMessage && (
        <div className="text-xs" style={{ color: "var(--text-muted)" }}>
          {statusMessage}
        </div>
      )}

      {/* Search button */}
      <button
        onClick={onSearch}
        className="px-4 py-2 rounded-md text-sm hover:opacity-80 transition-opacity flex items-center gap-2"
        style={{ background: "var(--surface-2)", border: "1px solid var(--border)" }}
      >
        <Search size={16} style={{ color: "var(--text-secondary)" }} />
        <span style={{ color: "var(--text-muted)" }}>Ctrl+K</span>
      </button>

      {/* Settings */}
      <button
        onClick={onSettings}
        className="px-4 py-2 rounded-md text-sm hover:opacity-80 transition-opacity"
        style={{ background: "var(--surface-2)", border: "1px solid var(--border)" }}
      >
        <Settings size={16} style={{ color: "var(--text-secondary)" }} />
      </button>
    </div>
  );
}
