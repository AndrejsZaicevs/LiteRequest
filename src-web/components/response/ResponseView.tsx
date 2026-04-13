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
      {/* Status bar — full-width separator */}
      <div
        className="flex items-center gap-3 px-4 h-10 text-xs flex-shrink-0"
        style={{ background: "var(--surface-1)", borderBottom: "1px solid var(--border)" }}
      >
        <span
          className="font-mono font-bold px-1.5 rounded"
          style={{
            color: statusColor(response.status),
            background: `${statusColor(response.status)}15`,
          }}
        >
          {response.status} {response.status_text}
        </span>
        <span className="tabular-nums font-mono" style={{ color: "var(--text-muted)" }}>
          {latency}ms
        </span>
        <span className="font-mono" style={{ color: "var(--text-muted)" }}>
          {formattedSize}
        </span>

        <div className="flex-1" />

        {(["body", "headers"] as const).map(t => (
          <button
            key={t}
            onClick={() => setTab(t)}
            className="capitalize text-xs px-3 py-1.5"
            style={{
              color: tab === t ? "var(--accent)" : "var(--text-muted)",
              borderBottom: tab === t ? "2px solid var(--accent)" : "2px solid transparent",
            }}
          >
            {t}{t === "headers" ? ` (${headerCount})` : ""}
          </button>
        ))}
      </div>

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
    <pre className="p-4 font-mono text-sm whitespace-pre-wrap break-all leading-relaxed" style={{ color: "var(--text-primary)" }}>
      {formatted}
    </pre>
  );
}

function ResponseHeaders({ headers }: { headers: Record<string, string> }) {
  const entries = Object.entries(headers);
  return (
    <div>
      {entries.map(([key, value], i) => (
        <div key={i} className="kv-row">
          <div className="kv-cell" style={{ color: "var(--accent)", fontWeight: 500, flex: "0 0 40%" }}>
            {key}
          </div>
          <div className="kv-divider" />
          <div className="kv-cell" style={{ color: "var(--text-secondary)", wordBreak: "break-all" }}>
            {value}
          </div>
        </div>
      ))}
    </div>
  );
}
