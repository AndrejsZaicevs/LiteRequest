import { useState, useEffect } from "react";
import type { Environment, EnvVariable, KeyValuePair, ClientCertEntry } from "../../lib/types";
import * as api from "../../lib/api";
import { KvTable } from "../inspector/KvTable";

interface AppSettingsProps {
  environments: Environment[];
  onUpdate: () => void;
}

export function AppSettings({ environments, onUpdate }: AppSettingsProps) {
  const [tab, setTab] = useState<"environments" | "headers" | "variables" | "certificates">("environments");
  const [selectedEnv, setSelectedEnv] = useState<string | null>(environments[0]?.id ?? null);
  const [envVars, setEnvVars] = useState<EnvVariable[]>([]);
  const [renamingEnv, setRenamingEnv] = useState<string | null>(null);
  const [renameValue, setRenameValue] = useState("");

  // App-wide settings
  const [defaultHeaders, setDefaultHeaders] = useState<KeyValuePair[]>([]);
  const [appWideVars, setAppWideVars] = useState<KeyValuePair[]>([]);
  const [clientCerts, setClientCerts] = useState<ClientCertEntry[]>([]);

  useEffect(() => {
    if (!selectedEnv) return;
    api.listEnvVariables(selectedEnv).then(setEnvVars).catch(console.error);
  }, [selectedEnv]);

  // Load app-wide settings on mount
  useEffect(() => {
    api.getAppSetting("default_headers").then(v => {
      if (v) { try { setDefaultHeaders(JSON.parse(v)); } catch {} }
    }).catch(() => {});
    api.getAppSetting("app_variables").then(v => {
      if (v) { try { setAppWideVars(JSON.parse(v)); } catch {} }
    }).catch(() => {});
    api.getAppSetting("client_certs").then(v => {
      if (v) { try { setClientCerts(JSON.parse(v)); } catch {} }
    }).catch(() => {});
  }, []);

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
    setRenamingEnv(env.id);
    setRenameValue(env.name);
  };

  const deleteEnvironment = async (id: string) => {
    await api.deleteEnvironment(id);
    onUpdate();
    if (selectedEnv === id) setSelectedEnv(null);
  };

  const handleRenameEnv = async () => {
    if (!renamingEnv || !renameValue.trim()) { setRenamingEnv(null); return; }
    await api.renameEnvironment(renamingEnv, renameValue.trim());
    setRenamingEnv(null);
    onUpdate();
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

  const saveDefaultHeaders = async (headers: KeyValuePair[]) => {
    setDefaultHeaders(headers);
    await api.setAppSetting("default_headers", JSON.stringify(headers));
  };

  const saveAppVars = async (vars: KeyValuePair[]) => {
    setAppWideVars(vars);
    await api.setAppSetting("app_variables", JSON.stringify(vars));
  };

  const addCert = () => {
    setClientCerts([...clientCerts, {
      enabled: true,
      host: "",
      cert_type: "Pem",
      cert_path: "",
      key_path: "",
      ca_path: "",
      passphrase: "",
    }]);
  };

  const updateCert = (i: number, patch: Partial<ClientCertEntry>) => {
    const next = [...clientCerts];
    next[i] = { ...next[i], ...patch };
    setClientCerts(next);
    api.setAppSetting("client_certs", JSON.stringify(next)).catch(console.error);
  };

  const removeCert = (i: number) => {
    const next = clientCerts.filter((_, idx) => idx !== i);
    setClientCerts(next);
    api.setAppSetting("client_certs", JSON.stringify(next)).catch(console.error);
  };

  const tabs = ["environments", "headers", "variables", "certificates"] as const;

  return (
    <div className="h-full flex flex-col overflow-hidden" style={{ background: "var(--surface-0)" }}>
      {/* Header */}
      <div className="px-6 pt-5 pb-3 flex-shrink-0">
        <h2 className="text-lg font-semibold flex items-center gap-2">
          <span>⚙️</span> Settings
        </h2>
        <p className="text-[11px] mt-1" style={{ color: "var(--text-muted)" }}>
          Global application settings — environments, default headers, variables, and certificates
        </p>
      </div>

      {/* Tab bar */}
      <div
        className="flex items-center gap-1 px-6 border-b flex-shrink-0"
        style={{ borderColor: "var(--border)" }}
      >
        {tabs.map(t => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className="px-3 py-2 text-[11px] font-medium capitalize transition-colors"
            style={{
              color: tab === t ? "var(--accent)" : "var(--text-muted)",
              borderBottom: tab === t ? "2px solid var(--accent)" : "2px solid transparent",
            }}
          >
            {t}
          </button>
        ))}
      </div>

      <div className="flex-1 overflow-y-auto px-6 py-4">
        {/* Environments tab */}
        {tab === "environments" && (
          <div>
            <div className="flex items-center justify-between mb-4">
              <span className="text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
                Manage environments and their variables
              </span>
              <button onClick={addEnvironment} className="btn-pill accent">+ New Environment</button>
            </div>

            <div className="flex gap-5">
              {/* Env list */}
              <div className="w-52 flex-shrink-0" style={{ border: "1px solid var(--border)", borderRadius: 4, overflow: "hidden" }}>
                {environments.map(env => (
                  <div
                    key={env.id}
                    className="flex items-center px-3 py-2 cursor-pointer transition-colors"
                    style={{
                      background: selectedEnv === env.id ? "var(--surface-2)" : "transparent",
                      borderBottom: "1px solid var(--border-subtle)",
                    }}
                    onClick={() => setSelectedEnv(env.id)}
                    onDoubleClick={() => { setRenamingEnv(env.id); setRenameValue(env.name); }}
                    onMouseEnter={(e) => { if (selectedEnv !== env.id) e.currentTarget.style.background = "var(--row-hover)"; }}
                    onMouseLeave={(e) => { e.currentTarget.style.background = selectedEnv === env.id ? "var(--surface-2)" : "transparent"; }}
                  >
                    {renamingEnv === env.id ? (
                      <input
                        value={renameValue}
                        onChange={(e) => setRenameValue(e.target.value)}
                        onBlur={handleRenameEnv}
                        onKeyDown={(e) => { if (e.key === "Enter") handleRenameEnv(); if (e.key === "Escape") setRenamingEnv(null); }}
                        className="flex-1 bg-transparent text-xs outline-none"
                        style={{ border: "none", borderBottom: "1px solid var(--accent)", borderRadius: 0, padding: "0 2px" }}
                        autoFocus
                        onClick={(e) => e.stopPropagation()}
                      />
                    ) : (
                      <span className="flex-1 text-xs truncate">{env.name}</span>
                    )}
                    {env.is_active && (
                      <span className="text-[9px] px-1.5 py-0.5 rounded font-medium ml-2 flex-shrink-0"
                        style={{ background: "var(--accent)", color: "#fff" }}>
                        active
                      </span>
                    )}
                  </div>
                ))}
                {environments.length === 0 && (
                  <div className="px-3 py-4 text-[11px] text-center" style={{ color: "var(--text-muted)" }}>
                    No environments
                  </div>
                )}
              </div>

              {/* Env variables */}
              {selectedEnv && (
                <div className="flex-1">
                  <div className="flex items-center justify-between mb-2">
                    <span className="text-xs font-medium" style={{ color: "var(--text-secondary)" }}>
                      Variables
                    </span>
                    <div className="flex gap-2">
                      <button onClick={addEnvVar} className="btn-pill accent">+ Add</button>
                      <button onClick={() => { setRenamingEnv(selectedEnv); setRenameValue(environments.find(e => e.id === selectedEnv)?.name ?? ""); }}
                        className="btn-pill">Rename</button>
                      <button onClick={() => deleteEnvironment(selectedEnv)} className="btn-pill danger">Delete</button>
                    </div>
                  </div>

                  <div style={{ border: "1px solid var(--border)", borderRadius: 4, overflow: "hidden" }}>
                    {envVars.map(v => (
                      <div key={v.id} className="kv-row">
                        <input
                          value={v.key}
                          onChange={(e) => updateEnvVar({ ...v, key: e.target.value })}
                          className="kv-cell"
                          style={{ border: "none", borderRadius: 0, padding: "4px 10px", fontWeight: 500 }}
                        />
                        <div className="kv-divider" />
                        <input
                          value={v.value}
                          type={v.is_secret ? "password" : "text"}
                          onChange={(e) => updateEnvVar({ ...v, value: e.target.value })}
                          className="kv-cell"
                          style={{ border: "none", borderRadius: 0, padding: "4px 10px" }}
                        />
                        <button
                          onClick={() => updateEnvVar({ ...v, is_secret: !v.is_secret })}
                          className="kv-action"
                          title={v.is_secret ? "Show" : "Hide"}
                        >
                          {v.is_secret ? "🔒" : "👁"}
                        </button>
                        <button onClick={() => deleteEnvVar(v.id)} className="kv-action">×</button>
                      </div>
                    ))}
                    {envVars.length === 0 && (
                      <div className="px-3 py-3 text-[11px] text-center" style={{ color: "var(--text-muted)" }}>
                        No variables — click + Add to create one
                      </div>
                    )}
                  </div>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Default Headers tab */}
        {tab === "headers" && (
          <div>
            <p className="text-[11px] mb-3" style={{ color: "var(--text-muted)" }}>
              Default headers sent with every request across all collections
            </p>
            <div style={{ border: "1px solid var(--border)", borderRadius: 4, overflow: "hidden", maxWidth: 600 }}>
              <KvTable
                rows={defaultHeaders}
                onChange={saveDefaultHeaders}
                placeholder={{ key: "Header-Name", value: "value" }}
              />
            </div>
          </div>
        )}

        {/* App-wide Variables tab */}
        {tab === "variables" && (
          <div>
            <p className="text-[11px] mb-3" style={{ color: "var(--text-muted)" }}>
              Global variables available in all collections (not environment-specific)
            </p>
            <div style={{ border: "1px solid var(--border)", borderRadius: 4, overflow: "hidden", maxWidth: 600 }}>
              <KvTable
                rows={appWideVars}
                onChange={saveAppVars}
                placeholder={{ key: "VARIABLE_NAME", value: "value" }}
              />
            </div>
          </div>
        )}

        {/* Client Certificates tab */}
        {tab === "certificates" && (
          <div>
            <div className="flex items-center justify-between mb-3">
              <p className="text-[11px]" style={{ color: "var(--text-muted)" }}>
                Client TLS certificates for mutual TLS authentication
              </p>
              <button onClick={addCert} className="btn-pill accent">+ Add Certificate</button>
            </div>

            {clientCerts.length === 0 && (
              <div className="text-[11px] p-4 rounded text-center" style={{ background: "var(--surface-1)", color: "var(--text-muted)" }}>
                No client certificates configured
              </div>
            )}

            <div className="space-y-3">
              {clientCerts.map((cert, i) => (
                <div
                  key={i}
                  className="rounded p-4"
                  style={{
                    border: "1px solid var(--border)",
                    background: "var(--surface-1)",
                    opacity: cert.enabled ? 1 : 0.5,
                  }}
                >
                  <div className="flex items-center justify-between mb-3">
                    <div className="flex items-center gap-2">
                      <button
                        onClick={() => updateCert(i, { enabled: !cert.enabled })}
                        style={{ color: cert.enabled ? "var(--accent)" : "var(--text-muted)", fontSize: 13 }}
                      >
                        {cert.enabled ? "✓" : "○"}
                      </button>
                      <span className="text-xs font-medium">Certificate {i + 1}</span>
                    </div>
                    <button onClick={() => removeCert(i)} className="btn-pill danger" style={{ padding: "2px 8px" }}>
                      Remove
                    </button>
                  </div>

                  <div className="grid grid-cols-2 gap-3">
                    <div>
                      <label className="label block">Host Pattern</label>
                      <input
                        value={cert.host}
                        onChange={(e) => updateCert(i, { host: e.target.value })}
                        placeholder="*.example.com"
                        className="w-full font-mono text-xs"
                      />
                    </div>
                    <div>
                      <label className="label block">Type</label>
                      <select
                        value={cert.cert_type}
                        onChange={(e) => updateCert(i, { cert_type: e.target.value as "Pem" | "Pkcs12" })}
                        className="text-xs"
                      >
                        <option value="Pem">PEM</option>
                        <option value="Pkcs12">PKCS#12</option>
                      </select>
                    </div>
                    <div>
                      <label className="label block">Certificate Path</label>
                      <input
                        value={cert.cert_path}
                        onChange={(e) => updateCert(i, { cert_path: e.target.value })}
                        placeholder="/path/to/cert.pem"
                        className="w-full font-mono text-xs"
                      />
                    </div>
                    <div>
                      <label className="label block">Key Path</label>
                      <input
                        value={cert.key_path}
                        onChange={(e) => updateCert(i, { key_path: e.target.value })}
                        placeholder="/path/to/key.pem"
                        className="w-full font-mono text-xs"
                      />
                    </div>
                    <div>
                      <label className="label block">CA Path (optional)</label>
                      <input
                        value={cert.ca_path}
                        onChange={(e) => updateCert(i, { ca_path: e.target.value })}
                        placeholder="/path/to/ca.pem"
                        className="w-full font-mono text-xs"
                      />
                    </div>
                    <div>
                      <label className="label block">Passphrase (optional)</label>
                      <input
                        type="password"
                        value={cert.passphrase}
                        onChange={(e) => updateCert(i, { passphrase: e.target.value })}
                        className="w-full font-mono text-xs"
                      />
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
