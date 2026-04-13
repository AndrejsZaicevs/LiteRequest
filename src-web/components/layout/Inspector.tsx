import { useState, useMemo, useEffect } from "react";
import type { RequestData, RequestVersion, RequestExecution, Environment, KeyValuePair } from "../../lib/types";
import { statusColor } from "../../lib/types";
import { KvTable } from "../inspector/KvTable";
import * as api from "../../lib/api";

interface InspectorProps {
  data: RequestData;
  onChange: (data: RequestData) => void;
  versions: RequestVersion[];
  executions: RequestExecution[];
  selectedVersionId: string | null;
  selectedExecutionId: string | null;
  onSelectVersion: (id: string) => void;
  onSelectExecution: (id: string) => void;
  environments: Environment[];
}

type Section = "params" | "headers" | "pathParams" | "versions" | "executions";

export function Inspector({
  data, onChange,
  versions, executions,
  selectedVersionId, selectedExecutionId,
  onSelectVersion, onSelectExecution,
  environments,
}: InspectorProps) {
  const [openSections, setOpenSections] = useState<Set<Section>>(
    new Set(["params", "headers", "pathParams"])
  );
  const [execEnvFilter, setExecEnvFilter] = useState<string>("all");
  const [execVersionFilter, setExecVersionFilter] = useState<string>("all");

  // Extract path params from URL
  const [pathParams, setPathParams] = useState<KeyValuePair[]>([]);

  useEffect(() => {
    if (!data.url) {
      if (pathParams.length > 0) setPathParams([]);
      return;
    }
    api.extractPathParams(data.url).then((paramNames) => {
      if (paramNames.length === 0) {
        if (pathParams.length > 0) setPathParams([]);
        return;
      }
      const existing = data.path_params ?? [];
      const merged = paramNames.map(name => {
        const found = existing.find(p => p.key === name);
        return found ?? { key: name, value: "", enabled: true };
      });
      setPathParams(merged);
    }).catch(() => {});
  }, [data.url]);

  const toggleSection = (s: Section) => {
    setOpenSections(prev => {
      const next = new Set(prev);
      if (next.has(s)) next.delete(s); else next.add(s);
      return next;
    });
  };

  const enabledParams = data.query_params.filter(p => p.enabled).length;
  const enabledHeaders = data.headers.filter(h => h.enabled).length;

  const updateParams = (params: KeyValuePair[]) => onChange({ ...data, query_params: params });
  const updateHeaders = (headers: KeyValuePair[]) => onChange({ ...data, headers });
  const updatePathParams = (pp: KeyValuePair[]) => onChange({ ...data, path_params: pp });

  // Filter executions
  const filteredExecutions = useMemo(() => {
    let filtered = executions;
    if (execEnvFilter !== "all") {
      filtered = filtered.filter(e => e.environment_id === execEnvFilter);
    }
    if (execVersionFilter !== "all") {
      filtered = filtered.filter(e => e.version_id === execVersionFilter);
    }
    return filtered;
  }, [executions, execEnvFilter, execVersionFilter]);

  // Group by date helper
  const groupByDate = <T,>(items: T[], getDate: (item: T) => string) => {
    const groups: { label: string; items: T[] }[] = [];
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterday = new Date(today.getTime() - 86400000);

    const todayItems: T[] = [];
    const yesterdayItems: T[] = [];
    const olderItems: T[] = [];

    for (const item of items) {
      const d = new Date(getDate(item));
      if (d >= today) todayItems.push(item);
      else if (d >= yesterday) yesterdayItems.push(item);
      else olderItems.push(item);
    }

    if (todayItems.length) groups.push({ label: "Today", items: todayItems });
    if (yesterdayItems.length) groups.push({ label: "Yesterday", items: yesterdayItems });
    if (olderItems.length) groups.push({ label: "Older", items: olderItems });
    return groups;
  };

  const groupedVersions = useMemo(() => groupByDate(versions, v => v.created_at), [versions]);
  const groupedExecutions = useMemo(() => groupByDate(filteredExecutions, e => e.executed_at), [filteredExecutions]);

  return (
    <div className="h-full flex flex-col overflow-y-auto" style={{ background: "var(--surface-1)" }}>
      {/* Query Params */}
      <div className="section-header" onClick={() => toggleSection("params")}>
        <span style={{ fontSize: 11 }}>{openSections.has("params") ? "▼" : "▶"}</span>
        <span>Query Params</span>
        {enabledParams > 0 && <span className="badge">{enabledParams}</span>}
      </div>
      {openSections.has("params") && (
        <KvTable rows={data.query_params} onChange={updateParams} placeholder={{ key: "param", value: "value" }} />
      )}

      {/* Path Params */}
      {pathParams.length > 0 && (
        <>
          <div className="section-header" onClick={() => toggleSection("pathParams")}>
            <span style={{ fontSize: 11 }}>{openSections.has("pathParams") ? "▼" : "▶"}</span>
            <span>Path Params</span>
            <span className="badge">{pathParams.length}</span>
          </div>
          {openSections.has("pathParams") && (
            <KvTable rows={pathParams} onChange={updatePathParams} placeholder={{ key: "param", value: "value" }} fixedKeys />
          )}
        </>
      )}

      {/* Headers */}
      <div className="section-header" onClick={() => toggleSection("headers")}>
        <span style={{ fontSize: 11 }}>{openSections.has("headers") ? "▼" : "▶"}</span>
        <span>Headers</span>
        {enabledHeaders > 0 && <span className="badge">{enabledHeaders}</span>}
      </div>
      {openSections.has("headers") && (
        <KvTable rows={data.headers} onChange={updateHeaders} placeholder={{ key: "header", value: "value" }} />
      )}

      {/* Versions */}
      <div className="section-header" onClick={() => toggleSection("versions")}>
        <span style={{ fontSize: 11 }}>{openSections.has("versions") ? "▼" : "▶"}</span>
        <span>Versions</span>
        <span className="badge">{versions.length}</span>
      </div>
      {openSections.has("versions") && (
        <div>
          {groupedVersions.map(group => (
            <div key={group.label}>
              <div
                className="px-3.5 py-1.5 text-xs font-medium uppercase tracking-wider"
                style={{ color: "var(--text-muted)", background: "var(--surface-0)" }}
              >
                {group.label}
              </div>
              {group.items.map(v => {
                const isSelected = v.id === selectedVersionId;
                const date = new Date(v.created_at);
                return (
                  <button
                    key={v.id}
                    onClick={() => onSelectVersion(v.id)}
                    className="w-full text-left px-3.5 py-2.5 text-xs flex items-center gap-2 transition-colors"
                    style={{
                      background: isSelected ? "var(--surface-2)" : "transparent",
                      borderLeft: isSelected ? "2px solid var(--accent)" : "2px solid transparent",
                    }}
                    onMouseEnter={(e) => { if (!isSelected) e.currentTarget.style.background = "var(--row-hover)"; }}
                    onMouseLeave={(e) => { if (!isSelected) e.currentTarget.style.background = "transparent"; }}
                  >
                    <span className="font-mono text-xs font-bold" style={{ color: "var(--text-muted)" }}>
                      {v.data.method}
                    </span>
                    <span className="truncate flex-1 font-mono text-xs" style={{ color: "var(--text-secondary)" }}>
                      {v.data.url || "(empty)"}
                    </span>
                    <span className="text-xs flex-shrink-0 tabular-nums" style={{ color: "var(--text-muted)" }}>
                      {date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                    </span>
                  </button>
                );
              })}
            </div>
          ))}
        </div>
      )}

      {/* Executions */}
      <div className="section-header" onClick={() => toggleSection("executions")}>
        <span style={{ fontSize: 11 }}>{openSections.has("executions") ? "▼" : "▶"}</span>
        <span>Executions</span>
        <span className="badge">{executions.length}</span>
      </div>
      {openSections.has("executions") && (
        <div>
          {/* Filters */}
          {(environments.length > 0 || versions.length > 1) && (
            <div className="flex items-center gap-2 px-3.5 py-2" style={{ background: "var(--surface-0)", borderBottom: "1px solid var(--border-subtle)" }}>
              {environments.length > 0 && (
                <select
                  value={execEnvFilter}
                  onChange={(e) => setExecEnvFilter(e.target.value)}
                  style={{ background: "var(--surface-2)", border: "1px solid var(--border)", color: "var(--text-secondary)", borderRadius: 4, fontSize: 12, padding: "5px 22px 5px 9px" }}
                >
                  <option value="all">All envs</option>
                  {environments.map(env => (
                    <option key={env.id} value={env.id}>{env.name}</option>
                  ))}
                </select>
              )}
              {versions.length > 1 && (
                <select
                  value={execVersionFilter}
                  onChange={(e) => setExecVersionFilter(e.target.value)}
                  style={{ background: "var(--surface-2)", border: "1px solid var(--border)", color: "var(--text-secondary)", borderRadius: 4, fontSize: 12, padding: "5px 22px 5px 9px" }}
                >
                  <option value="all">All versions</option>
                  {versions.map((v, i) => (
                    <option key={v.id} value={v.id}>v{versions.length - i}</option>
                  ))}
                </select>
              )}
            </div>
          )}

          {groupedExecutions.map(group => (
            <div key={group.label}>
              <div
                className="px-3.5 py-1.5 text-xs font-medium uppercase tracking-wider"
                style={{ color: "var(--text-muted)", background: "var(--surface-0)" }}
              >
                {group.label}
              </div>
              {group.items.map(exec => {
                const isSelected = exec.id === selectedExecutionId;
                const date = new Date(exec.executed_at);
                const status = exec.response.status;
                return (
                  <button
                    key={exec.id}
                    onClick={() => onSelectExecution(exec.id)}
                    className="w-full text-left px-3.5 py-2.5 text-xs flex items-center gap-3 transition-colors"
                    style={{
                      background: isSelected ? "var(--surface-2)" : "transparent",
                      borderLeft: isSelected ? "2px solid var(--accent)" : "2px solid transparent",
                    }}
                    onMouseEnter={(e) => { if (!isSelected) e.currentTarget.style.background = "var(--row-hover)"; }}
                    onMouseLeave={(e) => { if (!isSelected) e.currentTarget.style.background = "transparent"; }}
                  >
                    <span
                      className="font-mono font-bold text-sm w-9 text-center rounded px-1 py-0.5"
                      style={{
                        color: statusColor(status),
                        background: `${statusColor(status)}15`,
                      }}
                    >
                      {status}
                    </span>
                    <span className="text-xs tabular-nums font-mono" style={{ color: "var(--text-secondary)" }}>
                      {exec.latency_ms}ms
                    </span>
                    <span className="flex-1" />
                    <span className="text-xs flex-shrink-0 tabular-nums" style={{ color: "var(--text-muted)" }}>
                      {date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                    </span>
                  </button>
                );
              })}
            </div>
          ))}

          {filteredExecutions.length === 0 && (
            <div className="px-3.5 py-4 text-xs text-center" style={{ color: "var(--text-muted)" }}>
              No executions{execEnvFilter !== "all" || execVersionFilter !== "all" ? " matching filters" : ""}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
