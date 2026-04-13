import { useState, useEffect } from "react";
import type { Environment, EnvVariable } from "../../lib/types";
import * as api from "../../lib/api";

interface AppSettingsProps {
  environments: Environment[];
  onUpdate: () => void;
}

export function AppSettings({ environments, onUpdate }: AppSettingsProps) {
  const [tab, setTab] = useState<"environments" | "general">("environments");
  const [selectedEnv, setSelectedEnv] = useState<string | null>(environments[0]?.id ?? null);
  const [envVars, setEnvVars] = useState<EnvVariable[]>([]);

  useEffect(() => {
    if (!selectedEnv) return;
    api.listEnvVariables(selectedEnv).then(setEnvVars).catch(console.error);
  }, [selectedEnv]);

  const addEnvironment = async () => {
    const env: Environment = {
      id: crypto.randomUUID(),
      name: "New Environment",
      is_active: false,
      created_at: new Date().toISOString(),
    };
    await api.insertEnvironment(env);
    onUpdate();
    setSelectedEnv(env.id);
  };

  const deleteEnvironment = async (id: string) => {
    await api.deleteEnvironment(id);
    onUpdate();
    setSelectedEnv(null);
  };

  const addEnvVar = async () => {
    if (!selectedEnv) return;
    const v: EnvVariable = {
      id: crypto.randomUUID(),
      environment_id: selectedEnv,
      key: "NEW_VAR",
      value: "",
      is_secret: false,
    };
    await api.insertEnvVariable(v);
    const vars = await api.listEnvVariables(selectedEnv);
    setEnvVars(vars);
  };

  const updateEnvVar = async (v: EnvVariable) => {
    await api.updateEnvVariable(v);
    if (selectedEnv) {
      const vars = await api.listEnvVariables(selectedEnv);
      setEnvVars(vars);
    }
  };

  const deleteEnvVar = async (id: string) => {
    await api.deleteEnvVariable(id);
    if (selectedEnv) {
      const vars = await api.listEnvVariables(selectedEnv);
      setEnvVars(vars);
    }
  };

  return (
    <div className="h-full flex flex-col overflow-hidden" style={{ background: "var(--surface-0)" }}>
      {/* Tab bar */}
      <div className="flex items-center border-b px-4" style={{ borderColor: "var(--border)", background: "var(--surface-1)" }}>
        {(["environments", "general"] as const).map(t => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className="px-3 py-2 text-xs capitalize"
            style={{
              color: tab === t ? "var(--accent)" : "var(--text-muted)",
              borderBottom: tab === t ? "2px solid var(--accent)" : "2px solid transparent",
            }}
          >
            {t}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        {tab === "environments" && (
          <div>
            <div className="flex items-center gap-2 mb-3">
              <h3 className="text-sm font-semibold">Environments</h3>
              <button
                onClick={addEnvironment}
                className="text-xs px-2 py-0.5 rounded"
                style={{ background: "var(--surface-2)", border: "1px solid var(--border)", color: "var(--accent)" }}
              >
                + New
              </button>
            </div>

            <div className="flex gap-4">
              {/* Env list */}
              <div className="w-48 flex-shrink-0">
                {environments.map(env => (
                  <button
                    key={env.id}
                    onClick={() => setSelectedEnv(env.id)}
                    className="w-full text-left px-3 py-1.5 text-xs rounded mb-1 flex items-center justify-between"
                    style={{
                      background: selectedEnv === env.id ? "var(--surface-2)" : "transparent",
                      color: "var(--text-primary)",
                    }}
                  >
                    <span>{env.name}</span>
                    {env.is_active && (
                      <span className="text-[10px] px-1 rounded" style={{ background: "var(--accent)", color: "#fff" }}>
                        active
                      </span>
                    )}
                  </button>
                ))}
              </div>

              {/* Env variables */}
              {selectedEnv && (
                <div className="flex-1">
                  <div className="flex items-center justify-between mb-2">
                    <h4 className="text-xs font-semibold" style={{ color: "var(--text-secondary)" }}>
                      Variables
                    </h4>
                    <div className="flex gap-1">
                      <button onClick={addEnvVar} className="text-xs px-2 py-0.5 rounded"
                        style={{ background: "var(--surface-2)", border: "1px solid var(--border)", color: "var(--accent)" }}>
                        + Add
                      </button>
                      <button onClick={() => deleteEnvironment(selectedEnv)} className="text-xs px-2 py-0.5 rounded"
                        style={{ background: "var(--surface-2)", border: "1px solid var(--border)", color: "var(--danger)" }}>
                        Delete Env
                      </button>
                    </div>
                  </div>

                  <div className="text-xs">
                    {envVars.map(v => (
                      <div key={v.id} className="flex items-center border-b" style={{ borderColor: "var(--border)" }}>
                        <input
                          value={v.key}
                          onChange={(e) => updateEnvVar({ ...v, key: e.target.value })}
                          className="flex-1 bg-transparent border-none outline-none px-2 py-1.5 font-mono"
                          style={{ color: "var(--text-primary)", borderRight: "1px solid var(--border)" }}
                        />
                        <input
                          value={v.value}
                          type={v.is_secret ? "password" : "text"}
                          onChange={(e) => updateEnvVar({ ...v, value: e.target.value })}
                          className="flex-1 bg-transparent border-none outline-none px-2 py-1.5 font-mono"
                          style={{ color: "var(--text-primary)" }}
                        />
                        <button
                          onClick={() => updateEnvVar({ ...v, is_secret: !v.is_secret })}
                          className="w-6 flex-shrink-0 text-center"
                          style={{ color: "var(--text-muted)" }}
                          title={v.is_secret ? "Show" : "Hide"}
                        >
                          {v.is_secret ? "🔒" : "👁"}
                        </button>
                        <button
                          onClick={() => deleteEnvVar(v.id)}
                          className="w-6 flex-shrink-0 hover:opacity-80"
                          style={{ color: "var(--text-muted)" }}
                        >
                          ×
                        </button>
                      </div>
                    ))}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {tab === "general" && (
          <div className="text-xs" style={{ color: "var(--text-muted)" }}>
            <h3 className="text-sm font-semibold mb-3" style={{ color: "var(--text-primary)" }}>General Settings</h3>
            <p>General app settings will appear here (client certificates, default headers, etc.)</p>
          </div>
        )}
      </div>
    </div>
  );
}
