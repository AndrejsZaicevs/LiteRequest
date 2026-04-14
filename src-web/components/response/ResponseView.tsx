import { useState, useMemo } from "react";
import { Download } from "lucide-react";
import type { ResponseData } from "../../lib/types";
import { statusColor } from "../../lib/types";
import { save as dialogSave } from "@tauri-apps/plugin-dialog";
import * as api from "../../lib/api";

interface ResponseViewProps {
  response: ResponseData | null;
  latency: number;
  isLoading: boolean;
}

type Tab = "body" | "headers";

function statusDotColor(code: number): string {
  if (code >= 200 && code < 300) return "bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.5)]";
  if (code >= 300 && code < 400) return "bg-yellow-500 shadow-[0_0_8px_rgba(234,179,8,0.5)]";
  if (code >= 400 && code < 500) return "bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.5)]";
  if (code >= 500) return "bg-red-600 shadow-[0_0_8px_rgba(220,38,38,0.5)]";
  return "bg-gray-500";
}

export function ResponseView({ response, latency, isLoading }: ResponseViewProps) {
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
        </div>
      </div>

      <div className="flex-1 overflow-auto bg-[#0d0d0d]">
        {tab === "body" && <ResponseBody body={response.body} isBinary={response.is_binary ?? false} />}
        {tab === "headers" && <ResponseHeaders headers={response.headers} />}
      </div>
    </div>
  );
}

function ResponseBody({ body, isBinary }: { body: string; isBinary: boolean }) {
  const formatted = useMemo(() => {
    if (isBinary) return null;
    try {
      return JSON.stringify(JSON.parse(body), null, 2);
    } catch {
      return body;
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

  return (
    <pre className="p-4 font-mono text-sm whitespace-pre-wrap break-all leading-relaxed text-gray-300">
      {formatted}
    </pre>
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
