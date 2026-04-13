import { useState, useEffect, useRef, useMemo } from "react";
import type { Collection, Folder, Request } from "../../lib/types";

interface GlobalSearchProps {
  collections: Collection[];
  folders: Folder[];
  requests: Request[];
  onClose: () => void;
  onSelectRequest: (req: Request) => void;
  onSelectCollection: (id: string) => void;
}

interface SearchResult {
  type: "collection" | "folder" | "request";
  id: string;
  label: string;
  detail: string;
  item: Collection | Folder | Request;
}

export function GlobalSearch({
  collections, folders, requests,
  onClose, onSelectRequest, onSelectCollection,
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
        out.push({ type: "request", id: r.id, label: r.name, detail: col?.name ?? "", item: r });
      }
    }

    return out.slice(0, 50);
  }, [query, collections, folders, requests]);

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

  return (
    <div className="fixed inset-0 z-50 flex items-start justify-center pt-24" onClick={onClose}>
      <div className="absolute inset-0" style={{ background: "rgba(0,0,0,0.5)" }} />
      <div
        className="relative w-full max-w-lg rounded-lg shadow-2xl overflow-hidden"
        style={{ background: "var(--surface-1)", border: "1px solid var(--border)" }}
        onClick={(e) => e.stopPropagation()}
      >
        <input
          ref={inputRef}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Search collections, folders, requests..."
          className="w-full px-4 py-3 text-sm bg-transparent border-none outline-none"
          style={{ color: "var(--text-primary)", borderBottom: "1px solid var(--border)" }}
        />

        {results.length > 0 && (
          <div className="max-h-80 overflow-y-auto">
            {results.map((r, i) => (
              <button
                key={`${r.type}-${r.id}`}
                onClick={() => handleSelect(r)}
                className="w-full text-left px-4 py-2 flex items-center gap-2 text-xs"
                style={{
                  background: i === selectedIndex ? "var(--surface-2)" : "transparent",
                }}
              >
                <span
                  className="px-1 py-0.5 rounded text-[10px] font-medium uppercase"
                  style={{
                    background: r.type === "request" ? "var(--accent)" : r.type === "collection" ? "var(--success)" : "var(--warning)",
                    color: "#fff",
                  }}
                >
                  {r.type.slice(0, 3)}
                </span>
                {r.type === "request" && (
                  <span className="text-[10px] font-mono" style={{ color: "var(--text-muted)" }}>
                    REQ
                  </span>
                )}
                <span className="truncate" style={{ color: "var(--text-primary)" }}>{r.label}</span>
                <span className="truncate ml-auto text-[10px]" style={{ color: "var(--text-muted)" }}>{r.detail}</span>
              </button>
            ))}
          </div>
        )}

        {query && results.length === 0 && (
          <div className="px-4 py-6 text-center text-xs" style={{ color: "var(--text-muted)" }}>
            No results found
          </div>
        )}
      </div>
    </div>
  );
}
