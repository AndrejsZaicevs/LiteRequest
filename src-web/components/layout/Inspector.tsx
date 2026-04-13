import { useState, useMemo } from "react";
import type { RequestData, RequestVersion, RequestExecution, Environment, KeyValuePair } from "../../lib/types";
import { statusColor } from "../../lib/types";
import { KvTable } from "../inspector/KvTable";

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
}: InspectorProps) {
  const [openSections, setOpenSections] = useState<Set<Section>>(
    new Set(["params", "headers", "pathParams"])
  );

  const toggleSection = (s: Section) => {
    setOpenSections(prev => {
      const next = new Set(prev);
      if (next.has(s)) next.delete(s); else next.add(s);
      return next;
    });
  };

  const enabledParams = data.query_params.filter(p => p.enabled).length;
  const enabledHeaders = data.headers.filter(h => h.enabled).length;
  const pathParams = data.path_params ?? [];

  const updateParams = (params: KeyValuePair[]) => onChange({ ...data, query_params: params });
  const updateHeaders = (headers: KeyValuePair[]) => onChange({ ...data, headers });
  const updatePathParams = (pp: KeyValuePair[]) => onChange({ ...data, path_params: pp });

  // Group executions by date
  const groupedExecutions = useMemo(() => {
    const groups: { label: string; items: RequestExecution[] }[] = [];
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterday = new Date(today.getTime() - 86400000);

    const todayItems: RequestExecution[] = [];
    const yesterdayItems: RequestExecution[] = [];
    const olderItems: RequestExecution[] = [];

    for (const e of executions) {
      const d = new Date(e.executed_at);
      if (d >= today) todayItems.push(e);
      else if (d >= yesterday) yesterdayItems.push(e);
      else olderItems.push(e);
    }

    if (todayItems.length) groups.push({ label: "Today", items: todayItems });
    if (yesterdayItems.length) groups.push({ label: "Yesterday", items: yesterdayItems });
    if (olderItems.length) groups.push({ label: "Older", items: olderItems });
    return groups;
  }, [executions]);

  const SectionHeader = ({ section, label, count }: { section: Section; label: string; count?: number }) => (
    <button
      className="flex items-center gap-2 w-full px-3 py-1.5 text-xs font-semibold uppercase tracking-wider hover:bg-[var(--surface-2)] transition-colors"
      style={{ color: "var(--text-muted)" }}
      onClick={() => toggleSection(section)}
    >
      <span className="text-[10px]">{openSections.has(section) ? "▼" : "▶"}</span>
      <span>{label}</span>
      {count !== undefined && count > 0 && (
        <span className="ml-auto px-1.5 rounded-full text-[10px]" style={{ background: "var(--surface-2)", color: "var(--text-secondary)" }}>
          {count}
        </span>
      )}
    </button>
  );

  return (
    <div className="h-full flex flex-col overflow-y-auto" style={{ background: "var(--surface-1)" }}>
      {/* Query Params */}
      <SectionHeader section="params" label="Query Params" count={enabledParams} />
      {openSections.has("params") && (
        <KvTable rows={data.query_params} onChange={updateParams} placeholder={{ key: "param", value: "value" }} />
      )}

      {/* Path Params */}
      {pathParams.length > 0 && (
        <>
          <SectionHeader section="pathParams" label="Path Params" count={pathParams.length} />
          {openSections.has("pathParams") && (
            <KvTable rows={pathParams} onChange={updatePathParams} placeholder={{ key: "param", value: "value" }} fixedKeys />
          )}
        </>
      )}

      {/* Headers */}
      <SectionHeader section="headers" label="Headers" count={enabledHeaders} />
      {openSections.has("headers") && (
        <KvTable rows={data.headers} onChange={updateHeaders} placeholder={{ key: "header", value: "value" }} />
      )}

      {/* Versions */}
      <SectionHeader section="versions" label="Versions" count={versions.length} />
      {openSections.has("versions") && (
        <div className="px-1">
          {versions.map(v => {
            const isSelected = v.id === selectedVersionId;
            const date = new Date(v.created_at);
            return (
              <button
                key={v.id}
                onClick={() => onSelectVersion(v.id)}
                className="w-full text-left px-2 py-1 text-xs rounded hover:bg-[var(--surface-2)] transition-colors flex items-center gap-2"
                style={{
                  background: isSelected ? "var(--surface-2)" : "transparent",
                  borderLeft: isSelected ? "2px solid var(--accent)" : "2px solid transparent",
                }}
              >
                <span className="font-mono text-[10px]" style={{ color: "var(--text-muted)" }}>
                  {v.data.method}
                </span>
                <span className="truncate flex-1" style={{ color: "var(--text-secondary)" }}>
                  {v.data.url || "(empty)"}
                </span>
                <span className="text-[10px] flex-shrink-0" style={{ color: "var(--text-muted)" }}>
                  {date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                </span>
              </button>
            );
          })}
        </div>
      )}

      {/* Executions */}
      <SectionHeader section="executions" label="Executions" count={executions.length} />
      {openSections.has("executions") && (
        <div className="px-1">
          {groupedExecutions.map(group => (
            <div key={group.label}>
              <div className="px-2 py-0.5 text-[10px] font-medium" style={{ color: "var(--text-muted)" }}>
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
                    className="w-full text-left px-2 py-1 text-xs rounded hover:bg-[var(--surface-2)] transition-colors flex items-center gap-2"
                    style={{
                      background: isSelected ? "var(--surface-2)" : "transparent",
                      borderLeft: isSelected ? "2px solid var(--accent)" : "2px solid transparent",
                    }}
                  >
                    <span className="font-mono font-bold text-[10px]" style={{ color: statusColor(status) }}>
                      {status}
                    </span>
                    <span className="text-[10px]" style={{ color: "var(--text-muted)" }}>
                      {exec.latency_ms}ms
                    </span>
                    <span className="flex-1" />
                    <span className="text-[10px] flex-shrink-0" style={{ color: "var(--text-muted)" }}>
                      {date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                    </span>
                  </button>
                );
              })}
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
