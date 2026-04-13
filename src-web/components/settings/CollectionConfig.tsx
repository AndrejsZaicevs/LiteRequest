import { useState, useEffect } from "react";
import type { Collection, Environment, VarDef, VarRow, AuthConfig } from "../../lib/types";
import * as api from "../../lib/api";

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

export function CollectionConfig({ collectionId, collections, environments, onUpdate }: CollectionConfigProps) {
  const collection = collections.find(c => c.id === collectionId);
  const [basePath, setBasePath] = useState(collection?.base_path ?? "");
  const [authConfig, setAuthConfig] = useState<AuthConfig>(() => parseAuthConfig(collection?.auth_config ?? null));
  const [varDefs, setVarDefs] = useState<VarDef[]>([]);
  const [varRows, setVarRows] = useState<VarRow[]>([]);
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
  }, [collection]);

  const save = async () => {
    if (!collection) return;
    await api.updateCollection({
      ...collection,
      base_path: basePath,
      auth_config: JSON.stringify(authConfig),
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
    const defs = await api.listVarDefs(collectionId);
    setVarDefs(defs);
  };

  const updateVarValue = async (row: VarRow, value: string) => {
    if (!activeEnv) return;
    await api.upsertVarValue(
      row.value_id ?? crypto.randomUUID(),
      row.def_id,
      activeEnv.id,
      value,
      row.is_secret,
    );
    const rows = await api.loadVarRows(collectionId, activeEnv.id);
    setVarRows(rows);
  };

  const deleteVarDef = async (defId: string) => {
    await api.deleteVarDef(defId);
    const defs = await api.listVarDefs(collectionId);
    setVarDefs(defs);
    if (activeEnv) {
      const rows = await api.loadVarRows(collectionId, activeEnv.id);
      setVarRows(rows);
    }
  };

  if (!collection) {
    return <div className="p-4 text-xs" style={{ color: "var(--text-muted)" }}>Collection not found</div>;
  }

  return (
    <div className="h-full overflow-y-auto p-4" style={{ background: "var(--surface-0)" }}>
      <h2 className="text-lg font-semibold mb-4">{collection.name}</h2>

      {/* Base Path */}
      <div className="mb-4">
        <label className="block text-xs font-medium mb-1" style={{ color: "var(--text-secondary)" }}>Base Path</label>
        <input
          value={basePath}
          onChange={(e) => setBasePath(e.target.value)}
          onBlur={save}
          placeholder="https://api.example.com"
          className="w-full max-w-lg font-mono"
        />
      </div>

      {/* Auth Type */}
      <div className="mb-4">
        <label className="block text-xs font-medium mb-1" style={{ color: "var(--text-secondary)" }}>Authentication</label>
        <select
          value={authConfig.auth_type}
          onChange={(e) => { setAuthConfig({ ...authConfig, auth_type: e.target.value as AuthConfig["auth_type"] }); }}
          onBlur={save}
          className="px-2 py-1 rounded text-xs"
          style={{ background: "var(--surface-1)", border: "1px solid var(--border)", color: "var(--text-primary)" }}
        >
          <option value="none">None</option>
          <option value="bearer">Bearer Token</option>
          <option value="basic">Basic Auth</option>
          <option value="api_key">API Key</option>
        </select>
      </div>

      {/* Auth data fields */}
      {authConfig.auth_type === "bearer" && (
        <div className="mb-4">
          <label className="block text-xs font-medium mb-1" style={{ color: "var(--text-secondary)" }}>Token</label>
          <input
            value={authConfig.bearer_token ?? ""}
            onChange={(e) => setAuthConfig({ ...authConfig, bearer_token: e.target.value })}
            onBlur={save}
            placeholder="Bearer token value..."
            className="w-full max-w-lg font-mono"
          />
        </div>
      )}

      {authConfig.auth_type === "basic" && (
        <div className="mb-4 flex gap-2">
          <div className="flex-1">
            <label className="block text-xs font-medium mb-1" style={{ color: "var(--text-secondary)" }}>Username</label>
            <input
              value={authConfig.basic_username ?? ""}
              onChange={(e) => setAuthConfig({ ...authConfig, basic_username: e.target.value })}
              onBlur={save}
              className="w-full font-mono"
            />
          </div>
          <div className="flex-1">
            <label className="block text-xs font-medium mb-1" style={{ color: "var(--text-secondary)" }}>Password</label>
            <input
              type="password"
              value={authConfig.basic_password ?? ""}
              onChange={(e) => setAuthConfig({ ...authConfig, basic_password: e.target.value })}
              onBlur={save}
              className="w-full font-mono"
            />
          </div>
        </div>
      )}

      {authConfig.auth_type === "api_key" && (
        <div className="mb-4 flex gap-2">
          <div className="flex-1">
            <label className="block text-xs font-medium mb-1" style={{ color: "var(--text-secondary)" }}>Header Name</label>
            <input
              value={authConfig.api_key_header ?? "X-API-Key"}
              onChange={(e) => setAuthConfig({ ...authConfig, api_key_header: e.target.value })}
              onBlur={save}
              className="w-full font-mono"
            />
          </div>
          <div className="flex-1">
            <label className="block text-xs font-medium mb-1" style={{ color: "var(--text-secondary)" }}>API Key</label>
            <input
              value={authConfig.api_key_value ?? ""}
              onChange={(e) => setAuthConfig({ ...authConfig, api_key_value: e.target.value })}
              onBlur={save}
              className="w-full font-mono"
            />
          </div>
        </div>
      )}

      {/* Collection Variables */}
      <div className="mt-6">
        <div className="flex items-center justify-between mb-2">
          <h3 className="text-sm font-semibold" style={{ color: "var(--text-secondary)" }}>
            Collection Variables
            {activeEnv && (
              <span className="ml-2 text-[10px] font-normal px-1.5 py-0.5 rounded"
                style={{ background: "var(--accent)", color: "#fff" }}>
                {activeEnv.name}
              </span>
            )}
          </h3>
          <button
            onClick={addVarDef}
            className="text-xs px-2 py-0.5 rounded"
            style={{ background: "var(--surface-2)", border: "1px solid var(--border)", color: "var(--accent)" }}
          >
            + Add
          </button>
        </div>

        {!activeEnv && (
          <div className="text-xs p-2 rounded" style={{ background: "var(--surface-1)", color: "var(--text-muted)" }}>
            Select an environment to manage variable values
          </div>
        )}

        <div className="text-xs">
          {varDefs.map(def => {
            const row = varRows.find(r => r.def_id === def.id);
            return (
              <div key={def.id} className="flex items-center border-b" style={{ borderColor: "var(--border)" }}>
                <input
                  value={def.key}
                  onChange={(e) => updateVarKey(def.id, e.target.value)}
                  className="flex-1 bg-transparent border-none outline-none px-2 py-1.5 font-mono"
                  style={{ color: "var(--text-primary)", borderRight: "1px solid var(--border)" }}
                />
                <input
                  value={row?.value ?? ""}
                  onChange={(e) => {
                    if (row) updateVarValue(row, e.target.value);
                    else if (activeEnv) {
                      api.upsertVarValue(crypto.randomUUID(), def.id, activeEnv.id, e.target.value, false)
                        .then(() => api.loadVarRows(collectionId, activeEnv.id))
                        .then(setVarRows);
                    }
                  }}
                  placeholder="value"
                  className="flex-1 bg-transparent border-none outline-none px-2 py-1.5 font-mono"
                  style={{ color: "var(--text-primary)" }}
                  disabled={!activeEnv}
                />
                <button
                  onClick={() => deleteVarDef(def.id)}
                  className="w-6 flex-shrink-0 hover:opacity-80"
                  style={{ color: "var(--text-muted)" }}
                >
                  ×
                </button>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
