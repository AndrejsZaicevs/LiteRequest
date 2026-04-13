import { useState, useMemo } from "react";
import type { ResponseData } from "../../lib/types";
import { statusColor } from "../../lib/types";

interface ResponseViewProps {
  response: ResponseData | null;
  latency: number;
  isLoading: boolean;
}

type Tab = "body" | "headers";

export function ResponseView({ response, latency, isLoading }: ResponseViewProps) {
  const [tab, setTab] = useState<Tab>("body");

  if (isLoading) {
    return (
      <div className="h-full flex items-center justify-center" style={{ background: "var(--surface-0)" }}>
        <div className="text-sm animate-pulse" style={{ color: "var(--accent)" }}>
          Sending request…
        </div>
      </div>
    );
  }

  if (!response) {
    return (
      <div className="h-full flex items-center justify-center" style={{ background: "var(--surface-0)" }}>
        <div className="text-xs" style={{ color: "var(--text-muted)" }}>
          Send a request to see the response
        </div>
      </div>
    );
  }

  const headerCount = Object.keys(response.headers).length;
  const bodySize = new Blob([response.body]).size;
  const formattedSize = bodySize > 1024 ? `${(bodySize / 1024).toFixed(1)} KB` : `${bodySize} B`;

  return (
    <div className="h-full flex flex-col overflow-hidden" style={{ background: "var(--surface-0)" }}>
      {/* Status bar */}
      <div
        className="flex items-center gap-3 px-3 h-7 text-xs border-b flex-shrink-0"
        style={{ borderColor: "var(--border)", background: "var(--surface-1)" }}
      >
        <span className="font-mono font-bold" style={{ color: statusColor(response.status) }}>
          {response.status}
        </span>
        <span style={{ color: "var(--text-muted)" }}>⏱ {latency}ms</span>
        <span style={{ color: "var(--text-muted)" }}>📦 {formattedSize}</span>

        <div className="flex-1" />

        {/* Tabs */}
        {(["body", "headers"] as const).map(t => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className="capitalize"
            style={{
              color: tab === t ? "var(--accent)" : "var(--text-muted)",
            }}
          >
            {t}{t === "headers" ? ` (${headerCount})` : ""}
          </button>
        ))}
      </div>

      {/* Content */}
      <div className="flex-1 overflow-auto">
        {tab === "body" && <ResponseBody body={response.body} />}
        {tab === "headers" && <ResponseHeaders headers={response.headers} />}
      </div>
    </div>
  );
}

function ResponseBody({ body }: { body: string }) {
  const formatted = useMemo(() => {
    try {
      return JSON.stringify(JSON.parse(body), null, 2);
    } catch {
      return body;
    }
  }, [body]);

  if (!body) {
    return (
      <div className="flex items-center justify-center h-full text-xs" style={{ color: "var(--text-muted)" }}>
        Empty response body
      </div>
    );
  }

  return (
    <pre className="p-3 font-mono text-xs whitespace-pre-wrap break-all" style={{ color: "var(--text-primary)" }}>
      {formatted}
    </pre>
  );
}

function ResponseHeaders({ headers }: { headers: Record<string, string> }) {
  const entries = Object.entries(headers);
  return (
    <div className="text-xs">
      {entries.map(([key, value], i) => (
        <div key={i} className="flex border-b" style={{ borderColor: "var(--border)" }}>
          <div className="px-3 py-1.5 font-mono font-medium w-1/3 flex-shrink-0" style={{ color: "var(--accent)", borderRight: "1px solid var(--border)" }}>
            {key}
          </div>
          <div className="px-3 py-1.5 font-mono break-all" style={{ color: "var(--text-secondary)" }}>
            {value}
          </div>
        </div>
      ))}
    </div>
  );
}
