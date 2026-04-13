import { useState, useEffect, useRef, useMemo } from "react";
import { Search, FileText, Package, Clock, Layers } from "lucide-react";
import { METHOD_STYLES } from "../../lib/types";
import type { SearchHit } from "../../lib/types";
import * as api from "../../lib/api";

interface GlobalSearchProps {
  onClose: () => void;
  onNavigate: (requestId: string, versionId?: string | null, executionId?: string | null, collectionId?: string | null) => void;
}

const GROUP_ORDER = ["request", "version", "version_old", "execution", "collection"] as const;
const GROUP_LABELS: Record<string, string> = {
  request: "Saved Requests",
  version: "Request Content",
  version_old: "Older Versions",
  execution: "Execution History",
  collection: "Collections",
};

function statusColor(code: number) {
  if (code >= 200 && code < 300) return "text-green-400";
  if (code >= 300 && code < 400) return "text-yellow-400";
  if (code >= 400) return "text-red-400";
  return "text-gray-400";
}

function formatDate(iso: string) {
  try {
    return new Date(iso).toLocaleString(undefined, { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" });
  } catch { return iso; }
}

export function GlobalSearch({ onClose, onNavigate }: GlobalSearchProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchHit[]>([]);
  const [loading, setLoading] = useState(false);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => { inputRef.current?.focus(); }, []);

  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    const q = query.trim();
    if (!q) { setResults([]); setLoading(false); return; }

    setLoading(true);
    debounceRef.current = setTimeout(async () => {
      try {
        const hits = await api.searchAll(q);
        setResults(hits);
      } catch {
        setResults([]);
      } finally {
        setLoading(false);
      }
    }, 300);
    return () => { if (debounceRef.current) clearTimeout(debounceRef.current); };
  }, [query]);

  useEffect(() => { setSelectedIndex(0); }, [results]);

  const grouped = useMemo(() => {
    const map = new Map<string, SearchHit[]>();
    for (const hit of results) {
      const g = map.get(hit.result_type) ?? [];
      g.push(hit);
      map.set(hit.result_type, g);
    }
    return GROUP_ORDER.filter(k => map.has(k)).map(k => ({ key: k, label: GROUP_LABELS[k], items: map.get(k)! }));
  }, [results]);

  const flatList = useMemo(() => grouped.flatMap(g => g.items), [grouped]);

  const handleSelect = (hit: SearchHit) => {
    if (hit.result_type === "collection") {
      onNavigate("", null, null, hit.collection_id);
    } else {
      onNavigate(hit.request_id, hit.version_id, hit.execution_id);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") { onClose(); return; }
    if (e.key === "ArrowDown") { e.preventDefault(); setSelectedIndex(i => Math.min(i + 1, flatList.length - 1)); }
    if (e.key === "ArrowUp") { e.preventDefault(); setSelectedIndex(i => Math.max(i - 1, 0)); }
    if (e.key === "Enter" && flatList[selectedIndex]) { handleSelect(flatList[selectedIndex]); }
  };

  let flatIdx = -1;

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-[10vh] bg-black/60 backdrop-blur-sm px-4"
      onClick={onClose}
    >
      <div
        className="w-full max-w-2xl bg-[#161616] border border-gray-700 shadow-2xl rounded-xl overflow-hidden flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search input */}
        <div className="flex items-center px-4 py-4 border-b border-gray-800 bg-[#121212]">
          <Search className="text-gray-400 mr-3 shrink-0" size={18} />
          <input
            ref={inputRef}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            placeholder="Search requests, URLs, headers, body, history…"
            className="flex-1 bg-transparent text-gray-200 placeholder-gray-500 text-base"
            style={{ border: "none", outline: "none", padding: 0 }}
          />
          {loading && <span className="text-xs text-gray-500 ml-3 animate-pulse">searching…</span>}
          {!loading && <span className="text-[10px] text-gray-500 bg-gray-800 px-1.5 py-0.5 rounded border border-gray-700 ml-3">ESC</span>}
        </div>

        {/* Results */}
        {grouped.length > 0 && (
          <div className="max-h-[60vh] overflow-y-auto p-2 flex flex-col gap-4 bg-[#161616]">
            {grouped.map(group => (
              <div key={group.key}>
                <div className="px-3 py-1.5 text-[10px] font-bold text-gray-500 uppercase tracking-wider">{group.label}</div>
                <div className="flex flex-col gap-0.5 mt-1">
                  {group.items.map(hit => {
                    flatIdx++;
                    const idx = flatIdx;
                    const isSelected = idx === selectedIndex;
                    const colors = hit.method ? METHOD_STYLES[hit.method] : undefined;

                    return (
                      <button
                        key={`${hit.result_type}-${hit.request_id}-${hit.version_id ?? ""}-${hit.execution_id ?? ""}`}
                        onClick={() => handleSelect(hit)}
                        className={`w-full flex items-start gap-3 px-3 py-2 rounded-md transition-colors text-left group ${
                          isSelected ? "bg-blue-500/10 ring-1 ring-blue-500/30" : "hover:bg-[#1a1a1a]"
                        }`}
                      >
                        {/* Icon */}
                        <div className="mt-0.5 shrink-0">
                          {hit.result_type === "collection" && <Package size={14} className="text-gray-500" />}
                          {(hit.result_type === "request") && <FileText size={14} className="text-blue-400" />}
                          {(hit.result_type === "version" || hit.result_type === "version_old") && <Layers size={14} className="text-purple-400" />}
                          {hit.result_type === "execution" && <Clock size={14} className="text-amber-400" />}
                        </div>

                        {/* Main content */}
                        <div className="flex-1 overflow-hidden min-w-0">
                          <div className="flex items-center gap-2 flex-wrap">
                            <span className="text-sm font-medium text-gray-200 truncate">{hit.request_name || hit.collection_name}</span>
                            {hit.method && colors && (
                              <span className={`text-[10px] px-1.5 py-0.5 rounded font-mono border ${colors.bg} ${colors.text} ${colors.border} shrink-0`}>
                                {hit.method}
                              </span>
                            )}
                            {hit.result_type === "execution" && hit.status != null && (
                              <span className={`text-xs font-mono shrink-0 ${statusColor(hit.status)}`}>{hit.status}</span>
                            )}
                            {hit.result_type === "execution" && hit.executed_at && (
                              <span className="text-[11px] text-gray-600 shrink-0">{formatDate(hit.executed_at)}</span>
                            )}
                          </div>

                          {/* Match context */}
                          <div className="flex items-start gap-2 mt-1">
                            <span className="text-[10px] px-1.5 py-0.5 rounded bg-gray-800 text-gray-400 border border-gray-700 shrink-0">
                              {hit.match_field}
                            </span>
                            <span className="text-[11px] text-gray-500 truncate font-mono">{hit.match_context}</span>
                          </div>

                          {/* URL for version hits */}
                          {hit.url && hit.result_type !== "request" && (
                            <div className="text-[11px] text-gray-600 truncate mt-0.5">{hit.url}</div>
                          )}
                          {hit.collection_name && hit.result_type !== "collection" && (
                            <div className="text-[11px] text-gray-600 truncate">{hit.collection_name}</div>
                          )}
                        </div>
                      </button>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>
        )}

        {query.trim() && !loading && results.length === 0 && (
          <div className="px-4 py-10 text-center text-sm text-gray-600">
            No results for <span className="text-gray-400">"{query}"</span>
          </div>
        )}

        {!query.trim() && (
          <div className="px-4 py-8 text-center text-sm text-gray-600">
            Type to search requests, URLs, headers, body, and execution history
          </div>
        )}

        {/* Footer */}
        <div className="border-t border-gray-800 p-2.5 bg-[#121212] flex items-center gap-6 text-[11px] text-gray-500">
          <div className="flex items-center gap-1.5">
            <span className="bg-gray-800 px-1 rounded border border-gray-700">↑</span>
            <span className="bg-gray-800 px-1 rounded border border-gray-700">↓</span>
            navigate
          </div>
          <div className="flex items-center gap-1.5">
            <span className="bg-gray-800 px-1.5 rounded border border-gray-700">↵</span>
            select
          </div>
        </div>
      </div>
    </div>
  );
}
