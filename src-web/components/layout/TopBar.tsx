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
      className="flex items-center h-12 px-4 gap-3 border-b flex-shrink-0"
      style={{ background: "var(--surface-1)", borderColor: "var(--border)" }}
    >
      <div className="font-bold text-base tracking-tight mr-4" style={{ color: "var(--accent)" }}>
        LiteRequest
      </div>

      {/* Environment chips */}
      <div className="flex items-center gap-1.5">
        {environments.map(env => (
          <button
            key={env.id}
            onClick={() => onEnvChange(env.is_active ? null : env.id)}
            className="px-3 py-1.5 rounded-md text-xs font-medium transition-colors"
            style={{
              background: env.is_active ? "var(--accent)" : "var(--surface-2)",
              color: env.is_active ? "#fff" : "var(--text-secondary)",
              border: `1px solid ${env.is_active ? "var(--accent)" : "var(--border)"}`,
            }}
          >
            {env.name}
          </button>
        ))}
      </div>

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
        className="px-3 py-1.5 rounded-md text-xs hover:opacity-80 transition-opacity flex items-center gap-1.5"
        style={{ background: "var(--surface-2)", border: "1px solid var(--border)" }}
      >
        🔍 <span style={{ color: "var(--text-muted)" }}>Ctrl+K</span>
      </button>

      {/* Settings */}
      <button
        onClick={onSettings}
        className="px-3 py-1.5 rounded-md text-sm hover:opacity-80 transition-opacity"
        style={{ background: "var(--surface-2)", border: "1px solid var(--border)" }}
      >
        ⚙️
      </button>
    </div>
  );
}
