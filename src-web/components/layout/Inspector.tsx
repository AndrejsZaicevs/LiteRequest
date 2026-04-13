import { useState, useMemo, useEffect } from "react";
import { ChevronDown, ChevronRight, SlidersHorizontal } from "lucide-react";
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

function CollapsibleSection({ title, count, isOpen, onToggle, children }: {
  title: string; count: number; isOpen: boolean; onToggle: () => void; children: React.ReactNode;
}) {
  return (
    <div className="border-b border-gray-800/60 last:border-0">
      <button
        onClick={onToggle}
        className="w-full flex items-center justify-between p-3 hover:bg-[#1a1a1a] transition-colors select-none"
      >
        <div className="flex items-center gap-2">
          {isOpen ? <ChevronDown size={14} className="text-gray-500" /> : <ChevronRight size={14} className="text-gray-500" />}
          <span className="text-[11px] font-bold tracking-wider text-gray-400 uppercase">{title}</span>
        </div>
        {count > 0 && (
          <span className="text-[10px] bg-gray-800 text-gray-400 px-1.5 py-0.5 rounded-full">
            {count}
          </span>
        )}
      </button>
      {isOpen && (
        <div className="px-2 pb-3">
          {children}
        </div>
      )}
    </div>
  );
}

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

  const filteredExecutions = useMemo(() => {
    let filtered = executions;
    if (execEnvFilter !== "all") filtered = filtered.filter(e => e.environment_id === execEnvFilter);
    if (execVersionFilter !== "all") filtered = filtered.filter(e => e.version_id === execVersionFilter);
    return filtered;
  }, [executions, execEnvFilter, execVersionFilter]);

  const groupByDate = <T,>(items: T[], getDate: (item: T) => string) => {
    const groups: { label: string; items: T[] }[] = [];
    const now = new Date();
    const today = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterday = new Date(today.getTime() - 86400000);
    const todayItems: T[] = [], yesterdayItems: T[] = [], olderItems: T[] = [];
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
    <div className="h-full flex flex-col overflow-hidden bg-[#161616]">
      {/* Header */}
      <div className="h-12 border-b border-[var(--border)] flex items-center px-4 gap-2 flex-shrink-0">
        <SlidersHorizontal size={14} className="text-gray-400" />
        <span className="font-semibold text-sm text-gray-200">Inspector</span>
      </div>

      <div className="flex-1 overflow-y-auto">
        {/* Query Params */}
        <CollapsibleSection
          title="Query Params"
          count={enabledParams}
          isOpen={openSections.has("params")}
          onToggle={() => toggleSection("params")}
        >
          <KvTable rows={data.query_params} onChange={updateParams} placeholder={{ key: "param", value: "value" }} />
        </CollapsibleSection>

        {/* Path Params */}
        {pathParams.length > 0 && (
          <CollapsibleSection
            title="Path Variables"
            count={pathParams.length}
            isOpen={openSections.has("pathParams")}
            onToggle={() => toggleSection("pathParams")}
          >
            <KvTable rows={pathParams} onChange={updatePathParams} placeholder={{ key: "param", value: "value" }} fixedKeys />
          </CollapsibleSection>
        )}

        {/* Headers */}
        <CollapsibleSection
          title="Headers"
          count={enabledHeaders}
          isOpen={openSections.has("headers")}
          onToggle={() => toggleSection("headers")}
        >
          <KvTable rows={data.headers} onChange={updateHeaders} placeholder={{ key: "header", value: "value" }} />
        </CollapsibleSection>

        {/* Versions */}
        <CollapsibleSection
          title="Versions"
          count={versions.length}
          isOpen={openSections.has("versions")}
          onToggle={() => toggleSection("versions")}
        >
          <div className="flex flex-col gap-1">
            {groupedVersions.map(group => (
              <div key={group.label}>
                <div className="text-xs text-gray-500 mb-1.5 mt-1">{group.label}</div>
                {group.items.map(v => {
                  const isSelected = v.id === selectedVersionId;
                  const date = new Date(v.created_at);
                  return (
                    <button
                      key={v.id}
                      onClick={() => onSelectVersion(v.id)}
                      className={`w-full rounded p-2 cursor-pointer mb-1 text-left transition-colors ${
                        isSelected
                          ? "bg-[#242424] border border-gray-700/50 border-l-2 border-l-blue-500"
                          : "hover:bg-[#1a1a1a] border border-transparent"
                      }`}
                    >
                      <div className="flex items-center justify-between">
                        <span className={`text-xs font-semibold ${isSelected ? "text-blue-400" : "text-gray-400"}`}>
                          {v.data.method}
                        </span>
                        <span className="text-xs font-mono text-gray-500 truncate ml-2 flex-1 text-right">
                          {v.data.url || "(empty)"}
                        </span>
                      </div>
                      <div className="text-gray-500 text-xs mt-1">
                        {date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                      </div>
                    </button>
                  );
                })}
              </div>
            ))}
          </div>
        </CollapsibleSection>

        {/* Executions */}
        <CollapsibleSection
          title="Executions"
          count={executions.length}
          isOpen={openSections.has("executions")}
          onToggle={() => toggleSection("executions")}
        >
          <div>
            {/* Filters */}
            {(environments.length > 0 || versions.length > 1) && (
              <div className="flex items-center gap-2 mb-2">
                {environments.length > 0 && (
                  <select
                    value={execEnvFilter}
                    onChange={(e) => setExecEnvFilter(e.target.value)}
                    className="bg-[#1a1a1a] border border-gray-700 text-gray-300 rounded text-[11px] px-2 py-1 outline-none"
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
                    className="bg-[#1a1a1a] border border-gray-700 text-gray-300 rounded text-[11px] px-2 py-1 outline-none"
                  >
                    <option value="all">All versions</option>
                    {versions.map((v, i) => (
                      <option key={v.id} value={v.id}>v{versions.length - i}</option>
                    ))}
                  </select>
                )}
              </div>
            )}

            <div className="flex flex-col gap-1">
              {groupedExecutions.map(group => (
                <div key={group.label}>
                  <div className="text-xs text-gray-500 mb-1.5 mt-1">{group.label}</div>
                  {group.items.map(exec => {
                    const isSelected = exec.id === selectedExecutionId;
                    const date = new Date(exec.executed_at);
                    const status = exec.response.status;
                    const isSuccess = status >= 200 && status < 300;
                    return (
                      <button
                        key={exec.id}
                        onClick={() => onSelectExecution(exec.id)}
                        className={`w-full rounded p-2 cursor-pointer mb-1 text-left transition-colors ${
                          isSelected
                            ? `bg-[#242424] border border-gray-700/50 border-l-2 ${isSuccess ? "border-l-green-500" : "border-l-red-500"}`
                            : "hover:bg-[#1a1a1a] border border-transparent"
                        }`}
                      >
                        <div className="flex items-center gap-2">
                          <span
                            className="text-[10px] px-1 py-0.5 rounded font-bold"
                            style={{
                              color: statusColor(status),
                              background: `${statusColor(status)}20`,
                            }}
                          >
                            {status}
                          </span>
                          <span className="text-gray-300 text-xs font-mono">{exec.response.status_text}</span>
                          <span className="text-gray-500 text-xs ml-auto font-mono">{exec.latency_ms}ms</span>
                        </div>
                        <div className="text-gray-500 text-xs mt-1">
                          {date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })}
                        </div>
                      </button>
                    );
                  })}
                </div>
              ))}
            </div>

            {filteredExecutions.length === 0 && (
              <div className="py-4 text-xs text-center text-gray-600">
                No executions{execEnvFilter !== "all" || execVersionFilter !== "all" ? " matching filters" : ""}
              </div>
            )}
          </div>
        </CollapsibleSection>
      </div>
    </div>
  );
}
