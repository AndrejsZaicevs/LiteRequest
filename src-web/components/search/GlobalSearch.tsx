import { useState, useEffect, useRef, useMemo } from "react";
import { Search, FileText, Package, Folder, Clock } from "lucide-react";
import type { Collection, Folder as FolderType, Request } from "../../lib/types";
import { METHOD_STYLES } from "../../lib/types";

interface GlobalSearchProps {
  collections: Collection[];
  folders: FolderType[];
  requests: Request[];
  onClose: () => void;
  onSelectRequest: (req: Request) => void;
  onSelectCollection: (id: string) => void;
  requestMeta?: Map<string, { method: string; url: string }>;
}

interface SearchResult {
  type: "collection" | "folder" | "request";
  id: string;
  label: string;
  detail: string;
  item: Collection | FolderType | Request;
  method?: string;
}

export function GlobalSearch({
  collections, folders, requests,
  onClose, onSelectRequest, onSelectCollection,
  requestMeta,
}: GlobalSearchProps) {
  const [query, setQuery] = useState("");
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    inputRef.current?.focus();
  }, []);

  const results = useMemo((): SearchResult[] => {
    if (!query.trim()) return [];
    const q = query.toLowerCase();
    const out: SearchResult[] = [];

    for (const c of collections) {
      if (c.name.toLowerCase().includes(q) || c.base_path.toLowerCase().includes(q)) {
        out.push({ type: "collection", id: c.id, label: c.name, detail: c.base_path, item: c });
      }
    }
    for (const f of folders) {
      if (f.name.toLowerCase().includes(q)) {
        const col = collections.find(c => c.id === f.collection_id);
        out.push({ type: "folder", id: f.id, label: f.name, detail: col?.name ?? "", item: f });
      }
    }
    for (const r of requests) {
      if (r.name.toLowerCase().includes(q)) {
        const col = collections.find(c => c.id === r.collection_id);
        const meta = requestMeta?.get(r.id);
        out.push({ type: "request", id: r.id, label: r.name, detail: col?.name ?? "", item: r, method: meta?.method });
      }
    }

    return out.slice(0, 50);
  }, [query, collections, folders, requests, requestMeta]);

  useEffect(() => { setSelectedIndex(0); }, [query]);

  const handleSelect = (result: SearchResult) => {
    if (result.type === "request") onSelectRequest(result.item as Request);
    else if (result.type === "collection") onSelectCollection(result.id);
    else onClose();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === "Escape") { onClose(); return; }
    if (e.key === "ArrowDown") { e.preventDefault(); setSelectedIndex(i => Math.min(i + 1, results.length - 1)); }
    if (e.key === "ArrowUp") { e.preventDefault(); setSelectedIndex(i => Math.max(i - 1, 0)); }
    if (e.key === "Enter" && results[selectedIndex]) { handleSelect(results[selectedIndex]); }
  };

  const iconFor = (type: string) => {
    if (type === "collection") return <Package size={14} className="text-gray-500 shrink-0" />;
    if (type === "folder") return <Folder size={14} className="text-gray-500 shrink-0" />;
    return <FileText size={14} className="text-gray-500 group-hover:text-blue-400 shrink-0" />;
  };

  // Group results by type
  const grouped = useMemo(() => {
    const groups: { label: string; items: SearchResult[] }[] = [];
    const reqs = results.filter(r => r.type === "request");
    const cols = results.filter(r => r.type === "collection");
    const folds = results.filter(r => r.type === "folder");
    if (reqs.length) groups.push({ label: "Saved Requests", items: reqs });
    if (cols.length) groups.push({ label: "Collections", items: cols });
    if (folds.length) groups.push({ label: "Folders", items: folds });
    return groups;
  }, [results]);

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
            placeholder="Search collections, folders, requests..."
            className="flex-1 bg-transparent border-none outline-none text-gray-200 placeholder-gray-500 text-base"
            style={{ border: "none", padding: 0 }}
          />
          <span className="text-[10px] text-gray-500 bg-gray-800 px-1.5 py-0.5 rounded border border-gray-700 ml-3">ESC</span>
        </div>

        {/* Results */}
        {results.length > 0 && (
          <div className="max-h-[60vh] overflow-y-auto p-2 flex flex-col gap-4 bg-[#161616]">
            {grouped.map(group => (
              <div key={group.label}>
                <div className="px-3 py-1.5 text-[10px] font-bold text-gray-500 uppercase tracking-wider">{group.label}</div>
                <div className="flex flex-col gap-0.5 mt-1">
                  {group.items.map(r => {
                    flatIdx++;
                    const idx = flatIdx;
                    const isSelected = idx === selectedIndex;
                    const method = r.method;
                    const colors = method ? METHOD_STYLES[method] : undefined;

                    return (
                      <button
                        key={`${r.type}-${r.id}`}
                        onClick={() => handleSelect(r)}
                        className={`w-full flex items-center justify-between p-2 rounded-md transition-colors text-left group ${
                          isSelected ? "bg-blue-500/10 ring-1 ring-blue-500/30" : "hover:bg-[#1a1a1a]"
                        }`}
                      >
                        <div className="flex items-center gap-3 overflow-hidden">
                          {iconFor(r.type)}
                          <span className="text-sm text-gray-300 group-hover:text-blue-400 truncate">{r.label}</span>
                          {r.detail && (
                            <span className="text-[11px] text-gray-600 truncate">{r.detail}</span>
                          )}
                        </div>
                        {method && colors && (
                          <span className={`text-[10px] px-1.5 py-0.5 rounded font-mono border ${colors.bg} ${colors.text} ${colors.border} shrink-0 ml-2`}>
                            {method}
                          </span>
                        )}
                      </button>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>
        )}

        {query && results.length === 0 && (
          <div className="px-4 py-8 text-center text-sm text-gray-600">
            No results found
          </div>
        )}

        {/* Footer hints */}
        <div className="border-t border-gray-800 p-2.5 bg-[#121212] flex items-center gap-6 text-[11px] text-gray-500">
          <div className="flex items-center gap-1.5">
            <span className="bg-gray-800 px-1 rounded shadow-sm border border-gray-700">↑</span>
            <span className="bg-gray-800 px-1 rounded shadow-sm border border-gray-700">↓</span>
            to navigate
          </div>
          <div className="flex items-center gap-1.5">
            <span className="bg-gray-800 px-1.5 rounded shadow-sm border border-gray-700">↵</span>
            to select
          </div>
        </div>
      </div>
    </div>
  );
}
