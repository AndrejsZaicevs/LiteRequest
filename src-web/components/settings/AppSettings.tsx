import { useState, useEffect } from "react";
import { Settings, Plus, Trash2, Eye, EyeOff } from "lucide-react";
import type { Environment, EnvVariable, KeyValuePair, ClientCertEntry } from "../../lib/types";
import * as api from "../../lib/api";
import { KvTable } from "../inspector/KvTable";
import { CollapsibleSection } from "../shared/CollapsibleSection";

type Section = "environments" | "headers" | "variables" | "certificates";

interface AppSettingsProps {
  environments: Environment[];
  onUpdate: () => void;
}

export function AppSettings({ environments, onUpdate }: AppSettingsProps) {
  const [open, setOpen] = useState<Set<Section>>(new Set(["environments", "headers", "variables", "certificates"]));
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

  const toggle = (s: Section) => setOpen(prev => {
    const next = new Set(prev);
    if (next.has(s)) next.delete(s); else next.add(s);
    return next;
  });

  const inputClass = "w-full bg-[#0d0d0d] border border-gray-800 rounded-md px-3 py-2 text-sm text-gray-200 font-mono placeholder-gray-600 focus:outline-none focus:border-gray-700 focus:bg-[#1a1a1a] transition-colors";
  const selectClass = "bg-[#0d0d0d] border border-gray-800 rounded-md px-3 py-2 text-sm text-gray-200 focus:outline-none focus:border-gray-700 transition-colors";
  const labelClass = "block text-[11px] font-bold text-gray-500 uppercase tracking-wider mb-1.5";
  const btnClass = "flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-gray-300 bg-[#1a1a1a] border border-gray-700 rounded-md hover:bg-[#242424] hover:text-gray-100 transition-colors";
  const btnDangerClass = "flex items-center gap-1.5 px-3 py-1.5 text-xs font-medium text-red-400 bg-[#1a1a1a] border border-red-500/30 rounded-md hover:bg-red-500/10 transition-colors";

  return (
    <div className="h-full overflow-y-auto bg-[#121212]">
      {/* Header */}
      <div className="px-4 py-4 border-b border-gray-800 flex items-center gap-2">
        <Settings size={16} className="text-gray-500 shrink-0" />
        <h2 className="text-sm font-semibold text-gray-200">Settings</h2>
      </div>

      {/* Environments */}
      <CollapsibleSection
        title="Environments"
        count={environments.length}
        isOpen={open.has("environments")}
        onToggle={() => toggle("environments")}
        action={
          <button onClick={addEnvironment} className="flex items-center gap-1 px-2 py-1 text-[11px] font-medium text-gray-400 hover:text-gray-200 hover:bg-[#2a2a2a] rounded transition-colors">
            <Plus size={11} /> New
          </button>
        }
      >
        <div className="flex gap-4">
          {/* Env list */}
          <div className="w-44 shrink-0 border border-gray-800 rounded-md overflow-hidden">
            {environments.map(env => (
              <div
                key={env.id}
                className={`flex items-center px-3 py-2.5 cursor-pointer transition-colors border-b border-gray-800/50 last:border-0 ${
                  selectedEnv === env.id ? "bg-[#242424]" : "hover:bg-[#1a1a1a]"
                }`}
                onClick={() => setSelectedEnv(env.id)}
                onDoubleClick={() => { setRenamingEnv(env.id); setRenameValue(env.name); }}
              >
                {renamingEnv === env.id ? (
                  <input
                    value={renameValue}
                    onChange={(e) => setRenameValue(e.target.value)}
                    onBlur={handleRenameEnv}
                    onKeyDown={(e) => { if (e.key === "Enter") handleRenameEnv(); if (e.key === "Escape") setRenamingEnv(null); }}
                    className="flex-1 bg-transparent text-sm text-gray-200 outline-none"
                    style={{ border: "none", borderBottom: "1px solid #3b82f6", borderRadius: 0, padding: "0 2px" }}
                    autoFocus
                    onClick={(e) => e.stopPropagation()}
                  />
                ) : (
                  <span className="flex-1 text-sm text-gray-300 truncate">{env.name}</span>
                )}
                {env.is_active && (
                  <span className="text-[10px] px-1.5 py-0.5 rounded-full font-medium ml-1 shrink-0 bg-green-500/20 text-green-400 border border-green-500/30">
                    active
                  </span>
                )}
              </div>
            ))}
            {environments.length === 0 && (
              <div className="px-4 py-4 text-xs text-center text-gray-600">No environments</div>
            )}
          </div>

          {/* Env variables */}
          {selectedEnv && (
            <div className="flex-1 min-w-0">
              <div className="flex items-center justify-between mb-2">
                <span className="text-xs font-medium text-gray-400">Variables</span>
                <div className="flex gap-1.5">
                  <button onClick={addEnvVar} className={btnClass}><Plus size={12} /> Add</button>
                  <button onClick={() => { setRenamingEnv(selectedEnv); setRenameValue(environments.find(e => e.id === selectedEnv)?.name ?? ""); }} className={btnClass}>Rename</button>
                  <button onClick={() => deleteEnvironment(selectedEnv)} className={btnDangerClass}><Trash2 size={12} /></button>
                </div>
              </div>
              <div className="border border-gray-800 rounded-md overflow-hidden">
                {envVars.map(v => (
                  <div key={v.id} className="kv-row">
                    <input value={v.key} onChange={(e) => updateEnvVar({ ...v, key: e.target.value })} className="kv-cell" style={{ border: "none", borderRadius: 0, fontWeight: 500 }} />
                    <div className="kv-divider" />
                    <input value={v.value} type={v.is_secret ? "password" : "text"} onChange={(e) => updateEnvVar({ ...v, value: e.target.value })} className="kv-cell" style={{ border: "none", borderRadius: 0 }} />
                    <button onClick={() => updateEnvVar({ ...v, is_secret: !v.is_secret })} className="kv-action text-gray-600 hover:text-gray-300" title={v.is_secret ? "Show" : "Hide"}>
                      {v.is_secret ? <EyeOff size={12} /> : <Eye size={12} />}
                    </button>
                    <button onClick={() => deleteEnvVar(v.id)} className="kv-action text-gray-600 hover:text-red-400"><Trash2 size={12} /></button>
                  </div>
                ))}
                {envVars.length === 0 && (
                  <div className="px-4 py-4 text-xs text-center text-gray-600">No variables — click + Add</div>
                )}
              </div>
            </div>
          )}
        </div>
      </CollapsibleSection>

      {/* Default Headers */}
      <CollapsibleSection
        title="Default Headers"
        count={defaultHeaders.filter(h => h.key).length}
        isOpen={open.has("headers")}
        onToggle={() => toggle("headers")}
      >
        <p className="text-xs mb-3 text-gray-600">Sent with every request across all collections</p>
        <div className="border border-gray-800 rounded-md overflow-hidden" style={{ maxWidth: 600 }}>
          <KvTable rows={defaultHeaders} onChange={saveDefaultHeaders} placeholder={{ key: "Header-Name", value: "value" }} />
        </div>
      </CollapsibleSection>

      {/* Global Variables */}
      <CollapsibleSection
        title="Global Variables"
        count={appWideVars.filter(v => v.key).length}
        isOpen={open.has("variables")}
        onToggle={() => toggle("variables")}
      >
        <p className="text-xs mb-3 text-gray-600">Available in all collections, not environment-specific</p>
        <div className="border border-gray-800 rounded-md overflow-hidden" style={{ maxWidth: 600 }}>
          <KvTable rows={appWideVars} onChange={saveAppVars} placeholder={{ key: "VARIABLE_NAME", value: "value" }} />
        </div>
      </CollapsibleSection>

      {/* Client Certificates */}
      <CollapsibleSection
        title="Client Certificates"
        count={clientCerts.length}
        isOpen={open.has("certificates")}
        onToggle={() => toggle("certificates")}
        action={
          <button onClick={addCert} className="flex items-center gap-1 px-2 py-1 text-[11px] font-medium text-gray-400 hover:text-gray-200 hover:bg-[#2a2a2a] rounded transition-colors">
            <Plus size={11} /> Add
          </button>
        }
      >
        {clientCerts.length === 0 && (
          <div className="text-xs p-4 rounded-md text-center bg-[#1a1a1a] text-gray-600 border border-gray-800">
            No client certificates configured
          </div>
        )}
        <div className="space-y-3">
          {clientCerts.map((cert, i) => (
            <div key={i} className={`rounded-md p-4 border border-gray-800 bg-[#1a1a1a] ${cert.enabled ? "" : "opacity-50"}`}>
              <div className="flex items-center justify-between mb-3">
                <div className="flex items-center gap-2">
                  <button
                    onClick={() => updateCert(i, { enabled: !cert.enabled })}
                    className={`w-4 h-4 rounded border flex items-center justify-center ${cert.enabled ? "bg-blue-600 border-blue-500" : "border-gray-600"}`}
                  >
                    {cert.enabled && <span className="text-white text-[10px]">✓</span>}
                  </button>
                  <span className="text-sm font-medium text-gray-300">Certificate {i + 1}</span>
                </div>
                <button onClick={() => removeCert(i)} className={btnDangerClass} style={{ padding: "2px 8px" }}>
                  <Trash2 size={12} /> Remove
                </button>
              </div>
              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className={labelClass}>Host Pattern</label>
                  <input value={cert.host} onChange={(e) => updateCert(i, { host: e.target.value })} placeholder="*.example.com" className={inputClass} />
                </div>
                <div>
                  <label className={labelClass}>Type</label>
                  <select value={cert.cert_type} onChange={(e) => updateCert(i, { cert_type: e.target.value as "Pem" | "Pkcs12" })} className={selectClass}>
                    <option value="Pem">PEM</option>
                    <option value="Pkcs12">PKCS#12</option>
                  </select>
                </div>
                <div>
                  <label className={labelClass}>Certificate Path</label>
                  <input value={cert.cert_path} onChange={(e) => updateCert(i, { cert_path: e.target.value })} placeholder="/path/to/cert.pem" className={inputClass} />
                </div>
                <div>
                  <label className={labelClass}>Key Path</label>
                  <input value={cert.key_path} onChange={(e) => updateCert(i, { key_path: e.target.value })} placeholder="/path/to/key.pem" className={inputClass} />
                </div>
                <div>
                  <label className={labelClass}>CA Path (optional)</label>
                  <input value={cert.ca_path} onChange={(e) => updateCert(i, { ca_path: e.target.value })} placeholder="/path/to/ca.pem" className={inputClass} />
                </div>
                <div>
                  <label className={labelClass}>Passphrase (optional)</label>
                  <input type="password" value={cert.passphrase} onChange={(e) => updateCert(i, { passphrase: e.target.value })} className={inputClass} />
                </div>
              </div>
            </div>
          ))}
        </div>
      </CollapsibleSection>
    </div>
  );
}
