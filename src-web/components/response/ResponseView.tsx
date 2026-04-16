import { useState, useMemo, useCallback } from "react";
import { Download, Maximize2, Minimize2, Copy, Check } from "lucide-react";
import type { ResponseData } from "../../lib/types";
import { statusColor } from "../../lib/types";
import { save as dialogSave } from "@tauri-apps/plugin-dialog";
import * as api from "../../lib/api";
import CodeMirror from "@uiw/react-codemirror";
import { json } from "@codemirror/lang-json";
import { EditorView } from "@codemirror/view";
import { HighlightStyle, syntaxHighlighting } from "@codemirror/language";
import { tags } from "@lezer/highlight";

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
    <div className="h-full flex flex-col overflow-hidden bg-[#161616]">
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

      <div className="flex-1 overflow-auto bg-[#0d0d0d]">
        {tab === "body" && <ResponseBody body={response.body} isBinary={response.is_binary ?? false} />}
        {tab === "headers" && <ResponseHeaders headers={response.headers} />}
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
    ".cm-selectionBackground": { backgroundColor: "#3b82f625 !important" },
    ".cm-focused .cm-selectionBackground": { backgroundColor: "#3b82f625 !important" },
    ".cm-cursor": { display: "none" },
    ".cm-lineNumbers .cm-gutterElement": { color: "#374151" },
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

function ResponseBody({ body, isBinary }: { body: string; isBinary: boolean }) {
  const { isJson, formatted } = useMemo(() => {
    if (isBinary || !body) return { isJson: false, formatted: body };
    try {
      const parsed = JSON.parse(body);
      return { isJson: true, formatted: JSON.stringify(parsed, null, 2) };
    } catch {
      return { isJson: false, formatted: body };
    }
  }, [body, isBinary]);

  const jsonExtensions = useMemo(
    () => [responseViewerTheme, syntaxHighlighting(responseSyntax), json(), EditorView.lineWrapping, EditorView.editable.of(false)],
    []
  );

  const [copied, setCopied] = useState(false);
  const handleCopy = useCallback(() => {
    navigator.clipboard.writeText(formatted).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  }, [formatted]);

  // Show copy button for JSON always, or plain text under 512 KB
  const showCopy = !isBinary && !!body && (isJson || body.length < 512 * 1024);

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
      <div className="relative h-full">
        {showCopy && (
          <button
            onClick={handleCopy}
            title="Copy response body"
            className="absolute top-3 right-3 z-10 flex items-center gap-1.5 px-2 py-1 rounded text-xs bg-gray-800/80 hover:bg-gray-700 text-gray-400 hover:text-gray-200 transition-colors backdrop-blur-sm border border-gray-700/50"
          >
            {copied ? <Check size={11} className="text-green-400" /> : <Copy size={11} />}
            {copied ? "Copied!" : "Copy"}
          </button>
        )}
        <CodeMirror
          value={formatted}
          height="100%"
          theme="none"
          extensions={jsonExtensions}
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
      </div>
    );
  }

  return (
    <div className="relative h-full overflow-auto">
      {showCopy && (
        <button
          onClick={handleCopy}
          title="Copy response body"
          className="absolute top-3 right-3 z-10 flex items-center gap-1.5 px-2 py-1 rounded text-xs bg-gray-800/80 hover:bg-gray-700 text-gray-400 hover:text-gray-200 transition-colors backdrop-blur-sm border border-gray-700/50"
        >
          {copied ? <Check size={11} className="text-green-400" /> : <Copy size={11} />}
          {copied ? "Copied!" : "Copy"}
        </button>
      )}
      <pre className="p-4 font-mono text-sm whitespace-pre-wrap break-all leading-relaxed text-gray-300">
        {formatted}
      </pre>
    </div>
  );
}

function ResponseHeaders({ headers }: { headers: Record<string, string> }) {
  const entries = Object.entries(headers);
  return (
    <div className="p-2">
      {entries.map(([key, value], i) => (
        <div key={i} className="flex items-center gap-2 py-1 px-2 text-xs font-mono hover:bg-[#1a1a1a] rounded">
          <span className="text-blue-400 font-medium shrink-0" style={{ minWidth: "35%" }}>
            {key}
          </span>
          <span className="text-gray-400 break-all">
            {value}
          </span>
        </div>
      ))}
    </div>
  );
}
