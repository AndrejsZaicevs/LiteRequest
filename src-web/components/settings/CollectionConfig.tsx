import { useState, useEffect } from "react";
import { FolderOpen, Plus } from "lucide-react";
import type { Collection, Environment, VarDef, VarRow, AuthConfig, KeyValuePair } from "../../lib/types";
import * as api from "../../lib/api";
import { KvTable } from "../inspector/KvTable";
import { CollapsibleSection } from "../shared/CollapsibleSection";
import { VarDefTable } from "../shared/VarDefTable";

interface CollectionConfigProps {
  collectionId: string;
  collections: Collection[];
  environments: Environment[];
  onUpdate: () => void;
}

function parseAuthConfig(json: string | null): AuthConfig {
  if (!json) return { auth_type: "none" };
  try { return JSON.parse(json); } catch { return { auth_type: "none" }; }
}

function parseHeadersConfig(json: string | null): KeyValuePair[] {
  if (!json) return [];
  try { return JSON.parse(json); } catch { return []; }
}

type Section = "general" | "auth" | "headers" | "variables";

export function CollectionConfig({ collectionId, collections, environments, onUpdate }: CollectionConfigProps) {
  const collection = collections.find(c => c.id === collectionId);
  const [basePath, setBasePath] = useState(collection?.base_path ?? "");
  const [authConfig, setAuthConfig] = useState<AuthConfig>(() => parseAuthConfig(collection?.auth_config ?? null));
  const [headersConfig, setHeadersConfig] = useState<KeyValuePair[]>(() => parseHeadersConfig(collection?.headers_config ?? null));
  const [varDefs, setVarDefs] = useState<VarDef[]>([]);
  const [varRows, setVarRows] = useState<VarRow[]>([]);
  const [open, setOpen] = useState<Set<Section>>(new Set(["general", "auth", "headers", "variables"]));
  const activeEnv = environments.find(e => e.is_active);

  useEffect(() => {
    if (!collectionId) return;
    api.listVarDefs(collectionId).then(setVarDefs).catch(console.error);
    if (activeEnv) {
      api.loadVarRows(collectionId, activeEnv.id).then(setVarRows).catch(console.error);
    }
  }, [collectionId, activeEnv]);

  useEffect(() => {
    if (!collection) return;
    setBasePath(collection.base_path);
    setAuthConfig(parseAuthConfig(collection.auth_config));
    setHeadersConfig(parseHeadersConfig(collection.headers_config));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [collectionId]); // Only reset when switching collections, not on every parent refresh

  const toggle = (s: Section) => setOpen(prev => {
    const next = new Set(prev);
    if (next.has(s)) next.delete(s); else next.add(s);
    api.setAppSetting("collection_config_open_sections", JSON.stringify([...next])).catch(() => {});
    return next;
  });

  useEffect(() => {
    api.getAppSetting("collection_config_open_sections").then(v => {
      if (v) { try { setOpen(new Set(JSON.parse(v) as Section[])); } catch {} }
    }).catch(() => {});
  }, []);

  const save = async () => {
    if (!collection) return;
    await api.updateCollection({
      ...collection,
      base_path: basePath,
      auth_config: JSON.stringify(authConfig),
      headers_config: headersConfig.some(h => h.key) ? JSON.stringify(headersConfig) : null,
    });
    onUpdate();
  };

  const saveAuth = async (cfg: AuthConfig) => {
    if (!collection) return;
    await api.updateCollection({
      ...collection,
      base_path: basePath,
      auth_config: JSON.stringify(cfg),
      headers_config: headersConfig.some(h => h.key) ? JSON.stringify(headersConfig) : null,
    });
    onUpdate();
  };

  const saveHeaders = async (headers: KeyValuePair[]) => {
    setHeadersConfig(headers);
    if (!collection) return;
    await api.updateCollection({
      ...collection,
      base_path: basePath,
      auth_config: JSON.stringify(authConfig),
      headers_config: headers.some(h => h.key) ? JSON.stringify(headers) : null,
    });
    onUpdate();
  };

  const addVarDef = async () => {
    const def: VarDef = {
      id: crypto.randomUUID(),
      collection_id: collectionId,
      key: "NEW_VAR",
      sort_order: varDefs.length,
    };
    await api.insertVarDef(def);
    const defs = await api.listVarDefs(collectionId);
    setVarDefs(defs);
    if (activeEnv) {
      const rows = await api.loadVarRows(collectionId, activeEnv.id);
      setVarRows(rows);
    }
  };

  const updateVarKey = async (defId: string, key: string) => {
    await api.updateVarDefKey(defId, key);
    setVarDefs(await api.listVarDefs(collectionId));
  };

  const updateVarValue = (row: VarRow, value: string) => {
    if (!activeEnv) return;
    // Optimistic update so cursor isn't reset by async DB round-trip
    setVarRows(prev => prev.map(r => r.def_id === row.def_id ? { ...r, value } : r));
    api.upsertVarValue(row.value_id ?? crypto.randomUUID(), row.def_id, activeEnv.id, value, row.is_secret)
      .then(() => onUpdate())
      .catch(console.error);
  };

  const deleteVarDef = async (defId: string) => {
    await api.deleteVarDef(defId);
    setVarDefs(await api.listVarDefs(collectionId));
    if (activeEnv) setVarRows(await api.loadVarRows(collectionId, activeEnv.id));
  };

  const toggleVarSecret = (row: VarRow) => {
    if (!activeEnv) return;
    setVarRows(prev => prev.map(r => r.value_id === row.value_id ? { ...r, is_secret: !r.is_secret } : r));
    api.upsertVarValue(row.value_id ?? crypto.randomUUID(), row.def_id, activeEnv.id, row.value, !row.is_secret)
      .then(() => onUpdate())
      .catch(console.error);
  };

  const createVarRow = (defId: string, defKey: string, value: string) => {
    if (!activeEnv) return;
    const newId = crypto.randomUUID();
    const newRow: VarRow = { value_id: newId, def_id: defId, key: defKey, value, is_secret: false };
    setVarRows(prev => [...prev, newRow]);
    api.upsertVarValue(newId, defId, activeEnv.id, value, false)
      .then(() => onUpdate())
      .catch(console.error);
  };

  if (!collection) {
    return <div className="p-6 text-sm text-gray-600">Collection not found</div>;
  }

  const inputClass = "w-full bg-[#0d0d0d] border border-gray-800 rounded-md px-3 py-2 text-sm text-gray-200 font-mono placeholder-gray-600 focus:outline-none focus:border-gray-700 focus:bg-[#1a1a1a] transition-colors";
  const selectClass = "bg-[#0d0d0d] border border-gray-800 rounded-md px-3 py-2 pr-8 text-sm text-gray-200 focus:outline-none focus:border-gray-700 transition-colors cursor-pointer";
  const labelClass = "block text-[11px] font-bold text-gray-500 uppercase tracking-wider mb-1.5";

  return (
    <div className="h-full overflow-y-auto bg-[#121212]">
      {/* Header */}
      <div className="px-4 py-4 border-b border-gray-800 flex items-center gap-2">
        <FolderOpen size={16} className="text-gray-500 shrink-0" />
        <h2 className="text-sm font-semibold text-gray-200 truncate">{collection.name}</h2>
      </div>

      {/* General */}
      <CollapsibleSection
        title="General"
        isOpen={open.has("general")}
        onToggle={() => toggle("general")}
      >
        <div className="space-y-3 max-w-lg">
          <div>
            <label className={labelClass}>Base Path</label>
            <input
              value={basePath}
              onChange={(e) => setBasePath(e.target.value)}
              onBlur={save}
              placeholder="https://api.example.com"
              className={inputClass}
            />
            <p className="text-xs mt-1.5 text-gray-600">
              Prepended to request paths that don't start with http(s)://
            </p>
          </div>
        </div>
      </CollapsibleSection>

      {/* Authentication */}
      <CollapsibleSection
        title="Authentication"
        isOpen={open.has("auth")}
        onToggle={() => toggle("auth")}
      >
        <div className="space-y-3 max-w-lg">
          <div>
            <label className={labelClass}>Type</label>
            <select
              value={authConfig.auth_type}
              onChange={(e) => {
                const cfg = { ...authConfig, auth_type: e.target.value as AuthConfig["auth_type"] };
                setAuthConfig(cfg);
                saveAuth(cfg);
              }}
              className={selectClass}
              style={{ minWidth: 160, appearance: "none" }}
            >
              <option value="none">None</option>
              <option value="bearer">Bearer Token</option>
              <option value="basic">Basic Auth</option>
              <option value="api_key">API Key</option>
            </select>
          </div>

          {authConfig.auth_type === "bearer" && (
            <div>
              <label className={labelClass}>Token</label>
              <input
                value={authConfig.bearer_token ?? ""}
                onChange={(e) => setAuthConfig({ ...authConfig, bearer_token: e.target.value })}
                onBlur={save}
                placeholder="Bearer token value..."
                className={inputClass}
              />
            </div>
          )}

          {authConfig.auth_type === "basic" && (
            <div className="flex gap-3">
              <div className="flex-1">
                <label className={labelClass}>Username</label>
                <input
                  value={authConfig.basic_username ?? ""}
                  onChange={(e) => setAuthConfig({ ...authConfig, basic_username: e.target.value })}
                  onBlur={save}
                  className={inputClass}
                />
              </div>
              <div className="flex-1">
                <label className={labelClass}>Password</label>
                <input
                  type="password"
                  value={authConfig.basic_password ?? ""}
                  onChange={(e) => setAuthConfig({ ...authConfig, basic_password: e.target.value })}
                  onBlur={save}
                  className={inputClass}
                />
              </div>
            </div>
          )}

          {authConfig.auth_type === "api_key" && (
            <div className="flex gap-3">
              <div className="flex-1">
                <label className={labelClass}>Header Name</label>
                <input
                  value={authConfig.api_key_header ?? "X-API-Key"}
                  onChange={(e) => setAuthConfig({ ...authConfig, api_key_header: e.target.value })}
                  onBlur={save}
                  className={inputClass}
                />
              </div>
              <div className="flex-1">
                <label className={labelClass}>API Key</label>
                <input
                  value={authConfig.api_key_value ?? ""}
                  onChange={(e) => setAuthConfig({ ...authConfig, api_key_value: e.target.value })}
                  onBlur={save}
                  className={inputClass}
                />
              </div>
            </div>
          )}
        </div>
      </CollapsibleSection>

      {/* Default Headers */}
      <CollapsibleSection
        title="Default Headers"
        count={headersConfig.filter(h => h.key).length}
        isOpen={open.has("headers")}
        onToggle={() => toggle("headers")}
      >
        <p className="text-xs mb-3 text-gray-600">
          Sent with every request in this collection
        </p>
        <div className="border border-gray-800 rounded-md overflow-hidden">
          <KvTable rows={headersConfig} onChange={saveHeaders} placeholder={{ key: "Header-Name", value: "value" }} />
        </div>
      </CollapsibleSection>

      {/* Variables */}
      <CollapsibleSection
        title="Variables"
        count={varDefs.length}
        isOpen={open.has("variables")}
        onToggle={() => toggle("variables")}
        action={
          <button
            onClick={addVarDef}
            className="flex items-center gap-1 px-2 py-1 text-[11px] font-medium text-gray-400 hover:text-gray-200 hover:bg-[#2a2a2a] rounded transition-colors"
          >
            <Plus size={11} /> Add
          </button>
        }
      >
        <div className="flex items-center gap-2 mb-2">
          <span className="text-xs text-gray-500">Values for env:</span>
          {activeEnv
            ? <span className="text-[10px] font-medium px-2 py-0.5 rounded-full bg-blue-500/20 text-blue-400 border border-blue-500/30">{activeEnv.name}</span>
            : <span className="text-[10px] text-gray-600">none active — select an env to edit values</span>
          }
        </div>
        <VarDefTable
          defs={varDefs}
          rows={varRows}
          hasEnv={!!activeEnv}
          onKeyChange={updateVarKey}
          onValueChange={updateVarValue}
          onValueCreate={createVarRow}
          onToggleSecret={toggleVarSecret}
          onDelete={deleteVarDef}
          emptyMessage="No variables yet"
        />
      </CollapsibleSection>
    </div>
  );
}

