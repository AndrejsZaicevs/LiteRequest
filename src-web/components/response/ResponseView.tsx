import { useState, useMemo, useCallback, useRef } from "react";
import { Download, Maximize2, Minimize2, Copy, Check, Search, X } from "lucide-react";
import type { ResponseData } from "../../lib/types";
import { statusColor } from "../../lib/types";
import { save as dialogSave } from "@tauri-apps/plugin-dialog";
import * as api from "../../lib/api";
import CodeMirror from "@uiw/react-codemirror";
import { json } from "@codemirror/lang-json";
import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";
import { search as cmSearch } from "@codemirror/search";

interface ResponseViewProps {
  response: ResponseData | null;
  latency: number;
  isLoading: boolean;
  isMaximized?: boolean;
  onMaximize?: () => void;
}

type Tab = "body" | "headers";

function statusDotColor(code: number): string {
  if (code >= 200 && code < 300) return "bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]";
  if (code >= 300 && code < 400) return "bg-yellow-500 shadow-[0_0_8px_rgba(234,179,8,0.5)]";
  if (code >= 400 && code < 500) return "bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.5)]";
  if (code >= 500) return "bg-red-600 shadow-[0_0_8px_rgba(220,38,38,0.5)]";
  return "bg-gray-500";
}

export function ResponseView({ response, latency, isLoading, isMaximized, onMaximize }: ResponseViewProps) {
  const [tab, setTab] = useState<Tab>("body");
  const [copied, setCopied] = useState(false);
  const [showSearch, setShowSearch] = useState(false);
  const [searchText, setSearchText] = useState("");
  const searchInputRef = useRef<HTMLInputElement>(null);

  const openSearch = useCallback(() => {
    setShowSearch(true);
    setTimeout(() => searchInputRef.current?.focus(), 30);
  }, []);

  const closeSearch = useCallback(() => {
    setShowSearch(false);
    setSearchText("");
  }, []);

  const handleDownload = async () => {
    if (!response) return;
    const ct = response.headers["content-type"] ?? "";
    // Guess a file extension from content-type
    const ext = ct.includes("json") ? "json"
      : ct.includes("xml") ? "xml"
      : ct.includes("html") ? "html"
      : ct.includes("text/plain") ? "txt"
      : ct.includes("csv") ? "csv"
      : ct.includes("pdf") ? "pdf"
      : ct.includes("png") ? "png"
      : ct.includes("jpeg") || ct.includes("jpg") ? "jpg"
      : ct.includes("gif") ? "gif"
      : ct.includes("webp") ? "webp"
      : ct.includes("svg") ? "svg"
      : ct.includes("zip") ? "zip"
      : "bin";
    const path = await dialogSave({ defaultPath: `response.${ext}` });
    if (path) {
      await api.saveFile(path, response.body, response.is_binary ?? false);
    }
  };

  // Detect JSON body so we can delegate Ctrl+F to CM's native search panel
  const isJsonBody = useMemo(() => {
    if (!response || response.is_binary || !response.body) return false;
    try { JSON.parse(response.body); return true; } catch { return false; }
  }, [response]);

  // Compute copy eligibility: JSON always; plain text under 512 KB
  const copyText = useMemo(() => {
    if (!response || response.is_binary || !response.body) return null;
    try {
      return JSON.stringify(JSON.parse(response.body), null, 2);
    } catch {
      return response.body.length < 512 * 1024 ? response.body : null;
    }
  }, [response]);

  const handleCopy = useCallback(() => {
    if (!copyText) return;
    navigator.clipboard.writeText(copyText).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  }, [copyText]);

  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center bg-[#161616]">
        <div className="text-sm animate-pulse text-blue-400">
          Sending request…
        </div>
      </div>
    );
  }

  if (!response) {
    return (
      <div className="h-full flex items-center justify-center bg-[#161616]">
        <div className="text-xs text-gray-600">
        </div>
      </div>
    );
  }

  const headerCount = Object.keys(response.headers).length;
  const bodySize = response.size_bytes;
  const formattedSize = bodySize > 1024 ? `${(bodySize / 1024).toFixed(1)} KB` : `${bodySize} B`;

  return (
    <div
      className="h-full flex flex-col overflow-hidden bg-[#161616]"
      onKeyDown={e => {
        if ((e.ctrlKey || e.metaKey) && e.key === "f") {
          // JSON body: CM's native Ctrl+F handles it; don't open our bar too
          if (tab === "body" && isJsonBody) return;
          e.preventDefault();
          openSearch();
        }
      }}
      // Make the div focusable so keydown fires when clicking inside
      tabIndex={-1}
      style={{ outline: "none" }}
    >
      {/* Status bar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-gray-800 text-sm flex-shrink-0">
        <div className="flex items-center gap-4">
          <span className="flex items-center gap-2">
            <span className={`w-2 h-2 rounded-full ${statusDotColor(response.status)}`} />
            <span className="font-semibold" style={{ color: statusColor(response.status) }}>
              {response.status} {response.status_text}
            </span>
          </span>
          <span className="text-gray-500 font-mono text-xs">{latency}ms</span>
          <span className="text-gray-500 font-mono text-xs">{formattedSize}</span>
        </div>

        <div className="flex items-center gap-4 text-gray-400">
          {(["body", "headers"] as const).map(t => (
            <button
              key={t}
              onClick={() => setTab(t)}
              className={`capitalize text-sm pb-1 transition-colors ${
                tab === t
                  ? "text-gray-200 border-b-2 border-blue-500 -mb-[9px]"
                  : "hover:text-gray-200"
              }`}
            >
              {t}{t === "headers" ? ` (${headerCount})` : ""}
            </button>
          ))}
          {copyText && tab === "body" && (
            <button
              onClick={handleCopy}
              title="Copy response body"
              className="p-1.5 rounded text-gray-500 hover:text-gray-200 hover:bg-gray-700/50 transition-colors flex items-center gap-1"
            >
              {copied ? <Check size={13} className="text-green-400" /> : <Copy size={13} />}
            </button>
          )}
          <button
            onClick={openSearch}
            title="Search (Ctrl+F)"
            className={`p-1.5 rounded transition-colors ${showSearch ? "text-blue-400 bg-blue-500/10" : "text-gray-500 hover:text-gray-200 hover:bg-gray-700/50"}`}
          >
            <Search size={13} />
          </button>
          <button
            onClick={handleDownload}
            title="Save response to file"
            className="p-1.5 rounded text-gray-500 hover:text-gray-200 hover:bg-gray-700/50 transition-colors"
          >
            <Download size={13} />
          </button>
          {onMaximize && (
            <button
              onClick={onMaximize}
              title={isMaximized ? "Restore split" : "Maximize response"}
              className="p-1.5 rounded text-gray-500 hover:text-gray-200 hover:bg-gray-700/50 transition-colors"
            >
              {isMaximized ? <Minimize2 size={13} /> : <Maximize2 size={13} />}
            </button>
          )}
        </div>
      </div>

      <div className="flex-1 overflow-auto bg-[#0d0d0d] flex flex-col">
        {showSearch && (
          <div className="flex items-center gap-1 px-2 py-1 border-b border-gray-800 bg-[#161616] flex-shrink-0">
            <Search size={12} className="text-gray-500 shrink-0" />
            <input
              ref={searchInputRef}
              value={searchText}
              onChange={e => setSearchText(e.target.value)}
              onKeyDown={e => { if (e.key === "Escape") closeSearch(); }}
              placeholder="Search…"
              className="flex-1 bg-transparent text-xs text-gray-200 outline-none placeholder-gray-600"
            />
            <button onClick={closeSearch} className="p-0.5 text-gray-500 hover:text-gray-300"><X size={13} /></button>
          </div>
        )}
        <div className="flex-1 overflow-auto">
          {tab === "body" && <ResponseBody body={response.body} isBinary={response.is_binary ?? false} searchText={showSearch ? searchText : ""} />}
          {tab === "headers" && <ResponseHeaders headers={response.headers} searchText={showSearch ? searchText : ""} />}
        </div>
      </div>
    </div>
  );
}

const responseViewerTheme = EditorView.theme(
  {
    "&": { height: "100%", fontSize: "13px", backgroundColor: "#0d0d0d", color: "#d1d5db" },
    ".cm-scroller": {
      fontFamily: "'JetBrains Mono', 'Fira Code', 'Cascadia Code', monospace",
      lineHeight: "1.6",
      overflow: "auto",
    },
    ".cm-content": { padding: "16px", caretColor: "transparent" },
    ".cm-focused": { outline: "none" },
    ".cm-editor": { backgroundColor: "#0d0d0d" },
    ".cm-gutters": {
      backgroundColor: "#0d0d0d",
      borderRight: "1px solid #1f2937",
      color: "#4b5563",
      paddingRight: "8px",
    },
    ".cm-activeLineGutter": { backgroundColor: "transparent" },
    ".cm-activeLine": { backgroundColor: "transparent" },
    ".cm-selectionBackground": { backgroundColor: "#3b82f655 !important" },
    ".cm-focused .cm-selectionBackground": { backgroundColor: "#3b82f655 !important" },
    ".cm-cursor": { display: "none" },
    ".cm-lineNumbers .cm-gutterElement": { color: "#374151" },
    // Native CM search panel — styled to match the app
    ".cm-search": {
      backgroundColor: "#1a1a1a",
      borderTop: "1px solid #374151",
      padding: "6px 8px",
      display: "flex",
      gap: "6px",
      flexWrap: "wrap",
      alignItems: "center",
    },
    ".cm-search input": {
      backgroundColor: "#0d0d0d",
      border: "1px solid #374151",
      borderRadius: "4px",
      color: "#d1d5db",
      fontSize: "12px",
      padding: "2px 6px",
      outline: "none",
    },
    ".cm-search input:focus": { borderColor: "#3b82f6" },
    ".cm-search button": {
      backgroundColor: "transparent",
      border: "1px solid #374151",
      borderRadius: "4px",
      color: "#9ca3af",
      fontSize: "11px",
      padding: "2px 6px",
      cursor: "pointer",
    },
    ".cm-search button:hover": { color: "#e5e7eb", borderColor: "#6b7280" },
    ".cm-search label": { color: "#6b7280", fontSize: "11px", display: "flex", alignItems: "center", gap: "3px" },
    ".cm-search .cm-button": { display: "none" }, // hide "Replace" / "Replace all" buttons
    // Search match highlighting
    ".cm-searchMatch": { backgroundColor: "#f59e0b33", outline: "1px solid #f59e0b66" },
    ".cm-searchMatch-selected": { backgroundColor: "#f59e0b77" },
  },
  { dark: true }
);

const responseSyntax = HighlightStyle.define([
  { tag: tags.propertyName, color: "#60a5fa" },
  { tag: tags.string,       color: "#34d399" },
  { tag: tags.number,       color: "#f59e0b" },
  { tag: tags.bool,         color: "#f59e0b" },
  { tag: tags.null,         color: "#9ca3af" },
  { tag: tags.keyword,      color: "#c084fc" },
  { tag: tags.punctuation,  color: "#6b7280" },
  { tag: tags.bracket,      color: "#9ca3af" },
]);

const cmJsonExtensions = [
  responseViewerTheme,
  syntaxHighlighting(responseSyntax),
  json(),
  EditorView.lineWrapping,
  cmSearch({ top: true }),
];

function ResponseBody({ body, isBinary, searchText }: {
  body: string;
  isBinary: boolean;
  searchText: string;
}) {
  const { isJson, formatted } = useMemo(() => {
    if (isBinary || !body) return { isJson: false, formatted: body };
    try {
      const parsed = JSON.parse(body);
      return { isJson: true, formatted: JSON.stringify(parsed, null, 2) };
    } catch {
      return { isJson: false, formatted: body };
    }
  }, [body, isBinary]);

  if (!body) {
    return (
      <div className="flex items-center justify-center h-full text-xs text-gray-600">
        Empty response body
      </div>
    );
  }

  if (isBinary) {
    return (
      <div className="flex flex-col items-center justify-center h-full gap-2 text-gray-500">
        <span className="text-2xl">📦</span>
        <span className="text-sm">Binary response — use the download button to save it</span>
      </div>
    );
  }

  if (isJson) {
    return (
      <CodeMirror
        value={formatted}
        height="100%"
        theme="none"
        extensions={cmJsonExtensions}
        readOnly
        basicSetup={{
          lineNumbers: true,
          foldGutter: true,
          bracketMatching: true,
          closeBrackets: false,
          autocompletion: false,
          highlightActiveLine: false,
          indentOnInput: false,
          tabSize: 2,
        }}
        style={{ height: "100%" }}
      />
    );
  }

  // Plain text — highlight matches inline
  if (searchText) {
    const segments = highlightText(formatted, searchText);
    return (
      <pre className="p-4 font-mono text-sm whitespace-pre-wrap break-all leading-relaxed text-gray-300">
        {segments.map((seg, i) =>
          seg.match
            ? <mark key={i} className="bg-yellow-400/30 text-yellow-200 rounded-sm">{seg.text}</mark>
            : <span key={i}>{seg.text}</span>
        )}
      </pre>
    );
  }

  return (
    <pre className="p-4 font-mono text-sm whitespace-pre-wrap break-all leading-relaxed text-gray-300">
      {formatted}
    </pre>
  );
}

/** Split text into alternating match/non-match segments for inline highlighting. */
function highlightText(text: string, query: string): { text: string; match: boolean }[] {
  if (!query) return [{ text, match: false }];
  const escaped = query.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const re = new RegExp(escaped, "gi");
  const segments: { text: string; match: boolean }[] = [];
  let last = 0;
  let m: RegExpExecArray | null;
  while ((m = re.exec(text)) !== null) {
    if (m.index > last) segments.push({ text: text.slice(last, m.index), match: false });
    segments.push({ text: m[0], match: true });
    last = re.lastIndex;
  }
  if (last < text.length) segments.push({ text: text.slice(last), match: false });
  return segments;
}

function ResponseHeaders({ headers, searchText }: { headers: Record<string, string>; searchText: string }) {
  const entries = Object.entries(headers).filter(([key, value]) => {
    if (!searchText) return true;
    const q = searchText.toLowerCase();
    return key.toLowerCase().includes(q) || value.toLowerCase().includes(q);
  });
  return (
    <div className="p-2">
      {entries.length === 0 && searchText && (
        <div className="text-xs text-gray-600 px-2 py-4 text-center">No headers matching "{searchText}"</div>
      )}
      {entries.map(([key, value], i) => (
        <div key={i} className="flex items-center gap-2 py-1 px-2 text-xs font-mono hover:bg-[#1a1a1a] rounded">
          <span className="text-blue-400 font-medium shrink-0" style={{ minWidth: "35%" }}>
            {searchText ? highlightText(key, searchText).map((s, j) =>
              s.match ? <mark key={j} className="bg-yellow-400/30 text-yellow-200 rounded-sm">{s.text}</mark> : <span key={j}>{s.text}</span>
            ) : key}
          </span>
          <span className="text-gray-400 break-all">
            {searchText ? highlightText(value, searchText).map((s, j) =>
              s.match ? <mark key={j} className="bg-yellow-400/30 text-yellow-200 rounded-sm">{s.text}</mark> : <span key={j}>{s.text}</span>
            ) : value}
          </span>
        </div>
      ))}
    </div>
  );
}
