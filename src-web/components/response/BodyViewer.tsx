import { useState, useEffect, useMemo } from "react";
import { Check, Copy } from "lucide-react";
import { statusColor } from "../../lib/types";
import { ResponseBody } from "./ResponseView";
import * as api from "../../lib/api";

/**
 * Standalone floating window showing a single execution's response body.
 * Rendered instead of the main App when the URL has ?bodyViewer=1
 * (see main.tsx); opened from the response toolbar.
 */
export function BodyViewer() {
  const params = useMemo(() => new URLSearchParams(window.location.search), []);
  const executionId = params.get("executionId") ?? "";
  const title = params.get("title") ?? "Response";
  const status = Number(params.get("status") ?? "0");
  const isBinary = params.get("binary") === "1";

  const [body, setBody] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    api.getExecutionBody(executionId)
      .then(setBody)
      .catch(e => setError(String(e)));
  }, [executionId]);

  // Esc closes the window — unless CodeMirror's search panel is open,
  // in which case Esc should close the panel (capture runs before CM).
  useEffect(() => {
    const onKey = async (e: KeyboardEvent) => {
      if (e.key !== "Escape") return;
      if (document.querySelector(".cm-search")) return;
      const { getCurrentWindow } = await import("@tauri-apps/api/window");
      getCurrentWindow().close();
    };
    window.addEventListener("keydown", onKey, { capture: true });
    return () => window.removeEventListener("keydown", onKey, { capture: true });
  }, []);

  const handleCopy = async () => {
    if (body == null) return;
    try {
      let text = body;
      try { text = JSON.stringify(JSON.parse(body), null, 2); } catch { /* not JSON */ }
      await api.copyToClipboard(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    } catch { /* ignore */ }
  };

  return (
    <div className="h-screen flex flex-col bg-[#0d0d0d]">
      <div className="flex items-center gap-2 px-3 py-2 border-b border-gray-800 bg-[#161616] shrink-0">
        {status > 0 && (
          <span className="text-xs font-mono font-bold shrink-0" style={{ color: statusColor(status) }}>
            {status}
          </span>
        )}
        <span className="text-xs font-medium text-gray-300 truncate">{title}</span>
        <button
          onClick={handleCopy}
          title="Copy body"
          className="ml-auto p-1.5 rounded text-gray-500 hover:text-gray-200 hover:bg-gray-700/50 transition-colors shrink-0"
        >
          {copied ? <Check size={13} className="text-green-400" /> : <Copy size={13} />}
        </button>
      </div>
      <div className="flex-1 overflow-auto">
        {error ? (
          <div className="p-3 text-xs text-red-400">Failed to load body: {error}</div>
        ) : body == null ? (
          <div className="flex items-center justify-center h-full text-xs text-gray-600">Loading…</div>
        ) : (
          <ResponseBody body={body} isBinary={isBinary} searchText="" />
        )}
      </div>
    </div>
  );
}
