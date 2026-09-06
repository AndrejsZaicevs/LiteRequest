import { useState, useMemo, useEffect, useRef } from "react";
import { ChevronDown, ChevronRight, Zap } from "lucide-react";
import type { RequestData, RequestExecution, Environment, KeyValuePair, VarRow } from "../../lib/types";
import { statusColor, formatSize, formatDate, formatTime } from "../../lib/types";
import { KvTable } from "../inspector/KvTable";
import { CollapsibleSection } from "../shared/CollapsibleSection";

/** Extracts path param names from a URL inline — mirrors the Rust logic. */
function parsePathParamNames(url: string): string[] {
  const path = url.split("?")[0].split("#")[0];
  return path.split("/").filter(seg => seg.startsWith(":") && seg.length > 1).map(seg => seg.slice(1));
}

interface InspectorProps {
  data: RequestData;
  onChange: (data: RequestData) => void;
  executions: RequestExecution[];
  selectedExecutionId: string | null;
  onSelectExecution: (id: string) => void;
  onDeleteExecution?: (id: string) => void;
  environments: Environment[];
  variables?: Record<string, string>;
  operativeVarRows?: VarRow[];
  onOperativeVarChange?: (row: VarRow, value: string) => void;
}

type Section = "params" | "headers" | "pathParams" | "executions" | "variables";

type ExecEnvFilter = "selected" | "all";

function DateGroup({ label, isOpen, onToggle, children }: {
  label: string; isOpen: boolean; onToggle: () => void; children: React.ReactNode;
}) {
  return (
    <div>
      <button
        onClick={onToggle}
        className="flex items-center gap-1 w-full text-left py-1.5 px-1 text-[11px] text-gray-500 hover:text-gray-400 transition-colors select-none"
      >
        {isOpen ? <ChevronDown size={10} /> : <ChevronRight size={10} />}
        {label}
      </button>
      {isOpen && children}
    </div>
  );
}

export function Inspector({
  data, onChange,
  executions,
  selectedExecutionId,
  onSelectExecution, onDeleteExecution,
  environments, variables = {},
  operativeVarRows = [], onOperativeVarChange,
}: InspectorProps) {
  const [openSections, setOpenSections] = useState<Set<Section>>(
    new Set(["params", "headers", "pathParams", "variables"])
  );
  const [execEnvFilter, setExecEnvFilter] = useState<ExecEnvFilter>("selected");
  const [execCtxMenu, setExecCtxMenu] = useState<{ x: number; y: number; id: string } | null>(null);
  const execCtxMenuRef = useRef<HTMLDivElement>(null);

  // Close the execution context menu on any mousedown outside it
  useEffect(() => {
    if (!execCtxMenu) return;
    const handler = (e: MouseEvent) => {
      if (execCtxMenuRef.current && !execCtxMenuRef.current.contains(e.target as Node)) {
        setExecCtxMenu(null);
      }
    };
    document.addEventListener("mousedown", handler, { capture: true });
    return () => document.removeEventListener("mousedown", handler, { capture: true });
  }, [execCtxMenu]);

  // Auto-open executions section when a specific execution is selected.
  // Only widen the env filter to "all" if the execution isn't visible under
  // the current filter (e.g. when navigating here from global search with a
  // cross-env execution).
  useEffect(() => {
    if (!selectedExecutionId) return;
    setOpenSections(prev => new Set([...prev, "executions"]));

    const exec = executions.find(e => e.id === selectedExecutionId);
    if (!exec) return;
    const activeEnvId = environments.find(e => e.is_active)?.id ?? null;
    const effectiveEnv = execEnvFilter === "selected" ? activeEnvId : null;
    if (effectiveEnv && exec.environment_id !== effectiveEnv) {
      setExecEnvFilter("all");
    }
  }, [selectedExecutionId]); // eslint-disable-line react-hooks/exhaustive-deps

  // Collapsed date groups — collapse all except "Today" by default
  const [collapsedExecGroups, setCollapsedExecGroups] = useState<Set<string>>(
    new Set(["Yesterday", "Older"])
  );

  const [pathParams, setPathParams] = useState<KeyValuePair[]>([]);

  useEffect(() => {
    const paramNames = parsePathParamNames(data.url ?? "");
    if (paramNames.length === 0) { setPathParams([]); return; }
    const existing = data.path_params ?? [];
    const merged = paramNames.map(name => existing.find(p => p.key === name) ?? { key: name, value: "", enabled: true });
    setPathParams(merged);
  }, [data.url, data.path_params]);

  const toggleSection = (s: Section) => {
    setOpenSections(prev => {
      const next = new Set(prev);
      if (next.has(s)) next.delete(s); else next.add(s);
      return next;
    });
  };

  const toggleExecGroup = (label: string) => {
    setCollapsedExecGroups(prev => {
      const next = new Set(prev);
      if (next.has(label)) next.delete(label); else next.add(label);
      return next;
    });
  };

  const enabledParams = data.query_params.filter(p => p.enabled).length;
  const enabledHeaders = data.headers.filter(h => h.enabled).length;

  const updateParams = (params: KeyValuePair[]) => onChange({ ...data, query_params: params });
  const updateHeaders = (headers: KeyValuePair[]) => onChange({ ...data, headers });
  const updatePathParams = (pp: KeyValuePair[]) => {
    setPathParams(pp);
    onChange({ ...data, path_params: pp });
  };

  const activeEnv = environments.find(e => e.is_active);
  const activeEnvId = activeEnv?.id ?? null;

  // Executions are shown across all versions — only the environment is filtered.
  const filteredExecutions = useMemo(() => {
    const effectiveEnv = execEnvFilter === "selected" ? activeEnvId : null;
    return effectiveEnv ? executions.filter(e => e.environment_id === effectiveEnv) : executions;
  }, [executions, execEnvFilter, activeEnvId]);

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

  const groupedExecutions = useMemo(() => groupByDate(filteredExecutions, e => e.executed_at), [filteredExecutions]);

  // Only show operative vars that are actually referenced in the current request
  const usedVarNames = useMemo(() => {
    const VAR_RE = /\{\{([^}]+)\}\}/g;
    const names = new Set<string>();
    const scan = (s: string) => { let m; while ((m = VAR_RE.exec(s)) !== null) names.add(m[1].trim()); };
    scan(data.url ?? "");
    scan(data.body ?? "");
    for (const p of data.query_params ?? []) { scan(p.key); scan(p.value); }
    for (const h of data.headers ?? []) { scan(h.key); scan(h.value); }
    for (const p of data.path_params ?? []) { scan(p.key); scan(p.value); }
    return names;
  }, [data]);

  const visibleOperativeVars = useMemo(
    () => operativeVarRows.filter(r => usedVarNames.has(r.key)),
    [operativeVarRows, usedVarNames]
  );

  return (
    <div className="h-full flex flex-col overflow-hidden bg-[#161616]">
      <div className="flex-1 overflow-y-auto">

        {/* Operative Variables — only those used in this request */}
        {visibleOperativeVars.length > 0 && (
          <CollapsibleSection
            title="Variables"
            count={visibleOperativeVars.length}
            isOpen={openSections.has("variables")}
            onToggle={() => toggleSection("variables")}
            icon={<Zap size={11} className="text-amber-400" />}
          >
            <div className="flex flex-col gap-0.5">
              {visibleOperativeVars.map(row => (
                <div key={row.def_id} className="group flex items-center gap-1.5 h-7 border-b border-transparent hover:border-gray-800">
                  <span className="min-w-[80px] max-w-[160px] w-[40%] shrink text-xs px-1.5 py-0.5 text-amber-400/80 font-mono truncate">
                    {row.key}
                  </span>
                  <div className="w-px h-3.5 bg-gray-800 shrink-0" />
                  {row.is_secret ? (
                    <input
                      type="password"
                      value={row.value}
                      onChange={e => onOperativeVarChange?.(row, e.target.value)}
                      className="flex-1 min-w-0 bg-transparent text-xs outline-none text-gray-200 placeholder-gray-700 border border-transparent focus:border-gray-700 focus:bg-[#1a1a1a] rounded px-1.5 py-0.5"
                      placeholder="—"
                    />
                  ) : (
                    <input
                      type="text"
                      value={row.value}
                      onChange={e => onOperativeVarChange?.(row, e.target.value)}
                      className="flex-1 min-w-0 bg-transparent text-xs outline-none text-gray-200 placeholder-gray-700 border border-transparent focus:border-gray-700 focus:bg-[#1a1a1a] rounded px-1.5 py-0.5"
                      placeholder="—"
                    />
                  )}
                </div>
              ))}
            </div>
          </CollapsibleSection>
        )}

        {/* Path Params */}
        {pathParams.length > 0 && (
          <CollapsibleSection
            title="Path Variables"
            count={pathParams.length}
            isOpen={openSections.has("pathParams")}
            onToggle={() => toggleSection("pathParams")}
          >
            <KvTable rows={pathParams} onChange={updatePathParams} placeholder={{ key: "param", value: "value" }} fixedKeys variables={variables} />
          </CollapsibleSection>
        )}

        {/* Query Params */}
        <CollapsibleSection
          title="Query Params"
          count={enabledParams}
          isOpen={openSections.has("params")}
          onToggle={() => toggleSection("params")}
        >
          <KvTable rows={data.query_params} onChange={updateParams} placeholder={{ key: "param", value: "value" }} variables={variables} />
        </CollapsibleSection>



        {/* Headers */}
        <CollapsibleSection
          title="Headers"
          count={enabledHeaders}
          isOpen={openSections.has("headers")}
          onToggle={() => toggleSection("headers")}
        >
          <KvTable rows={data.headers} onChange={updateHeaders} placeholder={{ key: "header", value: "value" }} variables={variables} />
        </CollapsibleSection>

        {/* Executions */}
        <CollapsibleSection
          title="Executions"
          count={executions.length}
          isOpen={openSections.has("executions")}
          onToggle={() => toggleSection("executions")}
        >
          <div>
            {/* Env filter */}
            {environments.length > 0 && (
              <div className="flex items-center gap-1.5 mb-3 mt-1">
                {([
                  { value: "selected", label: activeEnv ? activeEnv.name : "Selected env", title: "Only executions from the active environment" },
                  { value: "all", label: "All envs", title: "Executions from every environment" },
                ] as { value: ExecEnvFilter; label: string; title: string }[]).map(opt => (
                  <button
                    key={opt.value}
                    onClick={() => setExecEnvFilter(opt.value)}
                    title={opt.title}
                    className={`px-2.5 py-1 rounded-full text-[11px] font-medium transition-all border ${
                      execEnvFilter === opt.value
                        ? "bg-blue-500 text-white border-blue-500"
                        : "bg-transparent text-gray-500 border-gray-700 hover:border-gray-500 hover:text-gray-300"
                    }`}
                  >
                    {opt.label}
                  </button>
                ))}
              </div>
            )}

            <div className="flex flex-col">
              {groupedExecutions.map(group => (
                <DateGroup
                  key={group.label}
                  label={group.label}
                  isOpen={!collapsedExecGroups.has(group.label)}
                  onToggle={() => toggleExecGroup(group.label)}
                >
                  {group.items.map(exec => {
                    const isSelected = exec.id === selectedExecutionId;
                    const status = exec.response.status;
                    const isSuccess = status >= 200 && status < 300;
                    return (
                      <div
                        key={exec.id}
                        onClick={() => onSelectExecution(exec.id)}
                        onContextMenu={(e) => {
                          e.preventDefault();
                          e.stopPropagation();
                          setExecCtxMenu({ x: e.clientX, y: e.clientY, id: exec.id });
                        }}
                        className={`w-full rounded p-2 cursor-pointer mb-1 text-left transition-colors ${isSelected
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
                        <div className="flex items-baseline text-xs mt-1">
                          <span className="text-gray-600">{formatDate(exec.executed_at)}</span>
                          <span className="text-gray-400 ml-3">{formatTime(exec.executed_at)}</span>
                          {exec.response.size_bytes != null && (
                            <span className="font-mono text-gray-500 ml-auto">{formatSize(exec.response.size_bytes)}</span>
                          )}
                        </div>
                      </div>
                    );
                  })}
                </DateGroup>
              ))}
            </div>

            {filteredExecutions.length === 0 && (
              <div className="py-4 text-xs text-center text-gray-600">
                No executions{execEnvFilter === "selected" && activeEnv ? ` in ${activeEnv.name}` : ""}
              </div>
            )}
          </div>
        </CollapsibleSection>
      </div>

      {execCtxMenu && (
        <div
          ref={execCtxMenuRef}
          className="fixed z-50 rounded-lg shadow-2xl overflow-hidden bg-[#1a1a1a] border border-gray-700 py-1"
          style={{ left: execCtxMenu.x, top: execCtxMenu.y, minWidth: 160 }}
        >
          {onDeleteExecution && (
            <button
              className="w-full text-left px-3 py-2 text-sm text-red-400 hover:bg-[#242424] hover:text-red-300"
              onClick={() => { onDeleteExecution(execCtxMenu.id); setExecCtxMenu(null); }}
            >
              Delete
            </button>
          )}
        </div>
      )}
    </div>
  );
}
